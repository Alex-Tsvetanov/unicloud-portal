//! `/api/v1/portal`: the JSON API behind the React app (ADR 0004). Same site only: the session cookie
//! signs the user in and every change carries `X-CSRF-Token` (checked by the CSRF middleware).
//! Every handler checks a permission; the app's navigation is only a convenience.

mod admin;
mod catalog;
mod inspect;
mod me;
mod teach;

use axum::Json;
use axum::Router;
use axum::extract::{FromRef, FromRequestParts};
use axum::http::StatusCode;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, put};
use serde_json::json;
use tower_sessions::Session;

use crate::access::Permission;
use crate::accounts::model::User;
use crate::state::AppState;
use crate::web::session::{csrf_token, session_user};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/me", get(me::me))
        .route("/options", get(me::options))
        .route("/profile", get(me::profile))
        .route("/students/{sub}", get(me::student))
        .route("/courses", get(catalog::courses))
        .route("/courses/{id}/enroll", post(catalog::enroll))
        .route("/enrollments", get(catalog::my_enrollments))
        .route("/enrollments/{id}/cancel", post(catalog::cancel))
        .route("/teach/offerings", get(teach::offerings))
        .route("/teach/offerings/{id}", get(teach::roster))
        .route("/teach/enrollments/{id}/reject", post(teach::reject))
        .route("/faculty", get(teach::faculty))
        .route("/inspect/students", get(inspect::students).post(inspect::create_student))
        .route("/inspect/students/{sub}/status", put(inspect::set_status))
        .route("/inspect/courses", get(inspect::courses).post(inspect::create_course))
        .route("/inspect/offerings", post(inspect::create_offering))
        .route("/inspect/offerings/{id}/toggle", post(inspect::toggle_offering))
        .route("/admin/staff", get(admin::staff).post(admin::create_staff))
        .route("/admin/staff/{sub}/toggle", post(admin::toggle_staff))
        .route("/admin/modules", get(admin::modules).post(admin::register_module))
}

/// `{error, message}` with a Bulgarian message, as the app shows it
#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: String,
}

pub type ApiResult<T> = Result<T, ApiError>;

impl ApiError {
    pub fn new(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
        }
    }
    pub fn unauthenticated() -> Self {
        Self::new(StatusCode::UNAUTHORIZED, "unauthenticated", "Влезте отново.")
    }
    pub fn forbidden() -> Self {
        Self::new(StatusCode::FORBIDDEN, "forbidden", "Нямате права за това действие.")
    }
    pub fn not_found() -> Self {
        Self::new(StatusCode::NOT_FOUND, "not_found", "Не е намерено.")
    }
    pub fn invalid(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, "invalid", message)
    }
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(StatusCode::CONFLICT, "conflict", message)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let mut res = (self.status, Json(json!({ "error": self.code, "message": self.message }))).into_response();
        res.headers_mut()
            .insert(axum::http::header::CACHE_CONTROL, axum::http::HeaderValue::from_static("no-store"));
        res
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        tracing::error!(error = %e, "database error");
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "server_error",
            "Възникна грешка. Опитайте отново.",
        )
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(e: anyhow::Error) -> Self {
        tracing::error!(error = %e, "unexpected error");
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "server_error",
            "Възникна грешка. Опитайте отново.",
        )
    }
}

/// The signed-in user for an API call. Answers JSON (never a redirect) when there is none.
pub struct ApiUser {
    pub user: User,
    pub csrf: String,
}

impl ApiUser {
    pub fn require(&self, p: Permission) -> ApiResult<()> {
        if self.user.can(p) { Ok(()) } else { Err(ApiError::forbidden()) }
    }
}

impl<S> FromRequestParts<S> for ApiUser
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let st = AppState::from_ref(state);
        let session = Session::from_request_parts(parts, state)
            .await
            .map_err(|_| ApiError::unauthenticated())?;
        let Some((user, _)) = session_user(&st, &session).await? else {
            return Err(ApiError::unauthenticated());
        };
        if user.must_change_password {
            return Err(ApiError::new(
                StatusCode::FORBIDDEN,
                "password_change_required",
                "Сменете първата си парола.",
            ));
        }
        let csrf = csrf_token(&session).await?;
        Ok(ApiUser { user, csrf })
    }
}

/// A JSON request body; a malformed one is a PortalError `invalid`, not axum's plain-text answer
pub struct ApiBody<T>(pub T);

impl<S, T> axum::extract::FromRequest<S> for ApiBody<T>
where
    T: serde::de::DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(req: axum::extract::Request, state: &S) -> Result<Self, Self::Rejection> {
        match Json::<T>::from_request(req, state).await {
            Ok(Json(v)) => Ok(ApiBody(v)),
            Err(e) => {
                tracing::debug!(error = %e, "bad request body");
                Err(ApiError::invalid("Невалидни данни. Попълнете всички полета."))
            }
        }
    }
}

/// The app's navigation (paths under `/app`). The server checks permissions again on every call.
pub fn nav_links(u: &User) -> Vec<(&'static str, &'static str)> {
    let mut links = vec![("/home", "Начало")];
    if u.can(Permission::StudentProfileReadOwn) {
        links.push(("/profile", "Профил"));
    }
    if u.can(Permission::CourseOfferingRead) {
        links.push(("/courses", "Курсове"));
    }
    if u.can(Permission::EnrollmentRequestCreate) {
        links.push(("/enrollments", "Моите записвания"));
    }
    if u.can(Permission::EnrollmentRequestAcceptReject) {
        links.push(("/teach", "Моите курсове"));
    }
    if u.can(Permission::StudentProfileReadFaculty) {
        links.push(("/faculty", "Факултет"));
    }
    if u.can(Permission::StudentCreate) || u.can(Permission::StudentProfileReadAny) {
        links.push(("/inspect/students", "Студенти"));
    }
    if u.can(Permission::CourseCreateModify) || u.can(Permission::CourseOfferingCreateModify) {
        links.push(("/inspect/courses", "Дисциплини"));
    }
    if u.can(Permission::StaffCreate) {
        links.push(("/admin/staff", "Служители"));
    }
    if u.role == crate::access::Role::Admin {
        links.push(("/admin/modules", "Модули"));
    }
    links
}

/// No-content success
pub fn done() -> StatusCode {
    StatusCode::NO_CONTENT
}

/// `sub` path values are user ids
pub fn parse_sub(sub: &str) -> ApiResult<i64> {
    sub.parse().map_err(|_| ApiError::not_found())
}
