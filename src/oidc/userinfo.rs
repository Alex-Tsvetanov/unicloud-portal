//! `GET|POST /userinfo` for access tokens from a user login. Claims are read at request time.

use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use jsonwebtoken::{Algorithm, Validation};
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use super::{claims, issuer};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct AccessClaims {
    pub sub: String,
    pub jti: String,
    #[serde(default)]
    pub aud: Value,
    #[serde(default)]
    pub scope: Value,
    #[serde(default)]
    pub client_id: Option<String>,
}

impl AccessClaims {
    pub fn audiences(&self) -> Vec<&str> {
        match &self.aud {
            Value::String(s) => vec![s.as_str()],
            Value::Array(a) => a.iter().filter_map(Value::as_str).collect(),
            _ => vec![],
        }
    }
    pub fn scopes(&self) -> Vec<&str> {
        match &self.scope {
            Value::String(s) => s.split_whitespace().collect(),
            Value::Array(a) => a.iter().filter_map(Value::as_str).collect(),
            _ => vec![],
        }
    }
}

pub fn bearer(headers: &HeaderMap) -> Option<&str> {
    let v = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    v.strip_prefix("Bearer ").or_else(|| v.strip_prefix("bearer ")).map(str::trim)
}

/// Checks signature, issuer and expiry of an access token. Audience is checked by the caller.
pub fn decode_access(st: &AppState, headers: &HeaderMap, token: &str) -> Option<AccessClaims> {
    let mut v = Validation::new(Algorithm::RS256);
    v.set_issuer(&[issuer(st, headers)]);
    v.validate_aud = false;
    v.set_required_spec_claims(&["exp", "iss", "sub"]);
    v.leeway = 30;
    jsonwebtoken::decode::<AccessClaims>(token, st.keys.decoding(), &v)
        .ok()
        .map(|d| d.claims)
}

pub fn invalid_token() -> Response {
    let mut res = (StatusCode::UNAUTHORIZED, Json(json!({ "error": "invalid_token" }))).into_response();
    res.headers_mut().insert(
        header::WWW_AUTHENTICATE,
        HeaderValue::from_static(r#"Bearer error="invalid_token""#),
    );
    res
}

pub async fn userinfo(State(st): State<AppState>, headers: HeaderMap) -> Response {
    let Some(claims) = bearer(&headers).and_then(|t| decode_access(&st, &headers, t)) else {
        return invalid_token();
    };
    let Ok(jti) = Uuid::parse_str(&claims.jti) else {
        return invalid_token();
    };
    // Only access tokens of a live, unrevoked user login (an ID token or a client token has no such grant)
    let grant: Result<Option<(i64, String)>, _> = sqlx::query_as(
        "SELECT user_id, scope FROM oauth_grants WHERE access_jti = $1 AND NOT revoked AND created_at > now() - interval '1 day'",
    )
    .bind(jti)
    .fetch_optional(&st.db)
    .await;
    let Ok(Some((user_id, scope))) = grant else {
        return invalid_token();
    };
    if claims.sub != user_id.to_string() {
        return invalid_token();
    }
    match claims::load(&st.db, user_id).await {
        Ok(Some(user)) => {
            let scopes: Vec<&str> = scope.split_whitespace().collect();
            let mut res = Json(Value::Object(user.for_scopes(&scopes))).into_response();
            res.headers_mut()
                .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
            res
        }
        _ => invalid_token(),
    }
}
