//! The academic office: students and their status, courses and offerings.

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{ApiBody, ApiError, ApiResult, ApiUser, done, parse_sub};
use crate::access::{Permission, Role};
use crate::accounts::admin::{CreateError, NewRecord, NewUser, create_user};
use crate::error::is_unique_violation;
use crate::state::AppState;

const STATUSES: [&str; 3] = ["ACTIVE", "INTERRUPTED", "GRADUATED"];
const MAX_STUDENTS: i64 = 300;

#[derive(Deserialize, Default)]
pub struct StudentFilter {
    #[serde(default)]
    q: String,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct InspectStudent {
    sub: String,
    name: String,
    email: String,
    faculty_number: String,
    specialty: String,
    study_year: i16,
    status: String,
}

pub async fn students(State(st): State<AppState>, u: ApiUser, Query(f): Query<StudentFilter>) -> ApiResult<Json<Vec<InspectStudent>>> {
    if !u.user.can(Permission::StudentCreate) && !u.user.can(Permission::StudentProfileReadAny) {
        return Err(ApiError::forbidden());
    }
    let like = format!("%{}%", f.q.trim().to_lowercase());
    let rows = sqlx::query_as(
        "SELECT u.id::text AS sub, u.first_name || ' ' || u.last_name AS name, u.email, r.faculty_number,
                s.faculty_code || ' · ' || s.name AS specialty, r.study_year, r.status
         FROM academic_records r JOIN users u ON u.id = r.user_id JOIN specialties s ON s.id = r.specialty_id
         WHERE $1 = '%%' OR lower(u.first_name || ' ' || u.last_name) LIKE $1 OR lower(r.faculty_number) LIKE $1 OR lower(u.email) LIKE $1
         ORDER BY u.last_name, u.first_name LIMIT $2",
    )
    .bind(like)
    .bind(MAX_STUDENTS)
    .fetch_all(&st.db)
    .await?;
    Ok(Json(rows))
}

#[derive(Deserialize)]
pub struct NewStudent {
    email: String,
    first_name: String,
    last_name: String,
    faculty_number: String,
    specialty_id: i32,
    study_year: i16,
    password: String,
}

pub fn created(sub: i64) -> Response {
    (StatusCode::CREATED, Json(json!({ "sub": sub.to_string() }))).into_response()
}

pub fn create_error(e: CreateError) -> ApiError {
    match e {
        CreateError::Invalid(m) => ApiError::invalid(m),
        CreateError::Duplicate(m) => ApiError::conflict(m),
        CreateError::Other(e) => e.into(),
    }
}

pub async fn create_student(State(st): State<AppState>, u: ApiUser, ApiBody(f): ApiBody<NewStudent>) -> ApiResult<Response> {
    u.require(Permission::StudentCreate)?;
    let new = NewUser {
        email: f.email,
        password: f.password,
        first_name: f.first_name,
        last_name: f.last_name,
        role: Role::Student,
        mask: Role::Student.default_mask(),
        faculty_code: None,
        student: Some(NewRecord {
            faculty_number: f.faculty_number,
            specialty_id: f.specialty_id,
            study_year: f.study_year,
            status: "ACTIVE".into(),
        }),
        must_change_password: true,
    };
    let id = create_user(&st.db, new).await.map_err(create_error)?;
    tracing::info!(created = id, by = u.user.id, "student created");
    Ok(created(id))
}

#[derive(Deserialize)]
pub struct StatusChange {
    status: String,
}

pub async fn set_status(
    State(st): State<AppState>,
    u: ApiUser,
    Path(sub): Path<String>,
    ApiBody(f): ApiBody<StatusChange>,
) -> ApiResult<StatusCode> {
    u.require(Permission::StudentCreate)?;
    let id = parse_sub(&sub)?;
    if !STATUSES.contains(&f.status.as_str()) {
        return Err(ApiError::invalid("Невалиден статус."));
    }
    let r = sqlx::query("UPDATE academic_records SET status = $1 WHERE user_id = $2")
        .bind(&f.status)
        .bind(id)
        .execute(&st.db)
        .await?;
    if r.rows_affected() == 0 {
        return Err(ApiError::not_found());
    }
    tracing::info!(student = id, status = f.status, by = u.user.id, "student status changed");
    Ok(done())
}

#[derive(Serialize, sqlx::FromRow)]
struct InspectCourse {
    code: String,
    title: String,
    ects: i16,
    faculty: String,
}

#[derive(Serialize, sqlx::FromRow)]
struct InspectOffering {
    id: i64,
    code: String,
    semester: String,
    teacher: String,
    enrolled: i32,
    capacity: i32,
    is_open: bool,
}

#[derive(Serialize, sqlx::FromRow)]
struct TeacherOption {
    sub: String,
    name: String,
    faculty: Option<String>,
}

pub async fn courses(State(st): State<AppState>, u: ApiUser) -> ApiResult<Json<serde_json::Value>> {
    if !u.user.can(Permission::CourseCreateModify) && !u.user.can(Permission::CourseOfferingCreateModify) {
        return Err(ApiError::forbidden());
    }
    let courses: Vec<InspectCourse> =
        sqlx::query_as("SELECT code, title, ects_credits AS ects, faculty_code AS faculty FROM courses ORDER BY code")
            .fetch_all(&st.db)
            .await?;
    let offerings: Vec<InspectOffering> = sqlx::query_as(
        "SELECT o.id, o.course_code AS code, o.semester, t.first_name || ' ' || t.last_name AS teacher,
                o.enrolled_count AS enrolled, o.capacity, o.is_open
         FROM course_offerings o JOIN users t ON t.id = o.teacher_id ORDER BY o.semester DESC, o.course_code",
    )
    .fetch_all(&st.db)
    .await?;
    let teachers: Vec<TeacherOption> = sqlx::query_as(
        "SELECT id::text AS sub, last_name || ' ' || first_name AS name, faculty_code AS faculty
         FROM users WHERE role = 'TEACHER' AND enabled ORDER BY last_name, first_name",
    )
    .fetch_all(&st.db)
    .await?;
    Ok(Json(json!({ "courses": courses, "offerings": offerings, "teachers": teachers })))
}

#[derive(Deserialize)]
pub struct NewCourse {
    code: String,
    title: String,
    ects_credits: i16,
    faculty_code: String,
}

pub async fn create_course(State(st): State<AppState>, u: ApiUser, ApiBody(f): ApiBody<NewCourse>) -> ApiResult<StatusCode> {
    u.require(Permission::CourseCreateModify)?;
    let code = f.code.trim().to_uppercase();
    if code.is_empty() || code.len() > 16 || !code.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err(ApiError::invalid("Кодът трябва да е до 16 латински букви, цифри или тирета."));
    }
    if f.title.trim().is_empty() || !(1..=30).contains(&f.ects_credits) {
        return Err(ApiError::invalid("Попълнете наименование и ECTS от 1 до 30."));
    }
    let r = sqlx::query("INSERT INTO courses (code, title, ects_credits, faculty_code) VALUES ($1, $2, $3, $4)")
        .bind(&code)
        .bind(f.title.trim())
        .bind(f.ects_credits)
        .bind(&f.faculty_code)
        .execute(&st.db)
        .await;
    match r {
        Ok(_) => Ok(StatusCode::CREATED),
        Err(e) if is_unique_violation(&e) => Err(ApiError::conflict("Дисциплина с този код вече съществува.")),
        Err(e) if is_foreign_key_violation(&e) => Err(ApiError::invalid("Няма такъв факултет.")),
        Err(e) => Err(e.into()),
    }
}

fn is_foreign_key_violation(e: &sqlx::Error) -> bool {
    matches!(e, sqlx::Error::Database(d) if d.code().as_deref() == Some("23503"))
}

#[derive(Deserialize)]
pub struct NewOffering {
    course_code: String,
    semester: String,
    teacher_sub: String,
    capacity: i32,
}

pub async fn create_offering(State(st): State<AppState>, u: ApiUser, ApiBody(f): ApiBody<NewOffering>) -> ApiResult<StatusCode> {
    u.require(Permission::CourseOfferingCreateModify)?;
    let semester = f.semester.trim();
    if f.capacity < 1 || semester.is_empty() || semester.len() > 16 {
        return Err(ApiError::invalid("Местата трябва да са поне 1, а семестърът да е попълнен."));
    }
    let teacher = f
        .teacher_sub
        .parse::<i64>()
        .map_err(|_| ApiError::invalid("Няма такъв преподавател."))?;
    // The teacher must belong to the course's faculty
    let ok: Option<(bool,)> = sqlx::query_as(
        "SELECT t.faculty_code = c.faculty_code FROM users t, courses c WHERE t.id = $1 AND t.role = 'TEACHER' AND c.code = $2",
    )
    .bind(teacher)
    .bind(&f.course_code)
    .fetch_optional(&st.db)
    .await?;
    if ok.map(|r| r.0) != Some(true) {
        return Err(ApiError::invalid("Преподавателят трябва да е от факултета на дисциплината."));
    }
    let r = sqlx::query("INSERT INTO course_offerings (course_code, semester, teacher_id, capacity) VALUES ($1, $2, $3, $4)")
        .bind(&f.course_code)
        .bind(semester)
        .bind(teacher)
        .bind(f.capacity)
        .execute(&st.db)
        .await;
    match r {
        Ok(_) => Ok(StatusCode::CREATED),
        Err(e) if is_unique_violation(&e) => Err(ApiError::conflict("Този курс вече съществува за семестъра.")),
        Err(e) => Err(e.into()),
    }
}

pub async fn toggle_offering(State(st): State<AppState>, u: ApiUser, Path(id): Path<i64>) -> ApiResult<Json<serde_json::Value>> {
    u.require(Permission::CourseOfferingCreateModify)?;
    let (open,): (bool,) = sqlx::query_as("UPDATE course_offerings SET is_open = NOT is_open WHERE id = $1 RETURNING is_open")
        .bind(id)
        .fetch_optional(&st.db)
        .await?
        .ok_or_else(ApiError::not_found)?;
    Ok(Json(json!({ "enabled": open })))
}
