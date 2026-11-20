//! `/health`, `/version`, the `X-Served-By` header, and the hourly cleanup.

use axum::Json;
use axum::extract::{Request, State};
use axum::http::{HeaderName, HeaderValue};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde_json::json;

use crate::state::AppState;

/// Never touches the database: the deploy waits for this to show the new commit
pub async fn health(State(st): State<AppState>) -> impl IntoResponse {
    no_store(Json(json!({ "status": "ok", "version": st.cfg.git_sha })))
}

pub async fn version(State(st): State<AppState>) -> impl IntoResponse {
    no_store(Json(
        json!({ "service": "unicloud-portal", "version": st.cfg.git_sha, "replica": st.cfg.replica }),
    ))
}

fn no_store(r: impl IntoResponse) -> Response {
    let mut res = r.into_response();
    res.headers_mut().insert(
        axum::http::header::CACHE_CONTROL,
        HeaderValue::from_static("no-cache, no-store, max-age=0, must-revalidate"),
    );
    res
}

pub async fn served_by(State(st): State<AppState>, req: Request, next: Next) -> Response {
    let mut res = next.run(req).await;
    if let Ok(v) = HeaderValue::from_str(&st.cfg.replica) {
        res.headers_mut().insert(HeaderName::from_static("x-served-by"), v);
    }
    res
}

/// Deletes logins older than a day and expired codes and tokens, every hour
pub fn spawn_cleanup(st: AppState) {
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(std::time::Duration::from_secs(3600));
        loop {
            tick.tick().await;
            let r = sqlx::query(
                "WITH a AS (DELETE FROM oauth_grants WHERE created_at < now() - interval '1 day'),
                      b AS (DELETE FROM oauth_client_tokens WHERE expires_at < now())
                 DELETE FROM oauth_codes WHERE expires_at < now() - interval '1 hour'",
            )
            .execute(&st.db)
            .await;
            if let Err(e) = r {
                tracing::warn!(error = %e, "cleanup failed");
            }
        }
    });
}
