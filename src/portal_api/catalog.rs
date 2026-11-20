//! The catalogue, enrolling and a student's own enrollments.

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};

use super::{ApiError, ApiResult, ApiUser};
use crate::access::Permission;
use crate::enrollment::{self, EnrollOutcome};
use crate::state::AppState;

#[derive(Deserialize, Default)]
pub struct CatalogQuery {
    #[serde(default)]
    q: String,
    #[serde(default)]
    faculty: String,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct CatalogOffering {
    id: i64,
    code: String,
    title: String,
    ects: i16,
    faculty: String,
    teacher: String,
    enrolled: i32,
    capacity: i32,
    my_status: Option<String>,
}

pub async fn courses(State(st): State<AppState>, u: ApiUser, Query(q): Query<CatalogQuery>) -> ApiResult<Json<Vec<CatalogOffering>>> {
    u.require(Permission::CourseOfferingRead)?;
    let like = format!("%{}%", q.q.trim().to_lowercase());
    let rows = sqlx::query_as(
        "SELECT o.id, c.code, c.title, c.ects_credits AS ects, c.faculty_code AS faculty,
                t.first_name || ' ' || t.last_name AS teacher, o.enrolled_count AS enrolled, o.capacity,
                e.status AS my_status
         FROM course_offerings o
         JOIN courses c ON c.code = o.course_code
         JOIN users t ON t.id = o.teacher_id
         LEFT JOIN enrollments e ON e.offering_id = o.id AND e.student_id = $1
         WHERE o.semester = $2 AND o.is_open
           AND ($3 = '%%' OR lower(c.code) LIKE $3 OR lower(c.title) LIKE $3)
           AND ($4 = '' OR c.faculty_code = $4)
         ORDER BY c.code",
    )
    .bind(u.user.id)
    .bind(&st.cfg.current_semester)
    .bind(like)
    .bind(q.faculty.trim())
    .fetch_all(&st.db)
    .await?;
    Ok(Json(rows))
}

fn outcome_code(o: &EnrollOutcome) -> &'static str {
    match o {
        EnrollOutcome::Enrolled => "ENROLLED",
        EnrollOutcome::AlreadyEnrolled => "ALREADY_ENROLLED",
        EnrollOutcome::Full => "FULL",
        EnrollOutcome::Closed => "CLOSED",
        EnrollOutcome::Rejected => "REJECTED",
        EnrollOutcome::NotActive => "NOT_ACTIVE",
        EnrollOutcome::NoSuchOffering => "NO_SUCH_OFFERING",
    }
}

pub async fn enroll(State(st): State<AppState>, u: ApiUser, Path(id): Path<i64>) -> ApiResult<Response> {
    u.require(Permission::EnrollmentRequestCreate)?;
    let outcome = enrollment::enroll(&st.db, u.user.id, id, &st.cfg.current_semester).await?;
    let status = if outcome == EnrollOutcome::Enrolled {
        tracing::info!(student = u.user.id, offering = id, "enrolled");
        StatusCode::OK
    } else {
        StatusCode::CONFLICT
    };
    let body = serde_json::json!({ "outcome": outcome_code(&outcome), "message": outcome.message_bg() });
    Ok((status, Json(body)).into_response())
}

#[derive(Serialize, sqlx::FromRow)]
pub struct MyEnrollment {
    id: i64,
    code: String,
    title: String,
    semester: String,
    teacher: String,
    status: String,
}

pub async fn my_enrollments(State(st): State<AppState>, u: ApiUser) -> ApiResult<Json<Vec<MyEnrollment>>> {
    u.require(Permission::EnrollmentRequestCreate)?;
    let rows = sqlx::query_as(
        "SELECT e.id, c.code, c.title, o.semester, t.first_name || ' ' || t.last_name AS teacher, e.status
         FROM enrollments e JOIN course_offerings o ON o.id = e.offering_id JOIN courses c ON c.code = o.course_code JOIN users t ON t.id = o.teacher_id
         WHERE e.student_id = $1 ORDER BY o.semester DESC, c.code",
    )
    .bind(u.user.id)
    .fetch_all(&st.db)
    .await?;
    Ok(Json(rows))
}

pub async fn cancel(State(st): State<AppState>, u: ApiUser, Path(id): Path<i64>) -> ApiResult<StatusCode> {
    u.require(Permission::EnrollmentRequestCreate)?;
    if !enrollment::cancel(&st.db, u.user.id, id).await? {
        return Err(ApiError::conflict("Записването не може да бъде отказано."));
    }
    Ok(StatusCode::OK)
}
