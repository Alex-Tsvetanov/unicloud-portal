//! The JSON error body of the portal's own APIs (staging tools), same shape as the Java portal:
//! `{timestamp, status, error, message, path, fieldErrors}`.

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

pub fn api_error(status: StatusCode, message: &str, path: &str, field_errors: Option<Vec<(&str, &str)>>) -> Response {
    let fe: Value = match field_errors {
        Some(list) => list.into_iter().map(|(f, m)| json!({ "field": f, "message": m })).collect(),
        None => Value::Null,
    };
    let body = json!({
        "timestamp": OffsetDateTime::now_utc().format(&Rfc3339).unwrap_or_default(),
        "status": status.as_u16(),
        "error": status.canonical_reason().unwrap_or(""),
        "message": message,
        "path": path,
        "fieldErrors": fe,
    });
    (status, Json(body)).into_response()
}
