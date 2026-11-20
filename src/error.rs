//! Errors that handlers return. Pages get a Bulgarian HTML page; JSON APIs use their own shapes.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::ui::components::error_page;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("not found")]
    NotFound,
    #[error("forbidden")]
    Forbidden,
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type AppResult<T> = Result<T, AppError>;

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, msg) = match &self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "Страницата не е намерена.".to_string()),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "Нямате права за това действие.".to_string()),
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, m.clone()),
            AppError::Conflict(m) => (StatusCode::CONFLICT, m.clone()),
            AppError::Db(e) => {
                tracing::error!(error = %e, "database error");
                (StatusCode::INTERNAL_SERVER_ERROR, "Възникна грешка. Опитайте отново.".to_string())
            }
            AppError::Other(e) => {
                tracing::error!(error = %e, "unexpected error");
                (StatusCode::INTERNAL_SERVER_ERROR, "Възникна грешка. Опитайте отново.".to_string())
            }
        };
        (status, error_page(status, &msg)).into_response()
    }
}

/// True when a database error is a unique-constraint violation
pub fn is_unique_violation(e: &sqlx::Error) -> bool {
    matches!(e, sqlx::Error::Database(d) if d.code().as_deref() == Some("23505"))
}
