//! A teacher's offerings, rosters and rejection; the dean's faculty overview.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use serde::Serialize;
use serde_json::{Value, json};

use super::{ApiError, ApiResult, ApiUser};
use crate::access::Permission;
use crate::enrollment;
use crate::state::AppState;

#[derive(Serialize, sqlx::FromRow)]
pub struct TeachOffering {
    id: i64,
    code: String,
    title: String,
    semester: String,
    enrolled: i32,
    capacity: i32,
    is_open: bool,
}

const TEACH_SELECT: &str = "SELECT o.id, c.code, c.title, o.semester, o.enrolled_count AS enrolled, o.capacity, o.is_open
     FROM course_offerings o JOIN courses c ON c.code = o.course_code WHERE o.teacher_id = $1";

pub async fn offerings(State(st): State<AppState>, u: ApiUser) -> ApiResult<Json<Vec<TeachOffering>>> {
    u.require(Permission::EnrollmentRequestAcceptReject)?;
    let sql = format!("{TEACH_SELECT} ORDER BY o.semester DESC, c.code");
    let rows = sqlx::query_as(sqlx::AssertSqlSafe(sql)).bind(u.user.id).fetch_all(&st.db).await?;
    Ok(Json(rows))
}

#[derive(Serialize, sqlx::FromRow)]
pub struct RosterEntry {
    enrollment_id: i64,
    student_sub: String,
    name: String,
    faculty_number: String,
    status: String,
}

#[derive(Serialize)]
pub struct OfferingRoster {
    offering: TeachOffering,
    roster: Vec<RosterEntry>,
}

/// Only the offering's own teacher sees its roster; anyone else gets 404
pub async fn roster(State(st): State<AppState>, u: ApiUser, Path(id): Path<i64>) -> ApiResult<Json<OfferingRoster>> {
    u.require(Permission::EnrollmentRequestAcceptReject)?;
    let sql = format!("{TEACH_SELECT} AND o.id = $2");
    let offering: TeachOffering = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(u.user.id)
        .bind(id)
        .fetch_optional(&st.db)
        .await?
        .ok_or_else(ApiError::not_found)?;
    let roster = sqlx::query_as(
        "SELECT e.id AS enrollment_id, u.id::text AS student_sub, u.first_name || ' ' || u.last_name AS name, r.faculty_number, e.status
         FROM enrollments e JOIN users u ON u.id = e.student_id JOIN academic_records r ON r.user_id = u.id
         WHERE e.offering_id = $1 ORDER BY e.status, u.last_name",
    )
    .bind(id)
    .fetch_all(&st.db)
    .await?;
    Ok(Json(OfferingRoster { offering, roster }))
}

pub async fn reject(State(st): State<AppState>, u: ApiUser, Path(id): Path<i64>) -> ApiResult<StatusCode> {
    u.require(Permission::EnrollmentRequestAcceptReject)?;
    if !enrollment::reject(&st.db, u.user.id, id).await? {
        return Err(ApiError::conflict("Записването не може да бъде отхвърлено."));
    }
    tracing::info!(teacher = u.user.id, enrollment = id, "enrollment rejected");
    Ok(StatusCode::OK)
}

#[derive(Serialize, sqlx::FromRow)]
struct FacultyOffering {
    code: String,
    title: String,
    teacher: String,
    enrolled: i32,
    capacity: i32,
}

#[derive(Serialize, sqlx::FromRow)]
struct FacultyStudent {
    sub: String,
    name: String,
    faculty_number: String,
    specialty: String,
    study_year: i16,
    status: String,
}

pub async fn faculty(State(st): State<AppState>, u: ApiUser) -> ApiResult<Json<Value>> {
    u.require(Permission::StudentProfileReadFaculty)?;
    let code = u.user.faculty_code.clone().ok_or_else(ApiError::not_found)?;
    let (name,): (String,) = sqlx::query_as("SELECT name FROM faculties WHERE code = $1")
        .bind(&code)
        .fetch_optional(&st.db)
        .await?
        .ok_or_else(ApiError::not_found)?;
    let offerings: Vec<FacultyOffering> = sqlx::query_as(
        "SELECT c.code, c.title, t.first_name || ' ' || t.last_name AS teacher, o.enrolled_count AS enrolled, o.capacity
         FROM course_offerings o JOIN courses c ON c.code = o.course_code JOIN users t ON t.id = o.teacher_id
         WHERE c.faculty_code = $1 AND o.semester = $2 ORDER BY c.code",
    )
    .bind(&code)
    .bind(&st.cfg.current_semester)
    .fetch_all(&st.db)
    .await?;
    let students: Vec<FacultyStudent> = sqlx::query_as(
        "SELECT u.id::text AS sub, u.first_name || ' ' || u.last_name AS name, r.faculty_number, s.name AS specialty, r.study_year, r.status
         FROM academic_records r JOIN users u ON u.id = r.user_id JOIN specialties s ON s.id = r.specialty_id
         WHERE s.faculty_code = $1 ORDER BY s.name, r.study_year, u.last_name",
    )
    .bind(&code)
    .fetch_all(&st.db)
    .await?;
    Ok(Json(json!({
        "faculty": { "code": code, "name": name },
        "semester": st.cfg.current_semester,
        "offerings": offerings,
        "students": students,
    })))
}
