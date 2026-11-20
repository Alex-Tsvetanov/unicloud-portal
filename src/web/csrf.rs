//! CSRF protection for the cookie-authenticated UI (ADR 0003).
//! Every non-GET request must carry the session's token, as the `X-CSRF-Token` header (htmx)
//! or as a `csrf` form field (plain forms). Applied to the UI router only.

use axum::body::{Body, to_bytes};
use axum::extract::Request;
use axum::http::{Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use tower_sessions::Session;

use crate::crypto::ct_eq;
use crate::web::session::KEY_CSRF;

const MAX_FORM_BYTES: usize = 64 * 1024;

pub async fn require_csrf(session: Session, req: Request, next: Next) -> Response {
    if matches!(*req.method(), Method::GET | Method::HEAD | Method::OPTIONS) {
        return next.run(req).await;
    }
    let expected: Option<String> = session.get(KEY_CSRF).await.ok().flatten();
    let header = req.headers().get("x-csrf-token").and_then(|v| v.to_str().ok()).map(str::to_owned);
    let (parts, body) = req.into_parts();
    let Ok(bytes) = to_bytes(body, MAX_FORM_BYTES).await else {
        return (StatusCode::PAYLOAD_TOO_LARGE, "Заявката е твърде голяма.").into_response();
    };
    let field = form_field(&bytes, "csrf");
    let given = header.or(field);
    let ok = matches!((&expected, &given), (Some(e), Some(g)) if ct_eq(e, g));
    if !ok {
        let page = crate::ui::components::error_page(
            StatusCode::FORBIDDEN,
            "Формулярът е изтекъл. Презаредете страницата и опитайте отново.",
        );
        return (StatusCode::FORBIDDEN, page).into_response();
    }
    next.run(Request::from_parts(parts, Body::from(bytes))).await
}

fn form_field(bytes: &[u8], name: &str) -> Option<String> {
    url::form_urlencoded::parse(bytes)
        .find(|(k, _)| k == name)
        .map(|(_, v)| v.into_owned())
}
