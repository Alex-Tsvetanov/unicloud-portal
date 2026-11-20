//! CSRF protection for the cookie-authenticated UI (ADR 0003).
//! Every non-GET request must carry the session's token, as the `X-CSRF-Token` header (htmx)
//! or as a `csrf` form field (plain forms). The JSON API accepts the header only.

use axum::Json;
use axum::body::{Body, to_bytes};
use axum::extract::Request;
use axum::http::{HeaderValue, Method, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use tower_sessions::Session;

use crate::crypto::ct_eq;
use crate::web::session::KEY_CSRF;

const MAX_FORM_BYTES: usize = 64 * 1024;

const EXPIRED: &str = "Формулярът е изтекъл. Презаредете страницата и опитайте отново.";

/// Server-rendered pages: the token as a header or a `csrf` form field; failures get an HTML page
pub async fn require_csrf(session: Session, req: Request, next: Next) -> Response {
    match check(&session, req, true).await {
        Ok(req) => next.run(req).await,
        Err(status) => (status, crate::ui::components::error_page(status, EXPIRED)).into_response(),
    }
}

/// The JSON API: the token as the `X-CSRF-Token` header only; failures get a PortalError
pub async fn require_csrf_api(session: Session, req: Request, next: Next) -> Response {
    match check(&session, req, false).await {
        Ok(req) => next.run(req).await,
        Err(status) => {
            let mut res = (status, Json(json!({ "error": "forbidden", "message": EXPIRED }))).into_response();
            res.headers_mut()
                .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
            res
        }
    }
}

/// Lets safe methods through; otherwise the request goes on only with the session's token
async fn check(session: &Session, req: Request, allow_form_field: bool) -> Result<Request, StatusCode> {
    if matches!(*req.method(), Method::GET | Method::HEAD | Method::OPTIONS) {
        return Ok(req);
    }
    let expected: Option<String> = session.get(KEY_CSRF).await.ok().flatten();
    let header = req.headers().get("x-csrf-token").and_then(|v| v.to_str().ok()).map(str::to_owned);
    let (parts, body) = req.into_parts();
    let bytes = to_bytes(body, MAX_FORM_BYTES).await.map_err(|_| StatusCode::PAYLOAD_TOO_LARGE)?;
    let field = if allow_form_field { form_field(&bytes, "csrf") } else { None };
    let given = header.or(field);
    if !matches!((&expected, &given), (Some(e), Some(g)) if ct_eq(e, g)) {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(Request::from_parts(parts, Body::from(bytes)))
}

fn form_field(bytes: &[u8], name: &str) -> Option<String> {
    url::form_urlencoded::parse(bytes)
        .find(|(k, _)| k == name)
        .map(|(_, v)| v.into_owned())
}
