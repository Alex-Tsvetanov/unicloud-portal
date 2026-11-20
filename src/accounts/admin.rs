//! Creating accounts. Inspectors create students, admins create staff. There is no public sign-up.

use axum::Form;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use maud::{Markup, html};
use serde::Deserialize;
use sqlx::PgPool;

use super::password;
use crate::access::{Permission, Role};
use crate::error::{AppError, AppResult, is_unique_violation};
use crate::state::AppState;
use crate::ui::components::{csrf_field, empty, field, flash, layout, page_title, select, status_badge};
use crate::web::session::CurrentUser;

/// Everything needed to create a user, already validated by the caller's form rules
pub struct NewUser {
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub role: Role,
    pub mask: i64,
    pub faculty_code: Option<String>,
    pub student: Option<NewRecord>,
    pub must_change_password: bool,
}

pub struct NewRecord {
    pub faculty_number: String,
    pub specialty_id: i32,
    pub study_year: i16,
    pub status: String,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateError {
    #[error("{0}")]
    Invalid(String),
    #[error("{0}")]
    Duplicate(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<sqlx::Error> for CreateError {
    fn from(e: sqlx::Error) -> Self {
        CreateError::Other(e.into())
    }
}

pub fn valid_email(e: &str) -> bool {
    let at = e.find('@');
    e.len() <= 254
        && at.is_some_and(|i| i > 0 && e[i + 1..].contains('.') && !e[i + 1..].starts_with('.'))
        && !e.contains(char::is_whitespace)
}

/// Creates the user (and the academic record) in one transaction
pub async fn create_user(db: &PgPool, u: NewUser) -> Result<i64, CreateError> {
    let email = u.email.trim().to_lowercase();
    if !valid_email(&email) {
        return Err(CreateError::Invalid("Невалиден имейл.".into()));
    }
    if u.first_name.trim().is_empty() || u.last_name.trim().is_empty() || u.first_name.len() > 100 || u.last_name.len() > 100 {
        return Err(CreateError::Invalid("Името и фамилията са задължителни (до 100 знака).".into()));
    }
    password::check_strength(&u.password).map_err(|m| CreateError::Invalid(m.into()))?;
    if u.role.needs_faculty() && u.faculty_code.is_none() {
        return Err(CreateError::Invalid("Изберете факултет.".into()));
    }
    if (u.role == Role::Student) != u.student.is_some() {
        return Err(CreateError::Invalid("Само студент има академичен профил.".into()));
    }
    let faculty = if u.role.needs_faculty() { u.faculty_code } else { None };
    let hash = password::hash(u.password).await?;
    let mut tx = db.begin().await?;
    let inserted: Result<(i64,), sqlx::Error> = sqlx::query_as(
        "INSERT INTO users (email, password_hash, first_name, last_name, role, permissions_mask, faculty_code, must_change_password)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id",
    )
    .bind(&email)
    .bind(hash)
    .bind(u.first_name.trim())
    .bind(u.last_name.trim())
    .bind(u.role.as_str())
    .bind(u.mask)
    .bind(faculty)
    .bind(u.must_change_password)
    .fetch_one(&mut *tx)
    .await;
    let (id,) = match inserted {
        Ok(r) => r,
        Err(e) if is_unique_violation(&e) => {
            let msg = if u.role == Role::Dean {
                "Имейлът вече се използва или факултетът вече има декан."
            } else {
                "Имейлът вече се използва."
            };
            return Err(CreateError::Duplicate(msg.into()));
        }
        Err(e) if matches!(&e, sqlx::Error::Database(d) if d.code().as_deref() == Some("23503")) => {
            return Err(CreateError::Invalid("Непознат факултет.".into()));
        }
        Err(e) => return Err(e.into()),
    };
    if let Some(r) = u.student {
        if r.faculty_number.trim().is_empty() || r.faculty_number.len() > 20 {
            return Err(CreateError::Invalid("Факултетният номер е задължителен (до 20 знака).".into()));
        }
        if !(1..=6).contains(&r.study_year) {
            return Err(CreateError::Invalid("Курсът трябва да е от 1 до 6.".into()));
        }
        let res = sqlx::query(
            "INSERT INTO academic_records (user_id, faculty_number, specialty_id, study_year, status) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(id)
        .bind(r.faculty_number.trim())
        .bind(r.specialty_id)
        .bind(r.study_year)
        .bind(&r.status)
        .execute(&mut *tx)
        .await;
        match res {
            Ok(_) => {}
            Err(e) if is_unique_violation(&e) => return Err(CreateError::Duplicate("Факултетният номер вече съществува.".into())),
            Err(e) if matches!(&e, sqlx::Error::Database(d) if d.code().as_deref() == Some("23503")) => {
                return Err(CreateError::Invalid("Непозната специалност.".into()));
            }
            Err(e) => return Err(e.into()),
        }
    }
    tx.commit().await?;
    Ok(id)
}

pub async fn faculty_options(db: &PgPool) -> AppResult<Vec<(String, String)>> {
    Ok(sqlx::query_as("SELECT code, code || ' · ' || name FROM faculties ORDER BY code")
        .fetch_all(db)
        .await?)
}

pub async fn specialty_options(db: &PgPool) -> AppResult<Vec<(String, String)>> {
    Ok(
        sqlx::query_as("SELECT id::text, faculty_code || ' · ' || name FROM specialties ORDER BY faculty_code, name")
            .fetch_all(db)
            .await?,
    )
}

// ---------- Staff (admin) ----------

#[derive(Deserialize)]
pub struct StaffForm {
    email: String,
    first_name: String,
    last_name: String,
    role: String,
    faculty_code: Option<String>,
    password: String,
}

pub async fn staff_page(State(st): State<AppState>, cu: CurrentUser) -> AppResult<Markup> {
    require(&cu, Permission::StaffCreate)?;
    staff_view(&st, &cu, None).await
}

async fn staff_view(st: &AppState, cu: &CurrentUser, msg: Option<(&str, &str)>) -> AppResult<Markup> {
    let staff: Vec<(i64, String, String, String, Option<String>, bool)> = sqlx::query_as(
        "SELECT id, first_name || ' ' || last_name, email, role, faculty_code, enabled FROM users WHERE role <> 'STUDENT' ORDER BY role, last_name",
    )
    .fetch_all(&st.db)
    .await?;
    let roles: Vec<(String, String)> = [Role::Teacher, Role::Dean, Role::Inspector, Role::Admin]
        .iter()
        .map(|r| (r.as_str().to_string(), r.label_bg().to_string()))
        .collect();
    let mut faculties = vec![(String::new(), "(без факултет)".to_string())];
    faculties.extend(faculty_options(&st.db).await?);
    Ok(layout(
        "Служители",
        Some(&cu.nav()),
        html! {
            (page_title("Служители", Some("Преподаватели, декани, инспектори и администратори.")))
            div.grid-2 {
                section.card #staff-form {
                    h2 { "Нов служител" }
                    @if let Some((k, m)) = msg { (flash(k, m)) }
                    form method="post" action="/admin/users" hx-post="/admin/users" hx-target="body" {
                        (csrf_field(&cu.csrf))
                        (field("Имейл", "email", "email", "", true))
                        (field("Име", "first_name", "text", "", true))
                        (field("Фамилия", "last_name", "text", "", true))
                        (select("Роля", "role", &roles, "TEACHER"))
                        (select("Факултет", "faculty_code", &faculties, ""))
                        (field("Първа парола (сменя се при първото влизане)", "password", "text", "", true))
                        button.primary type="submit" { "Създай" }
                    }
                }
                section.card {
                    h2 { "Всички служители" }
                    @if staff.is_empty() { (empty("Няма служители.")) } @else {
                        table.table {
                            thead { tr { th { "Име" } th { "Роля" } th { "Факултет" } th { "Достъп" } } }
                            tbody {
                                @for (id, name, email, role, fac, enabled) in &staff {
                                    tr {
                                        td { (name) br; span.muted.small { (email) } }
                                        td { (Role::parse(role).map(|r| r.label_bg()).unwrap_or(role)) }
                                        td { (fac.as_deref().unwrap_or("–")) }
                                        td {
                                            @if *id != cu.user.id {
                                                button.link hx-post={"/admin/users/" (id) "/toggle"} hx-target="body" {
                                                    @if *enabled { "Спри" } @else { "Разреши" }
                                                }
                                            }
                                            @if !*enabled { " " (status_badge("CANCELLED")) }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
    ))
}

pub async fn staff_create(State(st): State<AppState>, cu: CurrentUser, Form(f): Form<StaffForm>) -> AppResult<Markup> {
    require(&cu, Permission::StaffCreate)?;
    let role = Role::parse(&f.role)
        .filter(|r| *r != Role::Student)
        .ok_or_else(|| AppError::BadRequest("Невалидна роля.".into()))?;
    if role == Role::Admin && cu.user.role != Role::Admin {
        return Err(AppError::Forbidden);
    }
    let res = create_user(
        &st.db,
        NewUser {
            email: f.email,
            password: f.password,
            first_name: f.first_name,
            last_name: f.last_name,
            role,
            mask: role.default_mask(),
            faculty_code: f.faculty_code.filter(|s| !s.is_empty()),
            student: None,
            must_change_password: true,
        },
    )
    .await;
    match res {
        Ok(id) => {
            tracing::info!(created = id, by = cu.user.id, role = role.as_str(), "staff created");
            staff_view(&st, &cu, Some(("ok", "Служителят е създаден."))).await
        }
        Err(CreateError::Invalid(m) | CreateError::Duplicate(m)) => staff_view(&st, &cu, Some(("error", &m))).await,
        Err(CreateError::Other(e)) => Err(e.into()),
    }
}

pub async fn staff_toggle(State(st): State<AppState>, headers: HeaderMap, cu: CurrentUser, Path(id): Path<i64>) -> AppResult<Response> {
    require(&cu, Permission::StaffCreate)?;
    if id == cu.user.id {
        return Err(AppError::BadRequest("Не можете да спрете собствения си профил.".into()));
    }
    sqlx::query("UPDATE users SET enabled = NOT enabled WHERE id = $1 AND role <> 'STUDENT'")
        .bind(id)
        .execute(&st.db)
        .await?;
    Ok(crate::web::redirect(&headers, "/admin/users"))
}

pub fn require(cu: &CurrentUser, p: Permission) -> AppResult<()> {
    if cu.user.can(p) { Ok(()) } else { Err(AppError::Forbidden) }
}

pub fn into_page(r: AppResult<Markup>) -> Response {
    r.into_response()
}
