//! `POST /oauth2/token`: authorization codes (user logins) and client credentials (the directory API).
//! Clients authenticate with HTTP Basic only. Errors follow RFC 6749 section 5.2.

use axum::Form;
use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use super::clients::{self, Client};
use super::{DIRECTORY_AUDIENCE, DIRECTORY_SCOPE, TOKEN_TTL_SECS, claims, issuer};
use crate::crypto::{pkce_s256_matches, sha256_hex};
use crate::state::AppState;
use crate::web::session::now;

#[derive(Deserialize, Debug, Default)]
pub struct TokenForm {
    grant_type: Option<String>,
    code: Option<String>,
    redirect_uri: Option<String>,
    code_verifier: Option<String>,
    scope: Option<String>,
}

pub fn oauth_error(status: StatusCode, error: &str, description: &str) -> Response {
    let mut res = (status, Json(json!({ "error": error, "error_description": description }))).into_response();
    no_store(&mut res);
    if status == StatusCode::UNAUTHORIZED {
        res.headers_mut().insert(
            header::WWW_AUTHENTICATE,
            HeaderValue::from_static(r#"Basic realm="unicloud-portal""#),
        );
    }
    res
}

fn no_store(res: &mut Response) {
    res.headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    res.headers_mut().insert(header::PRAGMA, HeaderValue::from_static("no-cache"));
}

fn invalid_grant(desc: &str) -> Response {
    oauth_error(StatusCode::BAD_REQUEST, "invalid_grant", desc)
}

/// `Authorization: Basic base64(urlencode(client_id) ":" urlencode(secret))` (RFC 6749 section 2.3.1)
fn basic_credentials(headers: &HeaderMap) -> Option<(String, String)> {
    let v = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let b64 = v.strip_prefix("Basic ").or_else(|| v.strip_prefix("basic "))?;
    let decoded = String::from_utf8(STANDARD.decode(b64.trim()).ok()?).ok()?;
    let (id, secret) = decoded.split_once(':')?;
    let dec = |s: &str| {
        url::form_urlencoded::parse(format!("x={s}").as_bytes())
            .next()
            .map(|(_, v)| v.into_owned())
    };
    Some((dec(id)?, dec(secret)?))
}

async fn authenticate(st: &AppState, headers: &HeaderMap) -> Result<Client, Response> {
    let bad = || oauth_error(StatusCode::UNAUTHORIZED, "invalid_client", "Client authentication failed");
    let (id, secret) = basic_credentials(headers).ok_or_else(bad)?;
    let client = clients::find(&st.db, &id).await.map_err(|e| {
        tracing::error!(error = %e, "client lookup failed");
        oauth_error(StatusCode::INTERNAL_SERVER_ERROR, "server_error", "Try again")
    })?;
    match client {
        Some(c) if c.secret_matches(&secret) => Ok(c),
        _ => Err(bad()),
    }
}

pub async fn token(State(st): State<AppState>, headers: HeaderMap, Form(f): Form<TokenForm>) -> Response {
    let client = match authenticate(&st, &headers).await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let iss = issuer(&st, &headers);
    let result = match f.grant_type.as_deref() {
        Some("authorization_code") => authorization_code(&st, &client, &iss, &f).await,
        Some("client_credentials") => client_credentials(&st, &client, &iss, &f).await,
        _ => Ok(oauth_error(
            StatusCode::BAD_REQUEST,
            "unsupported_grant_type",
            "OAuth 2.0 Parameter: grant_type",
        )),
    };
    result.unwrap_or_else(|e| {
        tracing::error!(error = %e, "token endpoint failed");
        oauth_error(StatusCode::INTERNAL_SERVER_ERROR, "server_error", "Try again")
    })
}

#[derive(sqlx::FromRow)]
struct CodeRow {
    grant_id: Uuid,
    redirect_uri: String,
    nonce: Option<String>,
    code_challenge: String,
    auth_time: i64,
    expired: bool,
    redeemed: bool,
    user_id: i64,
    scope: String,
    session_id: String,
}

async fn authorization_code(st: &AppState, client: &Client, iss: &str, f: &TokenForm) -> anyhow::Result<Response> {
    let (Some(code), Some(verifier)) = (f.code.as_deref(), f.code_verifier.as_deref()) else {
        return Ok(oauth_error(
            StatusCode::BAD_REQUEST,
            "invalid_request",
            "code and code_verifier are required",
        ));
    };
    let hash = sha256_hex(code);
    let row: Option<CodeRow> = sqlx::query_as(
        "SELECT c.grant_id, c.redirect_uri, c.nonce, c.code_challenge, c.auth_time,
                c.expires_at <= now() AS expired, c.redeemed_at IS NOT NULL AS redeemed,
                g.user_id, g.scope, g.session_id
         FROM oauth_codes c JOIN oauth_grants g ON g.id = c.grant_id
         WHERE c.code_hash = $1 AND g.client_id = $2",
    )
    .bind(&hash)
    .bind(&client.client_id)
    .fetch_optional(&st.db)
    .await?;
    let Some(row) = row else {
        return Ok(invalid_grant("The authorization code is invalid"));
    };

    // Single use across replicas: only one exchange can flip `redeemed_at`
    let won = sqlx::query("UPDATE oauth_codes SET redeemed_at = now() WHERE code_hash = $1 AND redeemed_at IS NULL AND expires_at > now()")
        .bind(&hash)
        .execute(&st.db)
        .await?
        .rows_affected()
        == 1;
    if !won {
        if row.redeemed || !row.expired {
            // Someone else exchanged it: one of the two callers is not the module. Revoke what was issued.
            sqlx::query("UPDATE oauth_grants SET revoked = TRUE WHERE id = $1")
                .bind(row.grant_id)
                .execute(&st.db)
                .await?;
            tracing::warn!(client_id = client.client_id, "authorization code replayed; tokens revoked");
            return Ok(invalid_grant("The authorization code has already been used"));
        }
        return Ok(invalid_grant("The authorization code has expired"));
    }
    if f.redirect_uri.as_deref() != Some(row.redirect_uri.as_str()) {
        return Ok(invalid_grant("redirect_uri does not match the authorization request"));
    }
    if !pkce_s256_matches(verifier, &row.code_challenge) {
        return Ok(invalid_grant("code_verifier does not match the code_challenge"));
    }
    let Some(user) = claims::load(&st.db, row.user_id).await? else {
        return Ok(invalid_grant("The user no longer exists or is disabled"));
    };

    let scopes: Vec<&str> = row.scope.split_whitespace().collect();
    let (iat, exp) = (now(), now() + TOKEN_TTL_SECS);
    let (access_jti, id_jti) = (Uuid::new_v4(), Uuid::new_v4());
    let access = json!({
        "iss": iss, "sub": user.id.to_string(), "aud": [client.client_id], "client_id": client.client_id,
        "scope": scopes, "iat": iat, "nbf": iat, "exp": exp, "jti": access_jti.to_string(),
    });
    let mut id = user.for_scopes(&scopes);
    id.insert("iss".into(), json!(iss));
    id.insert("aud".into(), json!([client.client_id]));
    id.insert("azp".into(), json!(client.client_id));
    id.insert("iat".into(), json!(iat));
    id.insert("exp".into(), json!(exp));
    id.insert("auth_time".into(), json!(row.auth_time));
    id.insert("sid".into(), json!(row.session_id));
    id.insert("jti".into(), json!(id_jti.to_string()));
    if let Some(n) = &row.nonce {
        id.insert("nonce".into(), json!(n));
    }
    let access_token = st.keys.sign(&access)?;
    let id_token = st.keys.sign(&Value::Object(id))?;
    sqlx::query("UPDATE oauth_grants SET access_jti = $1, id_token_jti = $2 WHERE id = $3")
        .bind(access_jti)
        .bind(id_jti)
        .bind(row.grant_id)
        .execute(&st.db)
        .await?;
    let mut res = Json(json!({
        "access_token": access_token,
        "id_token": id_token,
        "scope": row.scope,
        "token_type": "Bearer",
        "expires_in": TOKEN_TTL_SECS,
    }))
    .into_response();
    no_store(&mut res);
    Ok(res)
}

async fn client_credentials(st: &AppState, client: &Client, iss: &str, f: &TokenForm) -> anyhow::Result<Response> {
    let requested: Vec<&str> = f.scope.as_deref().unwrap_or(DIRECTORY_SCOPE).split_whitespace().collect();
    if !client.directory_access || requested.iter().any(|s| *s != DIRECTORY_SCOPE) {
        return Ok(oauth_error(StatusCode::BAD_REQUEST, "invalid_scope", "OAuth 2.0 Parameter: scope"));
    }
    let (iat, exp, jti) = (now(), now() + TOKEN_TTL_SECS, Uuid::new_v4());
    let claims = json!({
        "iss": iss, "sub": client.client_id, "aud": [DIRECTORY_AUDIENCE], "client_id": client.client_id,
        "scope": [DIRECTORY_SCOPE], "iat": iat, "nbf": iat, "exp": exp, "jti": jti.to_string(),
    });
    let token = st.keys.sign(&claims)?;
    sqlx::query("INSERT INTO oauth_client_tokens (jti, client_id, expires_at) VALUES ($1, $2, to_timestamp($3))")
        .bind(jti)
        .bind(&client.client_id)
        .bind(exp as f64)
        .execute(&st.db)
        .await?;
    let mut res = Json(json!({ "access_token": token, "scope": DIRECTORY_SCOPE, "token_type": "Bearer", "expires_in": TOKEN_TTL_SECS }))
        .into_response();
    no_store(&mut res);
    Ok(res)
}
