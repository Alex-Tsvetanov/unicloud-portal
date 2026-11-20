//! Teacher pages (own offerings, rosters, rejection), the dean's faculty page, and reading another
//! student's profile, allowed only by a permission that covers that student.

use axum::extract::{Path, State};
use maud::{Markup, html};

use crate::access::Permission;
use crate::accounts::admin::require;
use crate::enrollment;
use crate::error::{AppError, AppResult};
use crate::oidc::claims;
use crate::state::AppState;
use crate::ui::components::{empty, flash, layout, page_title, seats, status_badge};
use crate::web::session::CurrentUser;

pub async fn teach(State(st): State<AppState>, cu: CurrentUser) -> AppResult<Markup> {
    require(&cu, Permission::EnrollmentRequestAcceptReject)?;
    let rows: Vec<(i64, String, String, String, i32, i32, bool)> = sqlx::query_as(
        "SELECT o.id, c.code, c.title, o.semester, o.enrolled_count, o.capacity, o.is_open
         FROM course_offerings o JOIN courses c ON c.code = o.course_code WHERE o.teacher_id = $1 ORDER BY o.semester DESC, c.code",
    )
    .bind(cu.user.id)
    .fetch_all(&st.db)
    .await?;
    Ok(layout(
        "Моите курсове",
        Some(&cu.nav()),
        html! {
            (page_title("Моите курсове", Some("Курсовете, които преподавате, и записаните студенти.")))
            @if rows.is_empty() { (empty("Нямате курсове.")) } @else {
                div.cards {
                    @for (id, code, title, sem, enrolled, cap, open) in &rows {
                        a.card.link-card href={"/teach/offerings/" (id)} {
                            p.kicker { code { (code) } " · " (sem) @if !*open { " · затворен" } }
                            h2 { (title) }
                            (seats(*enrolled, *cap))
                        }
                    }
                }
            }
        },
    ))
}

async fn roster(st: &AppState, offering: i64) -> sqlx::Result<Vec<(i64, i64, String, String, String)>> {
    sqlx::query_as(
        "SELECT e.id, u.id, u.first_name || ' ' || u.last_name, r.faculty_number, e.status
         FROM enrollments e JOIN users u ON u.id = e.student_id JOIN academic_records r ON r.user_id = u.id
         WHERE e.offering_id = $1 ORDER BY e.status, u.last_name",
    )
    .bind(offering)
    .fetch_all(&st.db)
    .await
}

fn roster_table(offering: i64, rows: &[(i64, i64, String, String, String)], notice: Option<(&str, &str)>) -> Markup {
    html! {
        div #roster {
            @if let Some((k, m)) = notice { (flash(k, m)) }
            @if rows.is_empty() { (empty("Няма записани студенти.")) } @else {
                table.table {
                    thead { tr { th { "Студент" } th { "Фак. номер" } th { "Статус" } th {} } }
                    tbody {
                        @for (eid, uid, name, fnum, status) in rows {
                            tr {
                                td { a href={"/students/" (uid)} { (name) } }
                                td { code { (fnum) } }
                                td { (status_badge(status)) }
                                td {
                                    @if status == "ENROLLED" {
                                        button.link.danger hx-post={"/teach/enrollments/" (eid) "/reject"} hx-vals={"{\"offering\":" (offering) "}"} hx-target="#roster" hx-swap="outerHTML" hx-confirm="Да отхвърлите ли записването? Студентът няма да може да се запише отново." { "Отхвърли" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub async fn offering_students(State(st): State<AppState>, cu: CurrentUser, Path(id): Path<i64>) -> AppResult<Markup> {
    require(&cu, Permission::EnrollmentRequestAcceptReject)?;
    let head: Option<(String, String, String)> = sqlx::query_as(
        "SELECT c.code, c.title, o.semester FROM course_offerings o JOIN courses c ON c.code = o.course_code WHERE o.id = $1 AND o.teacher_id = $2",
    )
    .bind(id)
    .bind(cu.user.id)
    .fetch_optional(&st.db)
    .await?;
    let (code, title, sem) = head.ok_or(AppError::NotFound)?;
    let rows = roster(&st, id).await?;
    Ok(layout(
        &title,
        Some(&cu.nav()),
        html! { (page_title(&format!("{code} · {title}"), Some(&format!("Семестър {sem}")))) (roster_table(id, &rows, None)) },
    ))
}

#[derive(serde::Deserialize)]
pub struct RejectForm {
    offering: i64,
}

pub async fn reject(
    State(st): State<AppState>,
    cu: CurrentUser,
    Path(eid): Path<i64>,
    axum::Form(f): axum::Form<RejectForm>,
) -> AppResult<Markup> {
    require(&cu, Permission::EnrollmentRequestAcceptReject)?;
    let done = enrollment::reject(&st.db, cu.user.id, eid).await?;
    if done {
        tracing::info!(teacher = cu.user.id, enrollment = eid, "enrollment rejected");
    }
    let mine: Option<(i64,)> = sqlx::query_as("SELECT id FROM course_offerings WHERE id = $1 AND teacher_id = $2")
        .bind(f.offering)
        .bind(cu.user.id)
        .fetch_optional(&st.db)
        .await?;
    if mine.is_none() {
        return Err(AppError::NotFound);
    }
    let rows = roster(&st, f.offering).await?;
    let notice = if done {
        ("ok", "Записването е отхвърлено. Мястото е освободено.")
    } else {
        ("error", "Записването не може да бъде отхвърлено.")
    };
    Ok(roster_table(f.offering, &rows, Some(notice)))
}

pub async fn faculty(State(st): State<AppState>, cu: CurrentUser) -> AppResult<Markup> {
    require(&cu, Permission::StudentProfileReadFaculty)?;
    let code = cu.user.faculty_code.clone().ok_or(AppError::NotFound)?;
    let offerings: Vec<(String, String, String, String, i32, i32)> = sqlx::query_as(
        "SELECT c.code, c.title, o.semester, t.first_name || ' ' || t.last_name, o.enrolled_count, o.capacity
         FROM course_offerings o JOIN courses c ON c.code = o.course_code JOIN users t ON t.id = o.teacher_id
         WHERE c.faculty_code = $1 AND o.semester = $2 ORDER BY c.code",
    )
    .bind(&code)
    .bind(&st.cfg.current_semester)
    .fetch_all(&st.db)
    .await?;
    let students: Vec<(i64, String, String, String, i16, String)> = sqlx::query_as(
        "SELECT u.id, u.first_name || ' ' || u.last_name, r.faculty_number, s.name, r.study_year, r.status
         FROM academic_records r JOIN users u ON u.id = r.user_id JOIN specialties s ON s.id = r.specialty_id
         WHERE s.faculty_code = $1 ORDER BY s.name, r.study_year, u.last_name",
    )
    .bind(&code)
    .fetch_all(&st.db)
    .await?;
    Ok(layout(
        "Факултет",
        Some(&cu.nav()),
        html! {
            (page_title(&format!("Факултет {code}"), Some(&format!("Курсове и студенти, семестър {}.", st.cfg.current_semester))))
            section.card {
                h2 { "Курсове (" (offerings.len()) ")" }
                @if offerings.is_empty() { (empty("Няма курсове този семестър.")) } @else {
                    table.table { thead { tr { th { "Код" } th { "Дисциплина" } th { "Преподавател" } th { "Места" } } }
                        tbody { @for (c, t, _s, teacher, e, cap) in &offerings { tr { td { code { (c) } } td { (t) } td { (teacher) } td { (seats(*e, *cap)) } } } } }
                }
            }
            section.card {
                h2 { "Студенти (" (students.len()) ")" }
                @if students.is_empty() { (empty("Няма студенти.")) } @else {
                    table.table { thead { tr { th { "Студент" } th { "Фак. номер" } th { "Специалност" } th { "Курс" } th { "Статус" } } }
                        tbody { @for (id, n, f, sp, y, s) in &students { tr { td { a href={"/students/" (id)} { (n) } } td { code { (f) } } td { (sp) } td { (y) } td { (status_badge(s)) } } } } }
                }
            }
        },
    ))
}

/// Another student's profile. Anyone without a covering permission gets 404, not 403.
pub async fn student_profile(State(st): State<AppState>, cu: CurrentUser, Path(id): Path<i64>) -> AppResult<Markup> {
    let u = &cu.user;
    let allowed = if (u.id == id && u.can(Permission::StudentProfileReadOwn)) || u.can(Permission::StudentProfileReadAny) {
        true
    } else {
        let row: Option<(bool, bool)> = sqlx::query_as(
            "SELECT s.faculty_code = $2,
                    EXISTS (SELECT 1 FROM enrollments e JOIN course_offerings o ON o.id = e.offering_id
                            WHERE e.student_id = r.user_id AND o.teacher_id = $3 AND e.status = 'ENROLLED')
             FROM academic_records r JOIN specialties s ON s.id = r.specialty_id WHERE r.user_id = $1",
        )
        .bind(id)
        .bind(u.faculty_code.as_deref().unwrap_or(""))
        .bind(u.id)
        .fetch_optional(&st.db)
        .await?;
        matches!(row, Some((same_faculty, enrolled)) if (same_faculty && u.can(Permission::StudentProfileReadFaculty)) || (enrolled && u.can(Permission::StudentProfileReadEnrolled)))
    };
    if !allowed {
        return Err(AppError::NotFound);
    }
    let c = claims::load(&st.db, id)
        .await?
        .filter(|c| c.faculty_number.is_some())
        .ok_or(AppError::NotFound)?;
    Ok(layout(
        &c.name(),
        Some(&cu.nav()),
        html! {
            (page_title(&c.name(), Some("Академичен профил")))
            section.card.narrow {
                dl.facts {
                    dt { "Имейл" } dd { (c.email) }
                    dt { "Факултет" } dd { (c.faculty_name.as_deref().unwrap_or("–")) }
                    dt { "Факултетен номер" } dd { code { (c.faculty_number.as_deref().unwrap_or("")) } }
                    dt { "Специалност" } dd { (c.specialty.as_deref().unwrap_or("")) }
                    dt { "Курс" } dd { (c.study_year.unwrap_or(0)) }
                    dt { "Статус" } dd { (status_badge(c.status.as_deref().unwrap_or(""))) }
                }
            }
        },
    ))
}
