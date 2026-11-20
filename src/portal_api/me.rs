//! Who is signed in, the form options, and profiles.

use axum::Json;
use axum::extract::{Path, State};
use serde_json::{Value, json};

use super::{ApiError, ApiResult, ApiUser, parse_sub};
use crate::access::{Permission, names_of};
use crate::oidc::claims;
use crate::state::AppState;

pub async fn me(State(st): State<AppState>, u: ApiUser) -> ApiResult<Json<Value>> {
    let c = claims::load(&st.db, u.user.id).await?.ok_or_else(ApiError::unauthenticated)?;
    let nav: Vec<Value> = super::nav_links(&u.user)
        .into_iter()
        .map(|(path, label)| json!({ "path": path, "label": label }))
        .collect();
    Ok(Json(json!({
        "user": {
            "sub": u.user.id.to_string(),
            "email": c.email,
            "given_name": c.first_name,
            "family_name": c.last_name,
            "name": c.name(),
            "role": u.user.role.as_str(),
            "role_label": u.user.role.label_bg(),
            "faculty": c.faculty(),
            "permissions": names_of(u.user.mask),
        },
        "csrf": u.csrf,
        "semester": st.cfg.current_semester,
        "nav": nav,
    })))
}

pub async fn options(State(st): State<AppState>, _u: ApiUser) -> ApiResult<Json<Value>> {
    let faculties: Vec<(String, String)> = sqlx::query_as("SELECT code, name FROM faculties ORDER BY code")
        .fetch_all(&st.db)
        .await?;
    let specialties: Vec<(i32, String, String)> =
        sqlx::query_as("SELECT id, faculty_code, name FROM specialties ORDER BY faculty_code, name")
            .fetch_all(&st.db)
            .await?;
    Ok(Json(json!({
        "faculties": faculties.into_iter().map(|(code, name)| json!({ "code": code, "name": name })).collect::<Vec<_>>(),
        "specialties": specialties.into_iter().map(|(id, f, n)| json!({ "id": id, "faculty_code": f, "name": n })).collect::<Vec<_>>(),
        "semester": st.cfg.current_semester,
    })))
}

/// The same shape as the directory's user lookup
pub fn profile_json(c: &claims::ClaimRow) -> Value {
    let mut m = c.for_scopes(&["email", "profile", "unicloud"]);
    m.remove("permissions");
    Value::Object(m)
}

pub async fn profile(State(st): State<AppState>, u: ApiUser) -> ApiResult<Json<Value>> {
    let c = claims::load(&st.db, u.user.id).await?.ok_or_else(ApiError::not_found)?;
    Ok(Json(profile_json(&c)))
}

/// Another student's profile, only with a permission that covers that student; otherwise 404
pub async fn student(State(st): State<AppState>, u: ApiUser, Path(sub): Path<String>) -> ApiResult<Json<Value>> {
    let id = parse_sub(&sub)?;
    let me = &u.user;
    let allowed = (me.id == id && me.can(Permission::StudentProfileReadOwn))
        || me.can(Permission::StudentProfileReadAny)
        || {
            let row: Option<(bool, bool)> = sqlx::query_as(
                "SELECT s.faculty_code = $2,
                    EXISTS (SELECT 1 FROM enrollments e JOIN course_offerings o ON o.id = e.offering_id
                            WHERE e.student_id = r.user_id AND o.teacher_id = $3 AND e.status = 'ENROLLED')
             FROM academic_records r JOIN specialties s ON s.id = r.specialty_id WHERE r.user_id = $1",
            )
            .bind(id)
            .bind(me.faculty_code.as_deref().unwrap_or(""))
            .bind(me.id)
            .fetch_optional(&st.db)
            .await?;
            matches!(row, Some((same_faculty, enrolled))
            if (same_faculty && me.can(Permission::StudentProfileReadFaculty)) || (enrolled && me.can(Permission::StudentProfileReadEnrolled)))
        };
    if !allowed {
        return Err(ApiError::not_found());
    }
    let c = claims::load(&st.db, id)
        .await?
        .filter(|c| c.faculty_number.is_some())
        .ok_or_else(ApiError::not_found)?;
    Ok(Json(profile_json(&c)))
}
