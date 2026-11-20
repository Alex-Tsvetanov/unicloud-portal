//! Black-box SSO conformance: the behaviour module teams rely on (docs/sso.md, docs/sso-wire-contract.md,
//! openspec sso-provider and module-directory). Ported from the Java SsoFlowTests and SsoAnyCallbackTests.

mod common;

use common::*;
use serde_json::json;
use unicloud_portal::config::Environment;

#[tokio::test]
async fn discovery_and_jwks_advertise_only_what_is_served() {
    let p = start(Environment::Staging, &[]).await;
    let b = Browser::new(&p.base);
    let d = b.get_with("/.well-known/openid-configuration", &[]).await.json();
    assert_eq!(d["issuer"], p.base);
    for (k, path) in [
        ("authorization_endpoint", "/oauth2/authorize"),
        ("token_endpoint", "/oauth2/token"),
        ("jwks_uri", "/oauth2/jwks"),
        ("userinfo_endpoint", "/userinfo"),
        ("end_session_endpoint", "/connect/logout"),
    ] {
        assert_eq!(d[k], format!("{}{path}", p.base), "{k}");
    }
    assert_eq!(d["grant_types_supported"], json!(["authorization_code", "client_credentials"]));
    assert_eq!(d["token_endpoint_auth_methods_supported"], json!(["client_secret_basic"]));
    assert_eq!(d["code_challenge_methods_supported"], json!(["S256"]));
    assert_eq!(
        d["scopes_supported"],
        json!(["openid", "profile", "email", "unicloud", "directory"])
    );
    assert!(d.get("revocation_endpoint").is_none() && d.get("introspection_endpoint").is_none());

    let jwks = b.get_with("/oauth2/jwks", &[]).await.json();
    let key = &jwks["keys"][0];
    assert_eq!(key["kty"], "RSA");
    for private in ["d", "p", "q", "dp", "dq", "qi"] {
        assert!(key.get(private).is_none(), "{private} must never be published");
    }
    assert_eq!(key["kid"], p.st.keys.kid);
}

#[tokio::test]
async fn full_login_flow_tokens_userinfo_and_replay() {
    let p = start(Environment::Staging, &[]).await;
    let b = Browser::new(&p.base);
    let (client, secret) = staging_client(&b, false).await;
    let (uid, email, pw) = staging_student(&b).await;
    let t = login_and_exchange(&b, &client, &secret, &email, &pw, "openid profile email unicloud").await;

    assert_eq!(t["token_type"], "Bearer");
    assert!(t.get("refresh_token").is_none());
    assert!((1700..=1800).contains(&t["expires_in"].as_i64().unwrap()));
    let id = jwt_payload(t["id_token"].as_str().unwrap());
    assert_eq!(jwt_header(t["id_token"].as_str().unwrap())["alg"], "RS256");
    assert_eq!(jwt_header(t["id_token"].as_str().unwrap())["kid"], p.st.keys.kid);
    assert_eq!(id["sub"], uid.to_string(), "sub is a JSON string");
    assert_eq!(id["aud"], client, "aud is a plain string, as the Java portal sent");
    assert_eq!(id["azp"], client);
    assert_eq!(id["iss"], p.base);
    assert_eq!(id["nonce"], "n-1");
    assert!(id["auth_time"].is_i64());
    assert_eq!(id["exp"].as_i64().unwrap() - id["iat"].as_i64().unwrap(), 1800);
    assert_eq!(id["role"], "STUDENT");
    assert_eq!(
        id["permissions"],
        json!(["STUDENT_PROFILE_READ_OWN", "COURSE_OFFERING_READ", "ENROLLMENT_REQUEST_CREATE"])
    );
    assert_eq!(id["student"]["study_year"], 2);
    assert_eq!(id["student"]["status"], "ACTIVE");
    assert_eq!(id["faculty"]["code"], "FCST");
    assert_eq!(id["name"], "Тест Студент");

    let access = t["access_token"].as_str().unwrap();
    assert_eq!(jwt_payload(access)["client_id"], client);
    assert_eq!(jwt_header(access)["typ"], "at+jwt", "access tokens say what they are (RFC 9068)");
    assert!(jwt_header(t["id_token"].as_str().unwrap()).get("typ").is_none());
    let ui = b.bearer_get("/userinfo", access).await;
    assert_eq!(ui.status, 200, "{}", ui.text);
    assert_eq!(ui.json()["sub"], uid.to_string());
    assert_eq!(ui.json()["family_name"], "Студент");

    // An ID token is not an access token
    assert_eq!(b.bearer_get("/userinfo", t["id_token"].as_str().unwrap()).await.status, 401);

    // A replayed code fails and revokes what the first exchange issued.
    // (Re-run the login to get a code, exchange it twice.)
    let pk = pkce();
    let cb = b.get(&authorize_url(&client, "openid", &pk, &[])).await;
    assert_eq!(cb.status, 302, "a live session skips the password");
    let code = cb.query("code").unwrap();
    let form = [
        ("grant_type", "authorization_code"),
        ("code", code.as_str()),
        ("redirect_uri", CB),
        ("code_verifier", pk.verifier.as_str()),
    ];
    let first = b.token(&client, &secret, &form).await;
    assert_eq!(first.status, 200);
    let second = b.token(&client, &secret, &form).await;
    assert_eq!(second.status, 400);
    assert_eq!(second.json()["error"], "invalid_grant");
    let revoked = b.bearer_get("/userinfo", first.json()["access_token"].as_str().unwrap()).await;
    assert_eq!(revoked.status, 401);
    assert_eq!(revoked.json()["error"], "invalid_token");
}

#[tokio::test]
async fn scopes_limit_the_claims() {
    let p = start(Environment::Staging, &[]).await;
    let b = Browser::new(&p.base);
    let (client, secret) = staging_client(&b, false).await;
    let (_, email, pw) = staging_student(&b).await;
    let t = login_and_exchange(&b, &client, &secret, &email, &pw, "openid email").await;
    let id = jwt_payload(t["id_token"].as_str().unwrap());
    assert!(id.get("email").is_some());
    for absent in ["family_name", "role", "faculty", "student", "permissions"] {
        assert!(id.get(absent).is_none(), "{absent} without its scope");
    }
}

#[tokio::test]
async fn untrusted_clients_and_callbacks_get_a_plain_400() {
    let p = start(Environment::Production, &["library.tu-sofia.bg"]).await;
    let b = Browser::new(&p.base);
    let cfg = &p.st.cfg;
    let reg = unicloud_portal::oidc::clients::Registration {
        client_id: Some(uniq("lib-")),
        name: Some("Library".into()),
        redirect_uris: Some(vec!["https://library.tu-sofia.bg/auth/callback".into()]),
        post_logout_redirect_uris: None,
        directory: false,
    };
    let r = unicloud_portal::oidc::clients::register(&p.st.db, cfg, reg, "test").await.unwrap();
    let pk = pkce();
    let base_q = |cb: &str| {
        format!(
            "/oauth2/authorize?response_type=code&client_id={}&redirect_uri={}&scope=openid&code_challenge={}&code_challenge_method=S256",
            r.client_id,
            urlenc(cb),
            pk.challenge
        )
    };

    for cb in [
        "https://library.tu-sofia.bg/auth/callback/../admin",
        "https://evil.example/cb",
        "javascript:alert(1)",
    ] {
        let res = b.get(&base_q(cb)).await;
        assert_eq!(res.status, 400, "{cb}");
        assert!(res.location().is_empty(), "no redirect for {cb}");
    }
    let unknown = b
        .get(&format!(
            "/oauth2/authorize?response_type=code&client_id=nope-x&redirect_uri={}&scope=openid",
            urlenc(CB)
        ))
        .await;
    assert_eq!(unknown.status, 400);

    // Registration in production follows the host list
    let bad = unicloud_portal::oidc::clients::Registration {
        client_id: Some(uniq("x-")),
        name: Some("x".into()),
        redirect_uris: Some(vec!["https://evil.example/cb".into()]),
        post_logout_redirect_uris: None,
        directory: false,
    };
    assert!(unicloud_portal::oidc::clients::register(&p.st.db, cfg, bad, "test").await.is_err());

    // The trusted callback without a session goes to sign-in
    let ok = b.get(&base_q("https://library.tu-sofia.bg/auth/callback")).await;
    assert_eq!(ok.status, 302);
    assert_eq!(ok.location(), "/login?sso=1");
}

fn urlenc(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}

#[tokio::test]
async fn request_errors_go_back_to_a_trusted_callback() {
    let p = start(Environment::Staging, &[]).await;
    let b = Browser::new(&p.base);
    let (client, _) = staging_client(&b, false).await;
    let pk = pkce();
    let cases = [
        (vec![("code_challenge", "")], "invalid_request"),
        (vec![("code_challenge_method", "plain")], "invalid_request"),
        (vec![("scope", "openid admin")], "invalid_scope"),
        (vec![("scope", "profile")], "invalid_request"),
        (vec![("response_type", "token")], "unsupported_response_type"),
    ];
    for (extra, err) in cases {
        let r = b.get(&authorize_url(&client, "openid", &pk, &extra)).await;
        assert_eq!(r.status, 302, "{extra:?}");
        assert!(
            r.location().starts_with(&format!("{CB}?error={err}")),
            "{extra:?} -> {}",
            r.location()
        );
        assert_eq!(r.query("state").as_deref(), Some("st-1"));
    }
}

#[tokio::test]
async fn token_endpoint_errors() {
    let p = start(Environment::Staging, &[]).await;
    let b = Browser::new(&p.base);
    let (client, secret) = staging_client(&b, false).await;
    let (_, email, pw) = staging_student(&b).await;
    b.sign_in(&email, &pw).await;
    let pk = pkce();
    let code = b.get(&authorize_url(&client, "openid", &pk, &[])).await.query("code").unwrap();

    let wrong_secret = b
        .token(&client, "wrong", &[("grant_type", "authorization_code"), ("code", &code)])
        .await;
    assert_eq!(
        (wrong_secret.status, wrong_secret.json()["error"].as_str()),
        (401, Some("invalid_client"))
    );

    let http = reqwest::Client::new();
    let post = http
        .post(format!("{}/oauth2/token", p.base))
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", code.as_str()),
            ("client_id", client.as_str()),
            ("client_secret", secret.as_str()),
        ])
        .send()
        .await
        .unwrap();
    assert_eq!(post.status().as_u16(), 401, "client_secret_post is not accepted");

    let bad_verifier = b
        .token(
            &client,
            &secret,
            &[
                ("grant_type", "authorization_code"),
                ("code", &code),
                ("redirect_uri", CB),
                ("code_verifier", &"x".repeat(50)),
            ],
        )
        .await;
    assert_eq!(
        (bad_verifier.status, bad_verifier.json()["error"].as_str()),
        (400, Some("invalid_grant"))
    );

    let grant = b.token(&client, &secret, &[("grant_type", "password")]).await;
    assert_eq!(grant.json()["error"], "unsupported_grant_type");

    let cc = b
        .token(&client, &secret, &[("grant_type", "client_credentials"), ("scope", "directory")])
        .await;
    assert_eq!(
        (cc.status, cc.json()["error"].as_str()),
        (400, Some("invalid_scope")),
        "no directory grant"
    );
}

#[tokio::test]
async fn disabled_user_gets_no_tokens() {
    let p = start(Environment::Staging, &[]).await;
    let b = Browser::new(&p.base);
    let (client, secret) = staging_client(&b, false).await;
    let (uid, email, pw) = staging_student(&b).await;
    b.sign_in(&email, &pw).await;
    let pk = pkce();
    let code = b.get(&authorize_url(&client, "openid", &pk, &[])).await.query("code").unwrap();
    sqlx::query("UPDATE users SET enabled = FALSE WHERE id = $1")
        .bind(uid)
        .execute(&p.st.db)
        .await
        .unwrap();
    let t = b
        .token(
            &client,
            &secret,
            &[
                ("grant_type", "authorization_code"),
                ("code", &code),
                ("redirect_uri", CB),
                ("code_verifier", &pk.verifier),
            ],
        )
        .await;
    assert_eq!(t.status, 400);
    assert!(t.json().get("id_token").is_none());
}

#[tokio::test]
async fn logout_everywhere_ends_the_session() {
    let p = start(Environment::Staging, &[]).await;
    let b = Browser::new(&p.base);
    let (client, secret) = staging_client(&b, false).await;
    let (_, email, pw) = staging_student(&b).await;
    let t = login_and_exchange(&b, &client, &secret, &email, &pw, "openid").await;
    // Without an ID token hint, a link from any website must not log the user out: the portal asks
    let ask = b.get("/connect/logout").await;
    assert_eq!(ask.status, 200);
    assert!(ask.text.contains("action=\"/logout\""));
    assert_eq!(b.get(&authorize_url(&client, "openid", &pkce(), &[])).await.status, 302);
    assert!(
        b.get(&authorize_url(&client, "openid", &pkce(), &[]))
            .await
            .location()
            .starts_with(CB),
        "still signed in"
    );
    let q = serde_urlencoded::to_string([
        ("id_token_hint", t["id_token"].as_str().unwrap()),
        ("client_id", &client),
        ("post_logout_redirect_uri", OUT),
        ("state", "bye"),
    ])
    .unwrap();
    let out = b.get(&format!("/connect/logout?{q}")).await;
    assert_eq!(out.status, 302);
    assert_eq!(out.location(), format!("{OUT}?state=bye"));
    let again = b.get(&authorize_url(&client, "openid", &pkce(), &[])).await;
    assert_eq!(again.location(), "/login?sso=1", "the SSO session is gone");

    let bad = b
        .get(&format!("/connect/logout?client_id={client}&post_logout_redirect_uri=javascript:x"))
        .await;
    assert_eq!(bad.status, 400);
}

#[tokio::test]
async fn directory_api_with_client_credentials() {
    let p = start(Environment::Staging, &[]).await;
    let b = Browser::new(&p.base);
    let (client, secret) = staging_client(&b, true).await;
    let (uid, _, _) = staging_student(&b).await;

    assert_eq!(
        b.get_with(&format!("/api/v1/directory/users/{uid}"), &[]).await.status,
        401,
        "no token"
    );
    let t = b
        .token(&client, &secret, &[("grant_type", "client_credentials"), ("scope", "directory")])
        .await;
    assert_eq!(t.status, 200, "{}", t.text);
    let token = t.json()["access_token"].as_str().unwrap().to_string();
    assert!(t.json().get("id_token").is_none());
    assert_eq!(jwt_payload(&token)["sub"], client);

    let u = b.bearer_get(&format!("/api/v1/directory/users/{uid}"), &token).await;
    assert_eq!(u.status, 200, "{}", u.text);
    assert_eq!(u.json()["role"], "STUDENT");
    assert_eq!(u.json()["faculty"]["code"], "FCST");
    assert!(u.json()["student"]["faculty_number"].is_string());

    assert_eq!(
        b.bearer_get(&format!("/api/v1/directory/users/{uid}/enrollments"), &token)
            .await
            .json(),
        json!([])
    );
    assert_eq!(
        b.bearer_get(&format!("/api/v1/directory/users/{uid}/offerings"), &token)
            .await
            .status,
        404,
        "not a teacher"
    );
    assert_eq!(b.bearer_get("/api/v1/directory/users/999999999", &token).await.status, 404);
    // Walk the pages (the shared test database may hold more than 200 students)
    let mut url = "/api/v1/directory/faculties/FCST/students".to_string();
    let mut found = false;
    for _ in 0..100 {
        let page = b.bearer_get(&url, &token).await;
        assert_eq!(page.status, 200);
        found |= page
            .json()
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["sub"].as_str() == Some(uid.to_string().as_str()));
        match page
            .headers
            .get("link")
            .and_then(|l| l.to_str().ok())
            .and_then(|l| l.split('>').next())
            .map(|l| l.trim_start_matches('<').to_string())
        {
            Some(next) if !found => url = next,
            _ => break,
        }
    }
    assert!(found, "the new student is in the faculty list");
    assert_eq!(b.bearer_get("/api/v1/directory/faculties/NOPE/students", &token).await.status, 404);

    // A client-credentials token is not a user login
    assert_eq!(b.bearer_get("/userinfo", &token).await.status, 401);
}

#[tokio::test]
async fn concurrent_exchanges_of_one_code_let_at_most_one_win() {
    let p = start(Environment::Staging, &[]).await;
    let b = Browser::new(&p.base);
    let (client, secret) = staging_client(&b, false).await;
    let (_, email, pw) = staging_student(&b).await;
    b.sign_in(&email, &pw).await;
    let pk = pkce();
    let code = b.get(&authorize_url(&client, "openid", &pk, &[])).await.query("code").unwrap();
    let form = vec![
        ("grant_type".to_string(), "authorization_code".to_string()),
        ("code".to_string(), code),
        ("redirect_uri".to_string(), CB.to_string()),
        ("code_verifier".to_string(), pk.verifier.clone()),
    ];
    let calls: Vec<_> = (0..8)
        .map(|_| {
            let (b, c, s, f) = (b.clone(), client.clone(), secret.clone(), form.clone());
            tokio::spawn(async move {
                let f: Vec<(&str, &str)> = f.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
                b.token(&c, &s, &f).await
            })
        })
        .collect();
    let mut winners = Vec::new();
    for c in calls {
        let r = c.await.unwrap();
        match r.status {
            200 => winners.push(r.json()["access_token"].as_str().unwrap().to_string()),
            400 => assert_eq!(r.json()["error"], "invalid_grant"),
            other => panic!("unexpected {other}: {}", r.text),
        }
    }
    assert!(winners.len() <= 1, "at most one exchange wins");
    for t in winners {
        assert_eq!(
            b.bearer_get("/userinfo", &t).await.status,
            401,
            "the winner's token is revoked by the replay"
        );
    }
}
