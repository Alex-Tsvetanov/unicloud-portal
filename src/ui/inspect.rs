//! Academic office pages: create students and change their status, manage courses and offerings.

use axum::Form;
use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::response::Response;
use maud::{Markup, html};
use serde::Deserialize;

use crate::access::{Permission, Role};
use crate::accounts::admin::{CreateError, NewRecord, NewUser, create_user, faculty_options, require, specialty_options};
use crate::error::{AppError, AppResult, is_unique_violation};
use crate::state::AppState;
use crate::ui::components::{csrf_field, empty, field, flash, layout, page_title, seats, select, status_badge};
use crate::web::{redirect, session::CurrentUser};

const STATUSES: [(&str, &str); 3] = [("ACTIVE", "Действащ"), ("INTERRUPTED", "Прекъснал"), ("GRADUATED", "Дипломиран")];

#[derive(Deserialize, Default)]
pub struct StudentFilter {
    #[serde(default)]
    q: String,
}

pub async fn students(State(st): State<AppState>, cu: CurrentUser, Query(f): Query<StudentFilter>) -> AppResult<Markup> {
    if !cu.user.can(Permission::StudentCreate) && !cu.user.can(Permission::StudentProfileReadAny) {
        return Err(AppError::Forbidden);
    }
    students_view(&st, &cu, &f.q, None).await
}

async fn students_view(st: &AppState, cu: &CurrentUser, q: &str, msg: Option<(&str, &str)>) -> AppResult<Markup> {
    let like = format!("%{}%", q.trim().to_lowercase());
    let rows: Vec<(i64, String, String, String, i16, String)> = sqlx::query_as(
        "SELECT u.id, u.first_name || ' ' || u.last_name, r.faculty_number, s.faculty_code || ' · ' || s.name, r.study_year, r.status
         FROM academic_records r JOIN users u ON u.id = r.user_id JOIN specialties s ON s.id = r.specialty_id
         WHERE $1 = '%%' OR lower(u.first_name || ' ' || u.last_name) LIKE $1 OR r.faculty_number LIKE $1 OR lower(u.email) LIKE $1
         ORDER BY u.last_name, u.first_name LIMIT 300",
    )
    .bind(like)
    .fetch_all(&st.db)
    .await?;
    let can_create = cu.user.can(Permission::StudentCreate);
    let specialties = specialty_options(&st.db).await?;
    let statuses: Vec<(String, String)> = STATUSES.iter().map(|(a, b)| (a.to_string(), b.to_string())).collect();
    Ok(layout(
        "Студенти",
        Some(&cu.nav()),
        html! {
            (page_title("Студенти", Some("Академични профили на студентите.")))
            div.grid-2 {
                @if can_create {
                    section.card {
                        h2 { "Нов студент" }
                        @if let Some((k, m)) = msg { (flash(k, m)) }
                        form method="post" action="/inspect/students" hx-boost="false" {
                            (csrf_field(&cu.csrf))
                            (field("Имейл", "email", "email", "", true))
                            (field("Име", "first_name", "text", "", true))
                            (field("Фамилия", "last_name", "text", "", true))
                            (field("Факултетен номер", "faculty_number", "text", "", true))
                            (select("Специалност", "specialty_id", &specialties, ""))
                            (field("Курс (1 до 6)", "study_year", "number", "1", true))
                            (field("Първа парола (сменя се при първото влизане)", "password", "text", "", true))
                            button.primary type="submit" { "Създай" }
                        }
                    }
                }
                section.card {
                    form.filters method="get" action="/inspect/students" {
                        input type="search" name="q" value=(q) placeholder="Име, имейл или факултетен номер";
                        button type="submit" { "Търси" }
                    }
                    @if rows.is_empty() { (empty("Няма студенти.")) } @else {
                        table.table {
                            thead { tr { th { "Студент" } th { "Фак. номер" } th { "Специалност" } th { "Курс" } th { "Статус" } } }
                            tbody {
                                @for (id, name, fnum, spec, year, status) in &rows {
                                    tr {
                                        td { a href={"/students/" (id)} { (name) } }
                                        td { code { (fnum) } }
                                        td.small { (spec) }
                                        td { (year) }
                                        td {
                                            @if can_create {
                                                form.inline method="post" action={"/inspect/students/" (id) "/status"} {
                                                    (csrf_field(&cu.csrf))
                                                    select name="status" data-autosubmit="true" aria-label="Статус" {
                                                        @for (v, t) in &statuses { option value=(v) selected[v == status] { (t) } }
                                                    }
                                                }
                                            } @else { (status_badge(status)) }
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

#[derive(Deserialize)]
pub struct StudentForm {
    email: String,
    first_name: String,
    last_name: String,
    faculty_number: String,
    specialty_id: i32,
    study_year: i16,
    password: String,
}

pub async fn create_student(State(st): State<AppState>, cu: CurrentUser, Form(f): Form<StudentForm>) -> AppResult<Markup> {
    require(&cu, Permission::StudentCreate)?;
    let res = create_user(
        &st.db,
        NewUser {
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
        },
    )
    .await;
    match res {
        Ok(id) => {
            tracing::info!(created = id, by = cu.user.id, "student created");
            students_view(&st, &cu, "", Some(("ok", "Студентът е създаден."))).await
        }
        Err(CreateError::Invalid(m) | CreateError::Duplicate(m)) => students_view(&st, &cu, "", Some(("error", &m))).await,
        Err(CreateError::Other(e)) => Err(e.into()),
    }
}

#[derive(Deserialize)]
pub struct StatusForm {
    status: String,
}

pub async fn set_status(
    State(st): State<AppState>,
    headers: HeaderMap,
    cu: CurrentUser,
    Path(id): Path<i64>,
    Form(f): Form<StatusForm>,
) -> AppResult<Response> {
    require(&cu, Permission::StudentCreate)?;
    if !STATUSES.iter().any(|(s, _)| *s == f.status) {
        return Err(AppError::BadRequest("Невалиден статус.".into()));
    }
    sqlx::query("UPDATE academic_records SET status = $1 WHERE user_id = $2")
        .bind(&f.status)
        .bind(id)
        .execute(&st.db)
        .await?;
    tracing::info!(student = id, status = f.status, by = cu.user.id, "student status changed");
    Ok(redirect(&headers, "/inspect/students"))
}

// ---------- Courses and offerings ----------

pub async fn courses(State(st): State<AppState>, cu: CurrentUser) -> AppResult<Markup> {
    if !cu.user.can(Permission::CourseCreateModify) && !cu.user.can(Permission::CourseOfferingCreateModify) {
        return Err(AppError::Forbidden);
    }
    courses_view(&st, &cu, None).await
}

async fn courses_view(st: &AppState, cu: &CurrentUser, msg: Option<(&str, &str)>) -> AppResult<Markup> {
    let courses: Vec<(String, String, i16, String)> =
        sqlx::query_as("SELECT code, title, ects_credits, faculty_code FROM courses ORDER BY code")
            .fetch_all(&st.db)
            .await?;
    let offerings: Vec<(i64, String, String, String, i32, i32, bool)> = sqlx::query_as(
        "SELECT o.id, o.course_code, o.semester, t.first_name || ' ' || t.last_name, o.enrolled_count, o.capacity, o.is_open
         FROM course_offerings o JOIN users t ON t.id = o.teacher_id ORDER BY o.semester DESC, o.course_code",
    )
    .fetch_all(&st.db)
    .await?;
    let teachers: Vec<(String, String)> = sqlx::query_as(
        "SELECT id::text, last_name || ' ' || first_name || ' (' || COALESCE(faculty_code, '–') || ')' FROM users WHERE role = 'TEACHER' AND enabled ORDER BY last_name",
    )
    .fetch_all(&st.db)
    .await?;
    let course_opts: Vec<(String, String)> = courses.iter().map(|(c, t, _, _)| (c.clone(), format!("{c} · {t}"))).collect();
    let faculties = faculty_options(&st.db).await?;
    let can_course = cu.user.can(Permission::CourseCreateModify);
    let can_offer = cu.user.can(Permission::CourseOfferingCreateModify);
    Ok(layout(
        "Дисциплини",
        Some(&cu.nav()),
        html! {
            (page_title("Дисциплини и курсове", Some("Каталог на дисциплините и курсовете по семестри.")))
            @if let Some((k, m)) = msg { (flash(k, m)) }
            div.grid-2 {
                @if can_course {
                    section.card {
                        h2 { "Нова дисциплина" }
                        form method="post" action="/inspect/courses" hx-boost="false" {
                            (csrf_field(&cu.csrf))
                            (field("Код (до 16 знака, напр. MATH1)", "code", "text", "", true))
                            (field("Наименование", "title", "text", "", true))
                            (field("ECTS кредити", "ects_credits", "number", "5", true))
                            (select("Факултет", "faculty_code", &faculties, ""))
                            button.primary type="submit" { "Добави" }
                        }
                    }
                }
                @if can_offer {
                    section.card {
                        h2 { "Нов курс за семестър" }
                        @if course_opts.is_empty() || teachers.is_empty() { (empty("Нужни са поне една дисциплина и един преподавател.")) } @else {
                            form method="post" action="/inspect/offerings" hx-boost="false" {
                                (csrf_field(&cu.csrf))
                                (select("Дисциплина", "course_code", &course_opts, ""))
                                (field("Семестър", "semester", "text", &st.cfg.current_semester, true))
                                (select("Преподавател", "teacher_id", &teachers, ""))
                                (field("Места", "capacity", "number", "30", true))
                                button.primary type="submit" { "Добави" }
                            }
                        }
                    }
                }
            }
            section.card {
                h2 { "Курсове" }
                @if offerings.is_empty() { (empty("Няма курсове.")) } @else {
                    table.table {
                        thead { tr { th { "Код" } th { "Семестър" } th { "Преподавател" } th { "Места" } th { "Записване" } } }
                        tbody {
                            @for (id, code, sem, teacher, e, cap, open) in &offerings {
                                tr {
                                    td { code { (code) } } td { (sem) } td { (teacher) } td { (seats(*e, *cap)) }
                                    td {
                                        @if can_offer {
                                            form.inline method="post" action={"/inspect/offerings/" (id) "/toggle"} {
                                                (csrf_field(&cu.csrf))
                                                button.link type="submit" { @if *open { "Отворено · затвори" } @else { "Затворено · отвори" } }
                                            }
                                        } @else { @if *open { "отворено" } @else { "затворено" } }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            section.card {
                h2 { "Дисциплини" }
                table.table {
                    thead { tr { th { "Код" } th { "Наименование" } th { "ECTS" } th { "Факултет" } } }
                    tbody { @for (c, t, e, f) in &courses { tr { td { code { (c) } } td { (t) } td { (e) } td { (f) } } } }
                }
            }
        },
    ))
}

#[derive(Deserialize)]
pub struct CourseForm {
    code: String,
    title: String,
    ects_credits: i16,
    faculty_code: String,
}

pub async fn create_course(State(st): State<AppState>, cu: CurrentUser, Form(f): Form<CourseForm>) -> AppResult<Markup> {
    require(&cu, Permission::CourseCreateModify)?;
    let code = f.code.trim().to_uppercase();
    if code.is_empty() || code.len() > 16 || !code.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return courses_view(
            &st,
            &cu,
            Some(("error", "Кодът трябва да е до 16 латински букви, цифри или тирета.")),
        )
        .await;
    }
    if f.title.trim().is_empty() || !(1..=30).contains(&f.ects_credits) {
        return courses_view(&st, &cu, Some(("error", "Попълнете наименование и ECTS от 1 до 30."))).await;
    }
    let r = sqlx::query("INSERT INTO courses (code, title, ects_credits, faculty_code) VALUES ($1, $2, $3, $4)")
        .bind(&code)
        .bind(f.title.trim())
        .bind(f.ects_credits)
        .bind(&f.faculty_code)
        .execute(&st.db)
        .await;
    match r {
        Ok(_) => courses_view(&st, &cu, Some(("ok", "Дисциплината е добавена."))).await,
        Err(e) if is_unique_violation(&e) => courses_view(&st, &cu, Some(("error", "Дисциплина с този код вече съществува."))).await,
        Err(e) => Err(e.into()),
    }
}

#[derive(Deserialize)]
pub struct OfferingForm {
    course_code: String,
    semester: String,
    teacher_id: i64,
    capacity: i32,
}

pub async fn create_offering(State(st): State<AppState>, cu: CurrentUser, Form(f): Form<OfferingForm>) -> AppResult<Markup> {
    require(&cu, Permission::CourseOfferingCreateModify)?;
    if f.capacity < 1 || f.semester.trim().is_empty() || f.semester.len() > 16 {
        return courses_view(
            &st,
            &cu,
            Some(("error", "Местата трябва да са поне 1, а семестърът да е попълнен.")),
        )
        .await;
    }
    // The teacher must belong to the course's faculty
    let ok: Option<(bool,)> = sqlx::query_as(
        "SELECT t.faculty_code = c.faculty_code FROM users t, courses c WHERE t.id = $1 AND t.role = 'TEACHER' AND c.code = $2",
    )
    .bind(f.teacher_id)
    .bind(&f.course_code)
    .fetch_optional(&st.db)
    .await?;
    if ok.map(|r| r.0) != Some(true) {
        return courses_view(
            &st,
            &cu,
            Some(("error", "Преподавателят трябва да е от факултета на дисциплината.")),
        )
        .await;
    }
    let r = sqlx::query("INSERT INTO course_offerings (course_code, semester, teacher_id, capacity) VALUES ($1, $2, $3, $4)")
        .bind(&f.course_code)
        .bind(f.semester.trim())
        .bind(f.teacher_id)
        .bind(f.capacity)
        .execute(&st.db)
        .await;
    match r {
        Ok(_) => courses_view(&st, &cu, Some(("ok", "Курсът е добавен."))).await,
        Err(e) if is_unique_violation(&e) => courses_view(&st, &cu, Some(("error", "Този курс вече съществува за семестъра."))).await,
        Err(e) => Err(e.into()),
    }
}

pub async fn toggle_offering(State(st): State<AppState>, headers: HeaderMap, cu: CurrentUser, Path(id): Path<i64>) -> AppResult<Response> {
    require(&cu, Permission::CourseOfferingCreateModify)?;
    sqlx::query("UPDATE course_offerings SET is_open = NOT is_open WHERE id = $1")
        .bind(id)
        .execute(&st.db)
        .await?;
    Ok(redirect(&headers, "/inspect/courses"))
}
