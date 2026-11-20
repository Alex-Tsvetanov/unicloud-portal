//! Staging tools for module teams: test users with any permissions, and module registration.
//! Mounted only in staging and development. In production the routes do not exist (404).

use axum::Json;
use axum::Router;
use axum::extract::{OriginalUri, Path, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, put};
use maud::{Markup, html};
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::access::{Permission, Role, mask_of, names_of};
use crate::accounts::admin::{CreateError, NewRecord, NewUser, create_user};
use crate::api_error::api_error;
use crate::oidc::clients::{RegisterError, Registration, register};
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/staging", get(page))
        .route("/staging/users-table", get(users_table))
        .route("/api/v1/staging/roles", get(roles))
        .route("/api/v1/staging/users", get(list_users).post(create))
        .route("/api/v1/staging/users/{id}/permissions", put(set_permissions))
        .route("/api/v1/staging/sso-clients", post(register_client))
}

async fn roles() -> Json<Value> {
    let defaults: serde_json::Map<String, Value> = Role::ALL
        .iter()
        .map(|r| (r.as_str().to_string(), json!(names_of(r.default_mask()))))
        .collect();
    Json(json!({ "roleDefaults": defaults, "permissions": Permission::ALL.iter().map(|p| p.as_str()).collect::<Vec<_>>() }))
}

#[derive(sqlx::FromRow)]
struct StagingUser {
    id: i64,
    email: String,
    first_name: String,
    last_name: String,
    role: String,
    faculty: Option<String>,
    permissions_mask: i64,
    student: bool,
}

impl StagingUser {
    fn json(&self) -> Value {
        json!({ "id": self.id, "email": self.email, "firstName": self.first_name, "lastName": self.last_name, "role": self.role,
                "faculty": self.faculty, "permissions": names_of(self.permissions_mask), "student": self.student })
    }
}

const STAGING_USER_SQL: &str =
    "SELECT u.id, u.email, u.first_name, u.last_name, u.role, COALESCE(u.faculty_code, s.faculty_code) AS faculty,
        u.permissions_mask, r.user_id IS NOT NULL AS student
     FROM users u LEFT JOIN academic_records r ON r.user_id = u.id LEFT JOIN specialties s ON s.id = r.specialty_id";

async fn all_users(db: &PgPool) -> sqlx::Result<Vec<StagingUser>> {
    sqlx::query_as(sqlx::AssertSqlSafe(format!("{STAGING_USER_SQL} ORDER BY u.id")))
        .fetch_all(db)
        .await
}

async fn one_user(db: &PgPool, id: i64) -> sqlx::Result<Option<StagingUser>> {
    sqlx::query_as(sqlx::AssertSqlSafe(format!("{STAGING_USER_SQL} WHERE u.id = $1")))
        .bind(id)
        .fetch_optional(db)
        .await
}

fn server_error(e: impl std::fmt::Display, path: &str) -> Response {
    tracing::error!(error = %e, "staging tools failed");
    api_error(StatusCode::INTERNAL_SERVER_ERROR, "Internal error", path, None)
}

async fn list_users(State(st): State<AppState>, OriginalUri(uri): OriginalUri) -> Response {
    match all_users(&st.db).await {
        Ok(u) => Json(u.iter().map(StagingUser::json).collect::<Value>()).into_response(),
        Err(e) => server_error(e, uri.path()),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct StudentBody {
    faculty_number: Option<String>,
    specialty: Option<String>,
    study_year: Option<i16>,
    status: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateBody {
    email: Option<String>,
    password: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    role: Option<String>,
    faculty: Option<String>,
    permissions: Option<Vec<String>>,
    student: Option<StudentBody>,
}

fn parse_permissions(list: &[String]) -> Result<i64, String> {
    let mut perms = Vec::new();
    for p in list {
        match Permission::parse(p) {
            Some(x) => perms.push(x),
            None => {
                let known: Vec<&str> = Permission::ALL.iter().map(|p| p.as_str()).collect();
                return Err(format!("permissions: unknown permission '{p}', known: {known:?}"));
            }
        }
    }
    Ok(mask_of(&perms))
}

async fn specialty_id(db: &PgPool, s: &str) -> sqlx::Result<Option<i32>> {
    let row: Option<(i32,)> =
        sqlx::query_as("SELECT id FROM specialties WHERE id::text = $1 OR lower(name) = lower($1) ORDER BY id LIMIT 1")
            .bind(s.trim())
            .fetch_optional(db)
            .await?;
    Ok(row.map(|r| r.0))
}

async fn create(State(st): State<AppState>, OriginalUri(uri): OriginalUri, Json(b): Json<CreateBody>) -> Response {
    let path = uri.path();
    let mut fields: Vec<(&str, &str)> = Vec::new();
    let (email, password, first, last) = (
        b.email.unwrap_or_default(),
        b.password.unwrap_or_default(),
        b.first_name.unwrap_or_default(),
        b.last_name.unwrap_or_default(),
    );
    if email.trim().is_empty() {
        fields.push(("email", "must not be blank"));
    }
    if !(8..=72).contains(&password.chars().count()) {
        fields.push(("password", "size must be between 8 and 72"));
    }
    if first.trim().is_empty() || first.chars().count() > 100 {
        fields.push(("firstName", "must be 1 to 100 characters"));
    }
    if last.trim().is_empty() || last.chars().count() > 100 {
        fields.push(("lastName", "must be 1 to 100 characters"));
    }
    if !fields.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, "Validation failed", path, Some(fields));
    }
    let names: Vec<&str> = Role::ALL.iter().map(|r| r.as_str()).collect();
    let Some(role) = b.role.as_deref().and_then(Role::parse) else {
        return api_error(StatusCode::BAD_REQUEST, &format!("role: must be one of {names:?}"), path, None);
    };
    let mask = match &b.permissions {
        None => role.default_mask(),
        Some(list) => match parse_permissions(list) {
            Ok(m) => m,
            Err(msg) => return api_error(StatusCode::BAD_REQUEST, &msg, path, None),
        },
    };
    let student = match (role, b.student) {
        (Role::Student, Some(s)) => {
            let status = s
                .status
                .as_deref()
                .map(str::trim)
                .filter(|x| !x.is_empty())
                .unwrap_or("ACTIVE")
                .to_ascii_uppercase();
            if !["ACTIVE", "INTERRUPTED", "GRADUATED"].contains(&status.as_str()) {
                return api_error(
                    StatusCode::BAD_REQUEST,
                    "student.status: must be one of [ACTIVE, INTERRUPTED, GRADUATED]",
                    path,
                    None,
                );
            }
            let spec = match specialty_id(&st.db, s.specialty.as_deref().unwrap_or("")).await {
                Ok(Some(id)) => id,
                Ok(None) => {
                    return api_error(
                        StatusCode::BAD_REQUEST,
                        "student.specialty: unknown specialty, see /staging for the list",
                        path,
                        None,
                    );
                }
                Err(e) => return server_error(e, path),
            };
            Some(NewRecord {
                faculty_number: s.faculty_number.unwrap_or_default(),
                specialty_id: spec,
                study_year: s.study_year.unwrap_or(0),
                status,
            })
        }
        (Role::Student, None) => {
            return api_error(
                StatusCode::BAD_REQUEST,
                "student: a STUDENT needs an academic record (facultyNumber, specialty, studyYear)",
                path,
                None,
            );
        }
        (_, Some(_)) => {
            return api_error(
                StatusCode::BAD_REQUEST,
                "student: only a user with role STUDENT has an academic record",
                path,
                None,
            );
        }
        (_, None) => None,
    };
    let new = NewUser {
        email,
        password,
        first_name: first,
        last_name: last,
        role,
        mask,
        faculty_code: b.faculty.map(|f| f.trim().to_uppercase()),
        student,
        must_change_password: false,
    };
    match create_user(&st.db, new).await {
        Ok(id) => match one_user(&st.db, id).await {
            Ok(Some(u)) => (StatusCode::CREATED, Json(u.json())).into_response(),
            Ok(None) => server_error("created user vanished", path),
            Err(e) => server_error(e, path),
        },
        Err(CreateError::Invalid(m)) => api_error(StatusCode::BAD_REQUEST, &m, path, None),
        Err(CreateError::Duplicate(m)) => api_error(StatusCode::CONFLICT, &m, path, None),
        Err(CreateError::Other(e)) => server_error(e, path),
    }
}

#[derive(Deserialize)]
struct PermsBody {
    permissions: Option<Vec<String>>,
}

async fn set_permissions(
    State(st): State<AppState>,
    OriginalUri(uri): OriginalUri,
    Path(id): Path<i64>,
    Json(b): Json<PermsBody>,
) -> Response {
    let path = uri.path();
    let Some(list) = b.permissions else {
        return api_error(
            StatusCode::BAD_REQUEST,
            "Validation failed",
            path,
            Some(vec![("permissions", "must not be null")]),
        );
    };
    let mask = match parse_permissions(&list) {
        Ok(m) => m,
        Err(msg) => return api_error(StatusCode::BAD_REQUEST, &msg, path, None),
    };
    match sqlx::query("UPDATE users SET permissions_mask = $1 WHERE id = $2")
        .bind(mask)
        .bind(id)
        .execute(&st.db)
        .await
    {
        Ok(r) if r.rows_affected() == 0 => api_error(StatusCode::NOT_FOUND, &format!("User not found with id: {id}"), path, None),
        Ok(_) => match one_user(&st.db, id).await {
            Ok(Some(u)) => Json(u.json()).into_response(),
            Ok(None) => api_error(StatusCode::NOT_FOUND, &format!("User not found with id: {id}"), path, None),
            Err(e) => server_error(e, path),
        },
        Err(e) => server_error(e, path),
    }
}

async fn register_client(State(st): State<AppState>, OriginalUri(uri): OriginalUri, Json(r): Json<Registration>) -> Response {
    let path = uri.path();
    match register(&st.db, &st.cfg, r, "staging-tools").await {
        Ok(reg) => (StatusCode::CREATED, Json(reg)).into_response(),
        Err(RegisterError::Fields(f)) => {
            let list: Vec<(&str, &str)> = f.iter().map(|(k, m)| (*k, m.as_str())).collect();
            api_error(StatusCode::BAD_REQUEST, "Validation failed", path, Some(list))
        }
        Err(RegisterError::Invalid(m)) => api_error(StatusCode::BAD_REQUEST, &m, path, None),
        Err(RegisterError::Duplicate(m)) => api_error(StatusCode::CONFLICT, &m, path, None),
        Err(RegisterError::Db(e)) => server_error(e, path),
    }
}

// ---------- Page ----------

async fn page(State(st): State<AppState>) -> Response {
    let table = users_fragment(&st.db)
        .await
        .unwrap_or_else(|_| html! { p { "Грешка при зареждане." } });
    let specialties: Vec<(i32, String, String)> =
        sqlx::query_as("SELECT id, faculty_code, name FROM specialties ORDER BY faculty_code, name")
            .fetch_all(&st.db)
            .await
            .unwrap_or_default();
    let faculties: Vec<(String, String)> = sqlx::query_as("SELECT code, name FROM faculties ORDER BY code")
        .fetch_all(&st.db)
        .await
        .unwrap_or_default();
    let mut res = crate::ui::staging_page::render(table, &specialties, &faculties).into_response();
    res.headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    res
}

async fn users_table(State(st): State<AppState>) -> Response {
    match users_fragment(&st.db).await {
        Ok(m) => m.into_response(),
        Err(e) => server_error(e, "/staging/users-table"),
    }
}

async fn users_fragment(db: &PgPool) -> sqlx::Result<Markup> {
    let users = all_users(db).await?;
    Ok(crate::ui::staging_page::users_table(
        &users
            .iter()
            .map(|u| {
                (
                    u.id,
                    u.email.clone(),
                    format!("{} {}", u.first_name, u.last_name),
                    u.role.clone(),
                    u.faculty.clone(),
                    names_of(u.permissions_mask),
                )
            })
            .collect::<Vec<_>>(),
    ))
}
