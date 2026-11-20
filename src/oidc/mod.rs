//! The OpenID Connect provider for UniCloud modules (docs/sso.md, docs/sso-wire-contract.md).
//! Hand-built on `jsonwebtoken`: see ADR 0001 for why no library.

pub mod authorize;
pub mod claims;
pub mod clients;
pub mod discovery;
pub mod keys;
pub mod logout;
pub mod policy;
pub mod token;
pub mod userinfo;

use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};

use crate::state::AppState;

/// Scopes a module may request in a user login
pub const USER_SCOPES: [&str; 4] = ["openid", "profile", "email", "unicloud"];
/// The scope of client-credentials tokens for the directory API
pub const DIRECTORY_SCOPE: &str = "directory";
/// The audience of client-credentials tokens
pub const DIRECTORY_AUDIENCE: &str = "unicloud-directory";
/// Lifetime of access and ID tokens
pub const TOKEN_TTL_SECS: i64 = 30 * 60;
/// Lifetime of an authorization code
pub const CODE_TTL_SECS: i64 = 60;

/// `iss`: the configured address, or the request's own scheme and host (development only)
pub fn issuer(st: &AppState, headers: &HeaderMap) -> String {
    if let Some(i) = &st.cfg.issuer {
        return i.clone();
    }
    let h = |k: &str| {
        headers
            .get(k)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
    };
    let proto = h("x-forwarded-proto")
        .filter(|p| p == "https" || p == "http")
        .unwrap_or_else(|| "http".into());
    let host = h("x-forwarded-host").or_else(|| h("host")).unwrap_or_else(|| "localhost".into());
    format!("{proto}://{host}")
}

/// Appends query parameters to a callback URL without normalising what the module registered
pub fn with_query(base: &str, params: &[(&str, &str)]) -> String {
    if params.is_empty() {
        return base.to_string();
    }
    let q = serde_urlencoded::to_string(params).unwrap_or_default();
    let sep = if base.contains('?') { '&' } else { '?' };
    format!("{base}{sep}{q}")
}

/// 302 Found, as the Java portal and docs/sso.md describe (axum's `Redirect::to` is 303)
pub fn found(location: &str) -> Response {
    let mut res = StatusCode::FOUND.into_response();
    if let Ok(v) = HeaderValue::from_str(location) {
        res.headers_mut().insert(header::LOCATION, v);
    }
    res.headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    res
}
