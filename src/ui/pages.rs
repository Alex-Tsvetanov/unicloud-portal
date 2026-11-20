//! Home and profile pages.

use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Redirect, Response};
use maud::{Markup, html};
use serde_json::json;
use tower_sessions::Session;

use crate::access::Permission;
use crate::error::AppResult;
use crate::oidc::claims;
use crate::state::AppState;
use crate::ui::components::{layout, page_title, status_badge};
use crate::web::session::{CurrentUser, session_user};

/// `/`: JSON `{service, version, replica}` for machines, the home page (or sign-in) for browsers
pub async fn home(State(st): State<AppState>, headers: HeaderMap, session: Session) -> AppResult<Response> {
    let wants_json = headers
        .get("accept")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|a| a.contains("application/json") && !a.contains("text/html"));
    if wants_json {
        return Ok(Json(json!({ "service": "unicloud-portal", "version": st.cfg.git_sha, "replica": st.cfg.replica })).into_response());
    }
    if session_user(&st, &session).await?.is_none() {
        return Ok(Redirect::to("/login").into_response());
    }
    Ok(Redirect::to("/home").into_response())
}

pub async fn dashboard(State(st): State<AppState>, cu: CurrentUser) -> AppResult<Markup> {
    let u = &cu.user;
    let stats: (i64, i64) = sqlx::query_as(
        "SELECT (SELECT count(*) FROM course_offerings WHERE semester = $1 AND is_open),
                (SELECT count(*) FROM enrollments WHERE student_id = $2 AND status = 'ENROLLED')",
    )
    .bind(&st.cfg.current_semester)
    .bind(u.id)
    .fetch_one(&st.db)
    .await?;
    let tiles = crate::ui::nav_links(u)
        .into_iter()
        .filter(|(h, _)| *h != "/home")
        .collect::<Vec<_>>();
    Ok(layout(
        "Начало",
        Some(&cu.nav()),
        html! {
            div.hero {
                p.kicker { (u.role.label_bg()) @if let Some(f) = &u.faculty_code { " · " (f) } }
                h1 { "Здравейте, " (u.first_name) "!" }
                p.muted { "Семестър " (st.cfg.current_semester) "." }
            }
            div.stats {
                @if u.can(Permission::CourseOfferingRead) {
                    div.stat { span.stat-n { (stats.0) } span.stat-l { "отворени курса този семестър" } }
                }
                @if u.can(Permission::EnrollmentRequestCreate) {
                    div.stat { span.stat-n { (stats.1) } span.stat-l { "ваши записвания" } }
                }
            }
            nav.tiles aria-label="Бързи връзки" {
                @for (href, label) in tiles { a.tile href=(href) { span { (label) } span.arrow { "→" } } }
            }
        },
    ))
}

pub async fn profile(State(st): State<AppState>, cu: CurrentUser) -> AppResult<Markup> {
    let c = claims::load(&st.db, cu.user.id).await?.ok_or(crate::error::AppError::NotFound)?;
    Ok(layout(
        "Профил",
        Some(&cu.nav()),
        html! {
            (page_title("Моят профил", None))
            div.grid-2 {
                section.card {
                    h2 { "Лични данни" }
                    dl.facts {
                        dt { "Име" } dd { (c.name()) }
                        dt { "Имейл" } dd { (c.email) }
                        dt { "Роля" } dd { (cu.user.role.label_bg()) }
                        dt { "Факултет" } dd { (c.faculty_name.as_deref().unwrap_or("–")) }
                    }
                    p { a href="/password" { "Смяна на парола" } }
                }
                @if c.faculty_number.is_some() {
                    section.card {
                        h2 { "Академичен профил" }
                        dl.facts {
                            dt { "Факултетен номер" } dd { code { (c.faculty_number.as_deref().unwrap_or("")) } }
                            dt { "Специалност" } dd { (c.specialty.as_deref().unwrap_or("")) }
                            dt { "Курс" } dd { (c.study_year.unwrap_or(0)) }
                            dt { "Статус" } dd { (status_badge(c.status.as_deref().unwrap_or(""))) }
                        }
                    }
                }
            }
        },
    ))
}
