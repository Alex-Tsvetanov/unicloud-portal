//! `/api/v1/directory`: read-only lookups for modules, with a client-credentials token that has the
//! `directory` scope (ADR 0002). Shapes match the SSO claims.

use axum::Json;
use axum::Router;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::oidc::userinfo::{bearer, decode_access, invalid_token};
use crate::oidc::{DIRECTORY_AUDIENCE, DIRECTORY_SCOPE, claims};
use crate::state::AppState;

const PAGE: i64 = 200;
/// 200 x 10,000 rows is far beyond any faculty; it also keeps OFFSET arithmetic small
const MAX_PAGE: i64 = 10_000;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/users/{sub}", get(user))
        .route("/users/{sub}/enrollments", get(user_enrollments))
        .route("/users/{sub}/offerings", get(user_offerings))
        .route("/offerings/{id}/students", get(offering_students))
        .route("/faculties/{code}/offerings", get(faculty_offerings))
        .route("/faculties/{code}/students", get(faculty_students))
}

/// The calling module, if its token is a live client-credentials token with the `directory` scope
async fn module(st: &AppState, headers: &HeaderMap) -> Option<String> {
    let c = decode_access(st, headers, bearer(headers)?)?;
    if !c.audiences().contains(&DIRECTORY_AUDIENCE) || !c.scopes().contains(&DIRECTORY_SCOPE) {
        return None;
    }
    let jti = Uuid::parse_str(&c.jti).ok()?;
    let ok: Option<(String,)> = sqlx::query_as(
        "SELECT t.client_id FROM oauth_client_tokens t JOIN oauth_clients c ON c.client_id = t.client_id
         WHERE t.jti = $1 AND t.expires_at > now() AND c.directory_access",
    )
    .bind(jti)
    .fetch_optional(&st.db)
    .await
    .ok()?;
    ok.map(|r| r.0).filter(|id| *id == c.sub)
}

macro_rules! guard {
    ($st:expr, $h:expr) => {
        if module(&$st, &$h).await.is_none() {
            return invalid_token();
        }
    };
}

fn not_found() -> Response {
    (StatusCode::NOT_FOUND, Json(json!({ "error": "not_found" }))).into_response()
}

fn db_err(e: sqlx::Error) -> Response {
    tracing::error!(error = %e, "directory query failed");
    (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "server_error" }))).into_response()
}

fn parse_sub(sub: &str) -> Option<i64> {
    sub.parse().ok()
}

async fn user(State(st): State<AppState>, headers: HeaderMap, Path(sub): Path<String>) -> Response {
    guard!(st, headers);
    let Some(id) = parse_sub(&sub) else { return not_found() };
    match claims::load(&st.db, id).await {
        Ok(Some(u)) => {
            let mut m = u.for_scopes(&["email", "profile", "unicloud"]);
            m.remove("permissions");
            Json(Value::Object(m)).into_response()
        }
        Ok(None) => not_found(),
        Err(e) => db_err(e),
    }
}

async fn role_of(st: &AppState, id: i64) -> sqlx::Result<Option<String>> {
    Ok(sqlx::query_as::<_, (String,)>("SELECT role FROM users WHERE id = $1 AND enabled")
        .bind(id)
        .fetch_optional(&st.db)
        .await?
        .map(|r| r.0))
}

#[derive(sqlx::FromRow)]
struct OfferingRow {
    id: i64,
    code: String,
    title: String,
    semester: String,
    teacher_id: i64,
}

fn offerings_json(rows: Vec<OfferingRow>, with_teacher: bool) -> Value {
    rows.into_iter()
        .map(|o| {
            let mut v = json!({ "id": o.id, "code": o.code, "title": o.title, "semester": o.semester });
            if with_teacher {
                v["teacher_sub"] = json!(o.teacher_id.to_string());
            }
            v
        })
        .collect()
}

const OFFERING_COLS: &str =
    "o.id, o.course_code AS code, c.title, o.semester, o.teacher_id FROM course_offerings o JOIN courses c ON c.code = o.course_code";

async fn user_enrollments(State(st): State<AppState>, headers: HeaderMap, Path(sub): Path<String>) -> Response {
    guard!(st, headers);
    let Some(id) = parse_sub(&sub) else { return not_found() };
    match role_of(&st, id).await {
        Ok(Some(r)) if r == "STUDENT" => {}
        Ok(_) => return not_found(),
        Err(e) => return db_err(e),
    }
    let sql = format!(
        "SELECT {OFFERING_COLS} JOIN enrollments e ON e.offering_id = o.id WHERE e.student_id = $1 AND e.status = 'ENROLLED' ORDER BY o.semester, o.course_code"
    );
    match sqlx::query_as::<_, OfferingRow>(sqlx::AssertSqlSafe(sql))
        .bind(id)
        .fetch_all(&st.db)
        .await
    {
        Ok(rows) => Json(offerings_json(rows, false)).into_response(),
        Err(e) => db_err(e),
    }
}

async fn user_offerings(State(st): State<AppState>, headers: HeaderMap, Path(sub): Path<String>) -> Response {
    guard!(st, headers);
    let Some(id) = parse_sub(&sub) else { return not_found() };
    match role_of(&st, id).await {
        Ok(Some(r)) if r == "TEACHER" => {}
        Ok(_) => return not_found(),
        Err(e) => return db_err(e),
    }
    let sql = format!("SELECT {OFFERING_COLS} WHERE o.teacher_id = $1 ORDER BY o.semester, o.course_code");
    match sqlx::query_as::<_, OfferingRow>(sqlx::AssertSqlSafe(sql))
        .bind(id)
        .fetch_all(&st.db)
        .await
    {
        Ok(rows) => Json(offerings_json(rows, false)).into_response(),
        Err(e) => db_err(e),
    }
}

async fn offering_students(State(st): State<AppState>, headers: HeaderMap, Path(id): Path<i64>) -> Response {
    guard!(st, headers);
    let exists: Result<Option<(i64,)>, _> = sqlx::query_as("SELECT id FROM course_offerings WHERE id = $1")
        .bind(id)
        .fetch_optional(&st.db)
        .await;
    match exists {
        Ok(Some(_)) => {}
        Ok(None) => return not_found(),
        Err(e) => return db_err(e),
    }
    let rows: Result<Vec<(i64, String, String)>, _> = sqlx::query_as(
        "SELECT u.id, u.first_name || ' ' || u.last_name, r.faculty_number
         FROM enrollments e JOIN users u ON u.id = e.student_id JOIN academic_records r ON r.user_id = u.id
         WHERE e.offering_id = $1 AND e.status = 'ENROLLED' ORDER BY u.last_name, u.first_name",
    )
    .bind(id)
    .fetch_all(&st.db)
    .await;
    match rows {
        Ok(r) => Json(
            r.into_iter()
                .map(|(id, name, fnum)| json!({ "sub": id.to_string(), "name": name, "faculty_number": fnum }))
                .collect::<Value>(),
        )
        .into_response(),
        Err(e) => db_err(e),
    }
}

#[derive(Deserialize, Default)]
struct ListQuery {
    semester: Option<String>,
    page: Option<i64>,
}

async fn faculty_exists(st: &AppState, code: &str) -> sqlx::Result<bool> {
    Ok(sqlx::query_as::<_, (String,)>("SELECT code FROM faculties WHERE code = $1")
        .bind(code)
        .fetch_optional(&st.db)
        .await?
        .is_some())
}

/// A page of items as a JSON array; when more remain, a `Link: <...>; rel="next"` header
fn paged(items: Vec<Value>, page: i64, base: &str) -> Response {
    let more = items.len() as i64 > PAGE;
    let items: Vec<Value> = items.into_iter().take(PAGE as usize).collect();
    let mut res = Json(Value::Array(items)).into_response();
    if more {
        let next = format!(
            "<{base}{}page={}>; rel=\"next\"",
            if base.contains('?') { '&' } else { '?' },
            page + 1
        );
        if let Ok(v) = axum::http::HeaderValue::from_str(&next) {
            res.headers_mut().insert(axum::http::header::LINK, v);
        }
    }
    res
}

async fn faculty_offerings(
    State(st): State<AppState>,
    headers: HeaderMap,
    Path(code): Path<String>,
    Query(q): Query<ListQuery>,
) -> Response {
    guard!(st, headers);
    match faculty_exists(&st, &code).await {
        Ok(true) => {}
        Ok(false) => return not_found(),
        Err(e) => return db_err(e),
    }
    let page = q.page.unwrap_or(1).clamp(1, MAX_PAGE);
    let sql = format!(
        "SELECT {OFFERING_COLS} WHERE c.faculty_code = $1 AND ($2::text IS NULL OR o.semester = $2) ORDER BY o.id LIMIT $3 OFFSET $4"
    );
    let rows = sqlx::query_as::<_, OfferingRow>(sqlx::AssertSqlSafe(sql))
        .bind(&code)
        .bind(&q.semester)
        .bind(PAGE + 1)
        .bind((page - 1) * PAGE)
        .fetch_all(&st.db)
        .await;
    match rows {
        Ok(rows) => {
            let Value::Array(items) = offerings_json(rows, true) else {
                return not_found();
            };
            let base = match &q.semester {
                Some(s) => format!(
                    "/api/v1/directory/faculties/{code}/offerings?semester={}",
                    url::form_urlencoded::byte_serialize(s.as_bytes()).collect::<String>()
                ),
                None => format!("/api/v1/directory/faculties/{code}/offerings"),
            };
            paged(items, page, &base)
        }
        Err(e) => db_err(e),
    }
}

async fn faculty_students(
    State(st): State<AppState>,
    headers: HeaderMap,
    Path(code): Path<String>,
    Query(q): Query<ListQuery>,
) -> Response {
    guard!(st, headers);
    match faculty_exists(&st, &code).await {
        Ok(true) => {}
        Ok(false) => return not_found(),
        Err(e) => return db_err(e),
    }
    let page = q.page.unwrap_or(1).clamp(1, MAX_PAGE);
    let rows: Result<Vec<(i64, String, String, String, i16, String)>, _> = sqlx::query_as(
        "SELECT u.id, u.first_name || ' ' || u.last_name, r.faculty_number, s.name, r.study_year, r.status
         FROM academic_records r JOIN users u ON u.id = r.user_id JOIN specialties s ON s.id = r.specialty_id
         WHERE s.faculty_code = $1 AND u.enabled ORDER BY u.id LIMIT $2 OFFSET $3",
    )
    .bind(&code)
    .bind(PAGE + 1)
    .bind((page - 1) * PAGE)
    .fetch_all(&st.db)
    .await;
    match rows {
        Ok(r) => {
            let items = r
                .into_iter()
                .map(|(id, name, fnum, spec, year, status)| {
                    json!({ "sub": id.to_string(), "name": name, "faculty_number": fnum, "specialty": spec, "study_year": year, "status": status })
                })
                .collect();
            paged(items, page, &format!("/api/v1/directory/faculties/{code}/students"))
        }
        Err(e) => db_err(e),
    }
}
