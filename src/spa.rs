//! Hosting the React app at `/app` and the shared component library at `/ui/v1` (ADR 0004).

use std::path::PathBuf;

use axum::Json;
use axum::Router;
use axum::extract::{Request, State};
use axum::http::{HeaderMap, HeaderValue, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Redirect, Response};
use serde_json::json;
use tower_http::services::{ServeDir, ServeFile};
use tower_sessions::Session;

use crate::error::AppResult;
use crate::state::AppState;
use crate::web::session::session_user;

/// Where the app starts after sign-in
pub const APP_HOME: &str = "/app/";

/// `/`: JSON `{service, version, replica}` for machines; the app (or sign-in) for browsers
pub async fn root(State(st): State<AppState>, headers: HeaderMap, session: Session) -> AppResult<Response> {
    let wants_json = headers
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|a| a.contains("application/json") && !a.contains("text/html"));
    if wants_json {
        return Ok(Json(json!({ "service": "unicloud-portal", "version": st.cfg.git_sha, "replica": st.cfg.replica })).into_response());
    }
    if session_user(&st, &session).await?.is_none() {
        return Ok(Redirect::to("/login").into_response());
    }
    Ok(Redirect::to(APP_HOME).into_response())
}

/// `/app` and everything under it. Unknown paths get `index.html`, so the app's router decides.
pub fn app_routes() -> Router<AppState> {
    let dir = web_dir();
    let index = dir.join("index.html");
    let files = ServeDir::new(&dir).fallback(ServeFile::new(index));
    // Built assets never fall back to index.html: a missing one is a 404, not HTML cached for a year
    let assets = ServeDir::new(dir.join("assets"));
    Router::new()
        .nest_service("/app/assets", assets)
        .nest_service("/app", files)
        .layer(middleware::from_fn(app_cache))
}

/// Built files carry a content hash in their name and never change; `index.html` must be checked every time
async fn app_cache(req: Request, next: Next) -> Response {
    let hashed = req.uri().path().starts_with("/app/assets/");
    let mut res = next.run(req).await;
    let value = if hashed && res.status().is_success() {
        "public, max-age=31536000, immutable"
    } else {
        "no-cache"
    };
    res.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static(value));
    res
}

/// `/ui/v1/`: the `@unicloud/ui` package tarball and its stylesheet, for the other teams' builds and pages
pub fn library_routes() -> Router<AppState> {
    Router::new()
        .nest_service("/ui/v1", ServeDir::new(ui_dir()))
        .layer(middleware::from_fn(library_headers))
}

async fn library_headers(req: Request, next: Next) -> Response {
    let mut res = next.run(req).await;
    let h = res.headers_mut();
    // Public, read-only files: any module's page or build may fetch them
    h.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));
    h.insert(header::CACHE_CONTROL, HeaderValue::from_static("public, max-age=3600"));
    h.insert("cross-origin-resource-policy", HeaderValue::from_static("cross-origin"));
    res
}

fn dir_from(var: &str, default: &str) -> PathBuf {
    std::env::var(var)
        .map(Into::into)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(default))
}

fn web_dir() -> PathBuf {
    dir_from("APP_WEB_DIR", "web/apps/portal/dist")
}

fn ui_dir() -> PathBuf {
    dir_from("APP_UI_DIR", "web/dist-ui")
}
