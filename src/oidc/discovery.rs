//! `/.well-known/openid-configuration` and `/oauth2/jwks`. Advertises only what the portal serves.

use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use serde_json::{Value, json};

use super::{DIRECTORY_SCOPE, USER_SCOPES, issuer};
use crate::state::AppState;

pub async fn discovery(State(st): State<AppState>, headers: HeaderMap) -> Json<Value> {
    let iss = issuer(&st, &headers);
    let mut scopes: Vec<&str> = USER_SCOPES.to_vec();
    scopes.push(DIRECTORY_SCOPE);
    Json(json!({
        "issuer": iss,
        "authorization_endpoint": format!("{iss}/oauth2/authorize"),
        "token_endpoint": format!("{iss}/oauth2/token"),
        "jwks_uri": format!("{iss}/oauth2/jwks"),
        "userinfo_endpoint": format!("{iss}/userinfo"),
        "end_session_endpoint": format!("{iss}/connect/logout"),
        "response_types_supported": ["code"],
        "response_modes_supported": ["query"],
        "grant_types_supported": ["authorization_code", "client_credentials"],
        "code_challenge_methods_supported": ["S256"],
        "subject_types_supported": ["public"],
        "id_token_signing_alg_values_supported": ["RS256"],
        "token_endpoint_auth_methods_supported": ["client_secret_basic"],
        "scopes_supported": scopes,
        "claims_supported": ["iss", "sub", "aud", "azp", "exp", "iat", "auth_time", "nonce", "sid", "email",
            "given_name", "family_name", "name", "role", "permissions", "faculty", "student"],
    }))
}

pub async fn jwks(State(st): State<AppState>) -> Json<Value> {
    Json(st.keys.jwks())
}
