//! Session, current user, CSRF and htmx helpers shared by the UI handlers.

pub mod csrf;
pub mod session;

use axum::http::HeaderMap;
use axum::response::{IntoResponse, Redirect, Response};

/// True when htmx made the request: answer with a fragment instead of a whole page
pub fn is_htmx(headers: &HeaderMap) -> bool {
    headers.get("hx-request").is_some_and(|v| v == "true") && headers.get("hx-boosted").is_none()
}

/// A redirect that also works for htmx requests
pub fn redirect(headers: &HeaderMap, to: &str) -> Response {
    if headers.get("hx-request").is_some() {
        ([("HX-Redirect", to.to_string())], "").into_response()
    } else {
        Redirect::to(to).into_response()
    }
}
