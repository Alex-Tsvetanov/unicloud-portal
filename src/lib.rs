//! UniCloud Portal. Vocabulary: GLOSSARY.md. Design: openspec/changes/rust-rewrite/design.md

// Query rows are plain tuples, and handlers return axum `Response` as their error type.
#![allow(clippy::type_complexity, clippy::result_large_err)]

pub mod access;
pub mod accounts;
pub mod api_error;
pub mod config;
pub mod crypto;
pub mod directory;
pub mod enrollment;
pub mod error;
pub mod oidc;
pub mod ops;
pub mod seed;
pub mod staging;
pub mod state;
pub mod ui;
pub mod web;

use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::middleware;
use axum::routing::{get, post};
use sqlx::Connection;
use sqlx::postgres::PgPoolOptions;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tower_sessions::cookie::SameSite;
use tower_sessions::cookie::time::Duration as CookieDuration;
use tower_sessions::{Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::PostgresStore;

use crate::config::Config;
use crate::oidc::keys::SigningKeys;
use crate::state::AppState;

/// Advisory lock key for start-up migrations
const STARTUP_LOCK: i64 = 0x0055_4e49_434c_4f55;

/// Connects, migrates and seeds. Called once at startup (and by the tests).
pub async fn build_state(cfg: Config) -> anyhow::Result<AppState> {
    let schema = cfg.db_schema.clone();
    // Replicas (and test binaries) may start together: run every start-up DDL step under one lock
    let mut lock = sqlx::postgres::PgConnection::connect(&cfg.database_url).await?;
    sqlx::query("SELECT pg_advisory_lock($1)")
        .bind(STARTUP_LOCK)
        .execute(&mut lock)
        .await?;
    if schema != "public" {
        sqlx::query(sqlx::AssertSqlSafe(format!("CREATE SCHEMA IF NOT EXISTS {schema}")))
            .execute(&mut lock)
            .await?;
    }
    let db = PgPoolOptions::new()
        .max_connections(cfg.db_pool_max)
        .acquire_timeout(Duration::from_secs(10))
        .after_connect(move |conn, _| {
            // The schema name is validated in Config (lower-case letters and underscores only)
            let set = format!("SET search_path TO {schema}");
            Box::pin(async move {
                sqlx::query(sqlx::AssertSqlSafe(set)).execute(conn).await?;
                Ok(())
            })
        })
        .connect(&cfg.database_url)
        .await?;
    sqlx::migrate!("./migrations").run(&db).await?;
    PostgresStore::new(db.clone()).migrate().await?;
    sqlx::query("SELECT pg_advisory_unlock($1)")
        .bind(STARTUP_LOCK)
        .execute(&mut lock)
        .await?;
    let keys = SigningKeys::from_pem(&cfg.signing_key_pem)?;
    let st = AppState {
        db,
        cfg: Arc::new(cfg),
        keys: Arc::new(keys),
    };
    seed::run(&st).await?;
    Ok(st)
}

pub async fn build_app(st: AppState) -> anyhow::Result<Router> {
    let store = PostgresStore::new(st.db.clone());
    let sessions = SessionManagerLayer::new(store)
        .with_name(st.cfg.cookie_name())
        .with_secure(st.cfg.secure_cookie)
        .with_http_only(true)
        .with_same_site(SameSite::Lax)
        .with_path("/")
        .with_expiry(Expiry::OnInactivity(CookieDuration::minutes(30)))
        // Every request moves the idle timeout ("30 minutes without use"), not only requests that write
        .with_always_save(true);

    // Pages: cookie session plus CSRF on every state change
    let ui = ui_routes().layer(middleware::from_fn(web::csrf::require_csrf));

    // Machine endpoints: no CSRF (they use client authentication or Bearer tokens)
    let machine = Router::new()
        .route("/health", get(ops::health))
        .route("/.well-known/openid-configuration", get(oidc::discovery::discovery))
        .route("/oauth2/jwks", get(oidc::discovery::jwks))
        .route("/oauth2/authorize", get(oidc::authorize::authorize))
        .route("/oauth2/token", post(oidc::token::token))
        .route("/userinfo", get(oidc::userinfo::userinfo).post(oidc::userinfo::userinfo))
        .route("/connect/logout", get(oidc::logout::logout))
        .nest("/api/v1/directory", directory::routes());

    let mut app = Router::new().merge(ui).merge(machine);
    if st.cfg.staging_tools() {
        app = app.merge(staging::routes());
    }
    Ok(app
        .nest_service("/assets", ServeDir::new(assets_dir()))
        .layer(sessions)
        .layer(middleware::from_fn_with_state(st.clone(), ops::served_by))
        .layer(middleware::from_fn(ops::security_headers))
        .layer(TraceLayer::new_for_http())
        .with_state(st))
}

fn ui_routes() -> Router<AppState> {
    use accounts::{admin, handlers as acc};
    Router::new()
        .route("/", get(ui::pages::home))
        .route("/home", get(ui::pages::dashboard))
        .route("/login", get(acc::login_page).post(acc::login_submit))
        .route("/logout", post(acc::logout))
        .route("/password", get(acc::password_page).post(acc::password_submit))
        .route("/profile", get(ui::pages::profile))
        .route("/students/{id}", get(ui::teach::student_profile))
        .route("/courses", get(ui::catalog::courses))
        .route("/courses/{id}/enroll", post(ui::catalog::enroll))
        .route("/enrollments", get(ui::catalog::my_enrollments))
        .route("/enrollments/{id}/cancel", post(ui::catalog::cancel))
        .route("/teach", get(ui::teach::teach))
        .route("/teach/offerings/{id}", get(ui::teach::offering_students))
        .route("/teach/enrollments/{id}/reject", post(ui::teach::reject))
        .route("/faculty", get(ui::teach::faculty))
        .route("/inspect/students", get(ui::inspect::students).post(ui::inspect::create_student))
        .route("/inspect/students/{id}/status", post(ui::inspect::set_status))
        .route("/inspect/courses", get(ui::inspect::courses).post(ui::inspect::create_course))
        .route("/inspect/offerings", post(ui::inspect::create_offering))
        .route("/inspect/offerings/{id}/toggle", post(ui::inspect::toggle_offering))
        .route("/admin/users", get(admin::staff_page).post(admin::staff_create))
        .route("/admin/users/{id}/toggle", post(admin::staff_toggle))
        .route(
            "/admin/modules",
            get(oidc::clients::modules_page).post(oidc::clients::modules_create),
        )
}

fn assets_dir() -> std::path::PathBuf {
    std::env::var("APP_ASSETS_DIR")
        .map(Into::into)
        .unwrap_or_else(|_| concat!(env!("CARGO_MANIFEST_DIR"), "/assets").into())
}
