//! The Rust portal against the recording of the live Java staging (tests/golden): every token,
//! claim and error a module saw from Java must still come, with the same JSON type.

mod common;

use common::*;
use serde_json::Value;
use unicloud_portal::config::Environment;

fn golden() -> Value {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden/java-staging-2026-11-20.json")).unwrap();
    serde_json::from_str::<Value>(&text).unwrap()["steps"].clone()
}

fn kind(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Every member of `java` is in `rust`, with the same JSON type (recursing into objects)
fn same_shape(what: &str, java: &Value, rust: &Value) {
    let (Value::Object(j), Value::Object(r)) = (java, rust) else {
        panic!("{what}: not objects")
    };
    for (k, jv) in j {
        let rv = r.get(k).unwrap_or_else(|| panic!("{what}: `{k}` is missing (Java sent it)"));
        assert_eq!(kind(jv), kind(rv), "{what}: `{k}` changed type");
        if jv.is_object() {
            same_shape(&format!("{what}.{k}"), jv, rv);
        }
    }
}

#[tokio::test]
async fn tokens_claims_and_userinfo_keep_the_java_shapes() {
    let g = golden();
    let p = start(Environment::Staging, &[]).await;
    let b = Browser::new(&p.base);
    let (client, secret) = staging_client(&b, false).await;
    let (_, email, pw) = staging_student(&b).await;
    let t = login_and_exchange(&b, &client, &secret, &email, &pw, "openid profile email unicloud").await;

    same_shape("token response", &g["token"]["json"], &t);
    same_shape(
        "ID token header",
        &g["token"]["id_token"]["header"],
        &jwt_header(t["id_token"].as_str().unwrap()),
    );
    same_shape(
        "ID token",
        &g["token"]["id_token"]["payload"],
        &jwt_payload(t["id_token"].as_str().unwrap()),
    );
    same_shape(
        "access token",
        &g["token"]["access_token_decoded"]["payload"],
        &jwt_payload(t["access_token"].as_str().unwrap()),
    );
    let ui = b.bearer_get("/userinfo", t["access_token"].as_str().unwrap()).await;
    same_shape("userinfo", &g["userinfo"]["json"], &ui.json());

    let jwks = b.get_with("/oauth2/jwks", &[]).await.json();
    same_shape("JWKS key", &g["jwks"]["json"]["keys"][0], &jwks["keys"][0]);
}

#[tokio::test]
async fn errors_keep_the_java_codes() {
    let g = golden();
    let p = start(Environment::Staging, &[]).await;
    let b = Browser::new(&p.base);
    let (client, secret) = staging_client(&b, false).await;
    let (_, email, pw) = staging_student(&b).await;
    b.sign_in(&email, &pw).await;

    let code = |s: &str| {
        (
            g[s]["status"].as_u64().unwrap() as u16,
            g[s]["json"]["error"].as_str().unwrap().to_string(),
        )
    };
    let pk = pkce();
    let c = b.get(&authorize_url(&client, "openid", &pk, &[])).await.query("code").unwrap();
    let ok = [
        ("grant_type", "authorization_code"),
        ("code", c.as_str()),
        ("redirect_uri", CB),
        ("code_verifier", pk.verifier.as_str()),
    ];
    assert_eq!(b.token(&client, &secret, &ok).await.status, 200);
    let replay = b.token(&client, &secret, &ok).await;
    assert_eq!(
        (replay.status, replay.json()["error"].as_str().unwrap().to_string()),
        code("token_replay")
    );
    let bad_secret = b.token(&client, "wrong", &ok).await;
    assert_eq!(
        (bad_secret.status, bad_secret.json()["error"].as_str().unwrap().to_string()),
        code("token_bad_secret")
    );
    let grant = b.token(&client, &secret, &[("grant_type", "password")]).await;
    assert_eq!(
        (grant.status, grant.json()["error"].as_str().unwrap().to_string()),
        code("token_unsupported_grant")
    );
    let ui = b.bearer_get("/userinfo", "not-a-token").await;
    assert_eq!(
        (ui.status, ui.json()["error"].as_str().unwrap().to_string()),
        code("userinfo_no_token")
    );

    for (step, extra) in [
        ("authorize_no_pkce", vec![("code_challenge", "")]),
        ("authorize_bad_scope", vec![("scope", "openid admin")]),
        ("authorize_plain_pkce", vec![("code_challenge_method", "plain")]),
    ] {
        let java = g[step]["headers"]["Location"].as_str().unwrap();
        let java_err = java.split("error=").nth(1).unwrap().split('&').next().unwrap();
        let r = b.get(&authorize_url(&client, "openid", &pkce(), &extra)).await;
        assert_eq!(r.status, 302, "{step}");
        assert_eq!(r.query("error").as_deref(), Some(java_err), "{step}");
    }
}

#[tokio::test]
async fn reading_a_page_moves_the_idle_timeout() {
    let p = start(Environment::Staging, &[]).await;
    let b = Browser::new(&p.base);
    let (_, email, pw) = staging_student(&b).await;
    b.sign_in(&email, &pw).await;
    let id = b.cookie_value().unwrap();
    let expiry = || async {
        let (t,): (time::OffsetDateTime,) = sqlx::query_as("SELECT expiry_date FROM tower_sessions.session WHERE id = $1")
            .bind(&id)
            .fetch_one(&p.st.db)
            .await
            .unwrap();
        t
    };
    let before = expiry().await;
    tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
    assert_eq!(b.me().await.status, 200);
    assert!(expiry().await > before, "a read-only API call extends the session");
}
