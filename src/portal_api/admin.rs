//! Administration: staff accounts and module (OIDC client) registration.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::inspect::{create_error, created};
use super::{ApiBody, ApiError, ApiResult, ApiUser, parse_sub};
use crate::access::{Permission, Role};
use crate::accounts::admin::{NewUser, create_user};
use crate::oidc::clients::{RegisterError, Registration, register};
use crate::state::AppState;

#[derive(Serialize, sqlx::FromRow)]
pub struct StaffMember {
    sub: String,
    name: String,
    email: String,
    role: String,
    faculty: Option<String>,
    enabled: bool,
}

pub async fn staff(State(st): State<AppState>, u: ApiUser) -> ApiResult<Json<Vec<StaffMember>>> {
    u.require(Permission::StaffCreate)?;
    let rows = sqlx::query_as(
        "SELECT id::text AS sub, first_name || ' ' || last_name AS name, email, role, faculty_code AS faculty, enabled
         FROM users WHERE role <> 'STUDENT' ORDER BY role, last_name",
    )
    .fetch_all(&st.db)
    .await?;
    Ok(Json(rows))
}

#[derive(Deserialize)]
pub struct NewStaff {
    email: String,
    first_name: String,
    last_name: String,
    role: String,
    faculty_code: Option<String>,
    password: String,
}

pub async fn create_staff(State(st): State<AppState>, u: ApiUser, ApiBody(f): ApiBody<NewStaff>) -> ApiResult<Response> {
    u.require(Permission::StaffCreate)?;
    let role = Role::parse(&f.role)
        .filter(|r| *r != Role::Student)
        .ok_or_else(|| ApiError::invalid("Невалидна роля."))?;
    if role == Role::Admin && u.user.role != Role::Admin {
        return Err(ApiError::forbidden());
    }
    let new = NewUser {
        email: f.email,
        password: f.password,
        first_name: f.first_name,
        last_name: f.last_name,
        role,
        mask: role.default_mask(),
        faculty_code: f.faculty_code.filter(|s| !s.is_empty()),
        student: None,
        must_change_password: true,
    };
    let id = create_user(&st.db, new).await.map_err(create_error)?;
    tracing::info!(created = id, by = u.user.id, role = role.as_str(), "staff created");
    Ok(created(id))
}

pub async fn toggle_staff(State(st): State<AppState>, u: ApiUser, Path(sub): Path<String>) -> ApiResult<Json<serde_json::Value>> {
    u.require(Permission::StaffCreate)?;
    let id = parse_sub(&sub)?;
    if id == u.user.id {
        return Err(ApiError::invalid("Не можете да спрете собствения си профил."));
    }
    let (enabled,): (bool,) =
        sqlx::query_as("UPDATE users SET enabled = NOT enabled WHERE id = $1 AND role <> 'STUDENT' RETURNING enabled")
            .bind(id)
            .fetch_optional(&st.db)
            .await?
            .ok_or_else(ApiError::not_found)?;
    Ok(Json(json!({ "enabled": enabled })))
}

fn require_admin(u: &ApiUser) -> ApiResult<()> {
    if u.user.role == Role::Admin {
        Ok(())
    } else {
        Err(ApiError::forbidden())
    }
}

#[derive(Serialize, sqlx::FromRow)]
pub struct ModuleSummary {
    client_id: String,
    name: String,
    redirect_uris: Vec<String>,
    directory: bool,
    created: String,
}

pub async fn modules(State(st): State<AppState>, u: ApiUser) -> ApiResult<Json<Vec<ModuleSummary>>> {
    require_admin(&u)?;
    let rows = sqlx::query_as(
        "SELECT client_id, name, redirect_uris, directory_access AS directory, to_char(created_at, 'YYYY-MM-DD') AS created
         FROM oauth_clients ORDER BY created_at DESC",
    )
    .fetch_all(&st.db)
    .await?;
    Ok(Json(rows))
}

/// The answer holds the secret once: never cached
pub async fn register_module(State(st): State<AppState>, u: ApiUser, ApiBody(reg): ApiBody<Registration>) -> ApiResult<Response> {
    require_admin(&u)?;
    match register(&st.db, &st.cfg, reg, &format!("user:{}", u.user.id)).await {
        Ok(r) => {
            let mut res = (StatusCode::CREATED, Json(r)).into_response();
            res.headers_mut()
                .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
            Ok(res)
        }
        Err(RegisterError::Fields(f)) => {
            let msg: Vec<String> = f.into_iter().map(|(k, m)| format!("{k}: {m}")).collect();
            Err(ApiError::invalid(msg.join("; ")))
        }
        Err(RegisterError::Invalid(m)) => Err(ApiError::invalid(m)),
        Err(RegisterError::Duplicate(m)) => Err(ApiError::conflict(m)),
        Err(RegisterError::Db(e)) => Err(e.into()),
    }
}
