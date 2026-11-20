//! Test harness: runs the real portal on a random port against PostgreSQL, and a small HTTP client
//! that keeps the session cookie and never follows redirects (so tests see every hop).
#![allow(dead_code)]

use std::sync::{Arc, Mutex};

use base64::Engine;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use reqwest::header::{HeaderMap, LOCATION, SET_COOKIE};
use reqwest::redirect::Policy;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use unicloud_portal::config::{CallbackPolicyMode, Config, Environment};
use unicloud_portal::state::AppState;

pub const CB: &str = "http://localhost:9/cb";
pub const OUT: &str = "http://localhost:9/out";

pub fn database_url() -> String {
    std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| "postgres://postgres:dev@127.0.0.1:55432/unicloud".into())
}

pub struct Portal {
    pub base: String,
    pub st: AppState,
}

pub async fn start(env: Environment, hosts: &[&str]) -> Portal {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port());
    let cfg = Config {
        env,
        bind: String::new(),
        database_url: database_url(),
        db_pool_max: 20,
        db_schema: std::env::var("TEST_DB_SCHEMA").unwrap_or_else(|_| "public".into()),
        issuer: Some(base.clone()),
        signing_key_pem: std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/test-only-signing-key.pem")).unwrap(),
        allowed_callback_hosts: hosts.iter().map(|h| h.to_string()).collect(),
        callback_policy: if env == Environment::Production {
            CallbackPolicyMode::Allowlist
        } else {
            CallbackPolicyMode::Any
        },
        secure_cookie: false,
        git_sha: "test".into(),
        replica: "test-replica".into(),
        current_semester: "2026-27-W".into(),
        demo_password: None,
        bootstrap_admin: None,
    };
    let st = unicloud_portal::build_state(cfg)
        .await
        .expect("database: start it with scripts/dev or set TEST_DATABASE_URL");
    let app = unicloud_portal::build_app(st.clone()).await.unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    Portal { base, st }
}

pub fn uniq(prefix: &str) -> String {
    format!("{prefix}{}", &uuid::Uuid::new_v4().simple().to_string()[..10])
}

/// A browser: one cookie, no redirect following
#[derive(Clone)]
pub struct Browser {
    pub base: String,
    http: reqwest::Client,
    cookie: Arc<Mutex<Option<String>>>,
}

pub struct Resp {
    pub status: u16,
    pub headers: HeaderMap,
    pub text: String,
}

impl Resp {
    pub fn location(&self) -> String {
        self.headers
            .get(LOCATION)
            .map(|v| v.to_str().unwrap().to_string())
            .unwrap_or_default()
    }
    pub fn json(&self) -> Value {
        serde_json::from_str(&self.text).unwrap_or(Value::Null)
    }
    pub fn query(&self, key: &str) -> Option<String> {
        let loc = self.location();
        let q = loc.split_once('?')?.1.to_string();
        url::form_urlencoded::parse(q.as_bytes())
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.into_owned())
    }
}

impl Browser {
    pub fn new(base: &str) -> Self {
        Self {
            base: base.into(),
            http: reqwest::Client::builder().redirect(Policy::none()).build().unwrap(),
            cookie: Arc::new(Mutex::new(None)),
        }
    }

    fn url(&self, path: &str) -> String {
        if path.starts_with("http") {
            path.into()
        } else {
            format!("{}{}", self.base, path)
        }
    }

    async fn send(&self, rb: reqwest::RequestBuilder) -> Resp {
        let cookie = self.cookie.lock().unwrap().clone();
        let rb = match cookie {
            Some(c) => rb.header("cookie", c),
            None => rb,
        };
        let r = rb.send().await.unwrap();
        for sc in r.headers().get_all(SET_COOKIE) {
            let s = sc.to_str().unwrap();
            let pair = s.split(';').next().unwrap().to_string();
            if s.contains("Max-Age=0") || pair.ends_with('=') {
                *self.cookie.lock().unwrap() = None;
            } else {
                *self.cookie.lock().unwrap() = Some(pair);
            }
        }
        let status = r.status().as_u16();
        let headers = r.headers().clone();
        Resp {
            status,
            headers,
            text: r.text().await.unwrap(),
        }
    }

    pub async fn get(&self, path: &str) -> Resp {
        self.send(self.http.get(self.url(path)).header("accept", "text/html")).await
    }

    pub async fn get_with(&self, path: &str, headers: &[(&str, &str)]) -> Resp {
        let mut rb = self.http.get(self.url(path));
        for (k, v) in headers {
            rb = rb.header(*k, *v);
        }
        self.send(rb).await
    }

    pub async fn post_form(&self, path: &str, form: &[(&str, &str)]) -> Resp {
        self.send(self.http.post(self.url(path)).form(form)).await
    }

    pub async fn post_json(&self, path: &str, body: &Value) -> Resp {
        self.send(self.http.post(self.url(path)).json(body)).await
    }

    pub async fn put_json(&self, path: &str, body: &Value) -> Resp {
        self.send(self.http.put(self.url(path)).json(body)).await
    }

    pub async fn token(&self, client: &str, secret: &str, form: &[(&str, &str)]) -> Resp {
        let basic = STANDARD.encode(format!("{client}:{secret}"));
        self.send(
            self.http
                .post(self.url("/oauth2/token"))
                .header("authorization", format!("Basic {basic}"))
                .form(form),
        )
        .await
    }

    pub async fn bearer_get(&self, path: &str, token: &str) -> Resp {
        self.send(self.http.get(self.url(path)).bearer_auth(token)).await
    }

    /// The session cookie's value (the session id)
    pub fn cookie_value(&self) -> Option<String> {
        self.cookie
            .lock()
            .unwrap()
            .as_ref()
            .and_then(|c| c.split_once('=').map(|(_, v)| v.to_string()))
    }

    pub fn has_cookie(&self) -> bool {
        self.cookie.lock().unwrap().is_some()
    }

    /// Signs in through the HTML form, using the CSRF token from the page
    pub async fn sign_in(&self, email: &str, password: &str) -> Resp {
        let page = self.get("/login").await;
        let csrf = csrf_from(&page.text);
        self.post_form("/login", &[("csrf", &csrf), ("email", email), ("password", password)])
            .await
    }
}

pub fn csrf_from(html: &str) -> String {
    let i = html.find(r#"name="csrf" value=""#).expect("no csrf field") + r#"name="csrf" value=""#.len();
    html[i..].split('"').next().unwrap().to_string()
}

pub fn jwt_payload(t: &str) -> Value {
    let p = t.split('.').nth(1).unwrap();
    serde_json::from_slice(&URL_SAFE_NO_PAD.decode(p).unwrap()).unwrap()
}

pub fn jwt_header(t: &str) -> Value {
    let p = t.split('.').next().unwrap();
    serde_json::from_slice(&URL_SAFE_NO_PAD.decode(p).unwrap()).unwrap()
}

pub struct Pkce {
    pub verifier: String,
    pub challenge: String,
}

pub fn pkce() -> Pkce {
    let verifier = format!("{}{}", uuid::Uuid::new_v4().simple(), uuid::Uuid::new_v4().simple());
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    Pkce { verifier, challenge }
}

pub fn authorize_url(client: &str, scope: &str, p: &Pkce, extra: &[(&str, &str)]) -> String {
    let mut q: Vec<(&str, &str)> = vec![
        ("response_type", "code"),
        ("client_id", client),
        ("redirect_uri", CB),
        ("scope", scope),
        ("state", "st-1"),
        ("nonce", "n-1"),
        ("code_challenge", &p.challenge),
        ("code_challenge_method", "S256"),
    ];
    for (k, v) in extra {
        q.retain(|(kk, _)| kk != k);
        if !v.is_empty() {
            q.push((k, v));
        }
    }
    format!("/oauth2/authorize?{}", serde_urlencoded::to_string(&q).unwrap())
}

/// Registers a module through the staging API: (client_id, secret)
pub async fn staging_client(b: &Browser, directory: bool) -> (String, String) {
    let id = uniq("m-");
    let r = b.post_json("/api/v1/staging/sso-clients", &json!({ "clientId": id, "name": "Test module", "redirectUris": [CB], "postLogoutRedirectUris": [OUT], "directory": directory })).await;
    assert_eq!(r.status, 201, "{}", r.text);
    (id, r.json()["clientSecret"].as_str().unwrap().to_string())
}

/// Creates a student through the staging API: (id, email, password)
pub async fn staging_student(b: &Browser) -> (i64, String, String) {
    let email = format!("{}@test.tu-sofia.bg", uniq("s"));
    let pw = "a-long-password".to_string();
    let r = b
        .post_json(
            "/api/v1/staging/users",
            &json!({ "email": email, "password": pw, "firstName": "Тест", "lastName": "Студент", "role": "STUDENT",
            "student": { "facultyNumber": uniq("9"), "specialty": "Компютърни науки", "studyYear": 2 } }),
        )
        .await;
    assert_eq!(r.status, 201, "{}", r.text);
    (r.json()["id"].as_i64().unwrap(), email, pw)
}

/// A full login: returns the token response JSON
pub async fn login_and_exchange(b: &Browser, client: &str, secret: &str, email: &str, pw: &str, scope: &str) -> Value {
    let p = pkce();
    let first = b.get(&authorize_url(client, scope, &p, &[])).await;
    assert_eq!(
        first.status, 302,
        "authorize without a session redirects to sign-in: {}",
        first.text
    );
    let signed = b.sign_in(email, pw).await;
    assert!(
        signed.location().starts_with("/oauth2/authorize?"),
        "sign-in resumes the login: {}",
        signed.location()
    );
    let cb = b.get(&signed.location()).await;
    let code = cb.query("code").expect("code in callback");
    assert_eq!(cb.query("state").as_deref(), Some("st-1"));
    let t = b
        .token(
            client,
            secret,
            &[
                ("grant_type", "authorization_code"),
                ("code", &code),
                ("redirect_uri", CB),
                ("code_verifier", &p.verifier),
            ],
        )
        .await;
    assert_eq!(t.status, 200, "{}", t.text);
    t.json()
}
