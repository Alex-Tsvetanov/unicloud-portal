//! `/courses` (catalogue with live search) and `/enrollments` (a student's own enrollments).

use axum::Form;
use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use maud::{Markup, html};
use serde::Deserialize;

use crate::access::Permission;
use crate::accounts::admin::require;
use crate::enrollment::{self, EnrollOutcome};
use crate::error::AppResult;
use crate::state::AppState;
use crate::ui::components::{empty, flash, layout, page_title, seats, status_badge};
use crate::web::{is_htmx, session::CurrentUser};

#[derive(Deserialize, Default)]
pub struct CatalogQuery {
    #[serde(default)]
    q: String,
    #[serde(default)]
    faculty: String,
}

#[derive(sqlx::FromRow)]
struct OfferingRow {
    id: i64,
    code: String,
    title: String,
    ects: i16,
    faculty: String,
    teacher: String,
    enrolled: i32,
    capacity: i32,
    mine: Option<String>,
}

async fn offerings(st: &AppState, student: i64, q: &CatalogQuery) -> sqlx::Result<Vec<OfferingRow>> {
    let like = format!("%{}%", q.q.trim().to_lowercase());
    sqlx::query_as(
        "SELECT o.id, c.code, c.title, c.ects_credits AS ects, c.faculty_code AS faculty,
                t.first_name || ' ' || t.last_name AS teacher, o.enrolled_count AS enrolled, o.capacity,
                e.status AS mine
         FROM course_offerings o
         JOIN courses c ON c.code = o.course_code
         JOIN users t ON t.id = o.teacher_id
         LEFT JOIN enrollments e ON e.offering_id = o.id AND e.student_id = $1
         WHERE o.semester = $2 AND o.is_open
           AND ($3 = '%%' OR lower(c.code) LIKE $3 OR lower(c.title) LIKE $3)
           AND ($4 = '' OR c.faculty_code = $4)
         ORDER BY c.code",
    )
    .bind(student)
    .bind(&st.cfg.current_semester)
    .bind(like)
    .bind(q.faculty.trim())
    .fetch_all(&st.db)
    .await
}

fn offering_rows(rows: &[OfferingRow], can_enroll: bool, notice: Option<(&str, &str)>) -> Markup {
    html! {
        div #catalog-results {
            @if let Some((k, m)) = notice { (flash(k, m)) }
            @if rows.is_empty() { (empty("Няма отворени курсове по тези критерии.")) } @else {
                table.table {
                    thead { tr { th { "Код" } th { "Дисциплина" } th { "ECTS" } th { "Преподавател" } th { "Места" } @if can_enroll { th {} } } }
                    tbody {
                        @for o in rows {
                            tr {
                                td { code { (o.code) } }
                                td { (o.title) br; span.muted.small { (o.faculty) } }
                                td { (o.ects) }
                                td { (o.teacher) }
                                td { (seats(o.enrolled, o.capacity)) }
                                @if can_enroll {
                                    td {
                                        @match o.mine.as_deref() {
                                            Some("ENROLLED") => (status_badge("ENROLLED")),
                                            Some("REJECTED") => (status_badge("REJECTED")),
                                            _ => {
                                                @if o.enrolled < o.capacity {
                                                    button.primary.small hx-post={"/courses/" (o.id) "/enroll"} hx-target="#catalog-results" hx-swap="outerHTML" hx-include="#catalog-filter" { "Запиши се" }
                                                } @else { span.muted { "Няма места" } }
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
    }
}

pub async fn courses(
    State(st): State<AppState>,
    headers: HeaderMap,
    cu: CurrentUser,
    Query(q): Query<CatalogQuery>,
) -> AppResult<Response> {
    require(&cu, Permission::CourseOfferingRead)?;
    let rows = offerings(&st, cu.user.id, &q).await?;
    let can_enroll = cu.user.can(Permission::EnrollmentRequestCreate);
    if is_htmx(&headers) {
        return Ok(offering_rows(&rows, can_enroll, None).into_response());
    }
    let faculties = crate::accounts::admin::faculty_options(&st.db).await?;
    Ok(layout(
        "Курсове",
        Some(&cu.nav()),
        html! {
            (page_title("Курсове", Some(&format!("Отворени курсове за семестър {}.", st.cfg.current_semester))))
            form.filters #catalog-filter hx-get="/courses" hx-target="#catalog-results" hx-swap="outerHTML" hx-trigger="input changed delay:250ms from:input, change from:select" {
                input type="search" name="q" value=(q.q) placeholder="Търсене по код или име" aria-label="Търсене";
                select name="faculty" aria-label="Факултет" {
                    option value="" { "Всички факултети" }
                    @for (code, name) in &faculties { option value=(code) selected[*code == q.faculty] { (name) } }
                }
            }
            (offering_rows(&rows, can_enroll, None))
        },
    )
    .into_response())
}

pub async fn enroll(State(st): State<AppState>, cu: CurrentUser, Path(id): Path<i64>, Form(q): Form<CatalogQuery>) -> AppResult<Markup> {
    require(&cu, Permission::EnrollmentRequestCreate)?;
    let outcome = enrollment::enroll(&st.db, cu.user.id, id, &st.cfg.current_semester).await?;
    if outcome == EnrollOutcome::Enrolled {
        tracing::info!(student = cu.user.id, offering = id, "enrolled");
    }
    let kind = if outcome == EnrollOutcome::Enrolled { "ok" } else { "error" };
    let rows = offerings(&st, cu.user.id, &q).await?;
    Ok(offering_rows(&rows, true, Some((kind, outcome.message_bg()))))
}

#[derive(sqlx::FromRow)]
struct MyRow {
    id: i64,
    code: String,
    title: String,
    semester: String,
    teacher: String,
    status: String,
}

async fn my_rows(st: &AppState, student: i64) -> sqlx::Result<Vec<MyRow>> {
    sqlx::query_as(
        "SELECT e.id, c.code, c.title, o.semester, t.first_name || ' ' || t.last_name AS teacher, e.status
         FROM enrollments e JOIN course_offerings o ON o.id = e.offering_id JOIN courses c ON c.code = o.course_code JOIN users t ON t.id = o.teacher_id
         WHERE e.student_id = $1 ORDER BY o.semester DESC, c.code",
    )
    .bind(student)
    .fetch_all(&st.db)
    .await
}

fn my_table(rows: &[MyRow], notice: Option<(&str, &str)>) -> Markup {
    html! {
        div #my-enrollments {
            @if let Some((k, m)) = notice { (flash(k, m)) }
            @if rows.is_empty() { (empty("Още нямате записвания. Изберете курс от каталога.")) } @else {
                table.table {
                    thead { tr { th { "Код" } th { "Дисциплина" } th { "Семестър" } th { "Преподавател" } th { "Статус" } th {} } }
                    tbody {
                        @for r in rows {
                            tr {
                                td { code { (r.code) } } td { (r.title) } td { (r.semester) } td { (r.teacher) }
                                td { (status_badge(&r.status)) }
                                td {
                                    @if r.status == "ENROLLED" {
                                        button.link hx-post={"/enrollments/" (r.id) "/cancel"} hx-target="#my-enrollments" hx-swap="outerHTML" hx-confirm="Да откажете ли записването?" { "Откажи" }
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

pub async fn my_enrollments(State(st): State<AppState>, cu: CurrentUser) -> AppResult<Markup> {
    require(&cu, Permission::EnrollmentRequestCreate)?;
    let rows = my_rows(&st, cu.user.id).await?;
    Ok(layout(
        "Моите записвания",
        Some(&cu.nav()),
        html! { (page_title("Моите записвания", None)) (my_table(&rows, None)) },
    ))
}

pub async fn cancel(State(st): State<AppState>, cu: CurrentUser, Path(id): Path<i64>) -> AppResult<Markup> {
    require(&cu, Permission::EnrollmentRequestCreate)?;
    let done = enrollment::cancel(&st.db, cu.user.id, id).await?;
    let rows = my_rows(&st, cu.user.id).await?;
    let notice = if done {
        ("ok", "Записването е отказано. Мястото е освободено.")
    } else {
        ("error", "Записването не може да бъде отказано.")
    };
    Ok(my_table(&rows, Some(notice)))
}
