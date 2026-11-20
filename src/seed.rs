//! Startup data. Production: a first admin from `APP_BOOTSTRAP_ADMIN_*`, if no admin exists.
//! Staging and development with `APP_DEMO_PASSWORD`: demo people, courses and offerings, once.

use crate::access::Role;
use crate::accounts::admin::{CreateError, NewRecord, NewUser, create_user};
use crate::state::AppState;

pub async fn run(st: &AppState) -> anyhow::Result<()> {
    if let Some((email, password)) = &st.cfg.bootstrap_admin {
        let (admins,): (i64,) = sqlx::query_as("SELECT count(*) FROM users WHERE role = 'ADMIN'")
            .fetch_one(&st.db)
            .await?;
        if admins == 0 {
            let u = person(email, password, "Системен", "Администратор", Role::Admin, None, true);
            ignore_duplicate(create_user(&st.db, u).await)?;
            tracing::info!(email, "bootstrap admin created; the password must be changed at first sign-in");
        }
    }
    if let Some(pw) = st.cfg.demo_password.as_deref().filter(|_| st.cfg.staging_tools()) {
        demo(st, pw).await?;
    }
    Ok(())
}

fn person(email: &str, pw: &str, first: &str, last: &str, role: Role, faculty: Option<&str>, must_change: bool) -> NewUser {
    NewUser {
        email: email.into(),
        password: pw.into(),
        first_name: first.into(),
        last_name: last.into(),
        role,
        mask: role.default_mask(),
        faculty_code: faculty.map(Into::into),
        student: None,
        must_change_password: must_change,
    }
}

fn ignore_duplicate(r: Result<i64, CreateError>) -> anyhow::Result<Option<i64>> {
    match r {
        Ok(id) => Ok(Some(id)),
        Err(CreateError::Duplicate(_)) => Ok(None),
        Err(CreateError::Invalid(m)) => anyhow::bail!("seed data is invalid: {m}"),
        Err(CreateError::Other(e)) => Err(e),
    }
}

async fn demo(st: &AppState, pw: &str) -> anyhow::Result<()> {
    let (courses,): (i64,) = sqlx::query_as("SELECT count(*) FROM courses").fetch_one(&st.db).await?;
    if courses > 0 {
        return Ok(());
    }
    tracing::info!("seeding demo data");
    let staff = [
        ("ivan.petrov@tu-sofia.bg", "Иван", "Петров", Role::Teacher),
        ("maria.ivanova@tu-sofia.bg", "Мария", "Иванова", Role::Teacher),
        ("georgi.dimitrov@tu-sofia.bg", "Георги", "Димитров", Role::Teacher),
        ("dean.fcst@tu-sofia.bg", "Петя", "Колева", Role::Dean),
        ("inspector.fcst@tu-sofia.bg", "Стела", "Маринова", Role::Inspector),
        ("admin@tu-sofia.bg", "Админ", "Портал", Role::Admin),
    ];
    let mut teacher_ids = Vec::new();
    for (email, first, last, role) in staff {
        let fac = role.needs_faculty().then_some("FCST");
        if let Some(id) = ignore_duplicate(create_user(&st.db, person(email, pw, first, last, role, fac, false)).await)? {
            if role == Role::Teacher {
                teacher_ids.push(id);
            }
        }
    }
    let spec: Vec<(i32, String)> = sqlx::query_as("SELECT id, name FROM specialties WHERE faculty_code = 'FCST' ORDER BY id")
        .fetch_all(&st.db)
        .await?;
    let spec_id = |name: &str| spec.iter().find(|(_, n)| n == name).map(|(i, _)| *i).unwrap_or(spec[0].0);
    let students = [
        (
            "student1@student.tu-sofia.bg",
            "Александър",
            "Стоянов",
            "121219001",
            "Компютърни науки",
            2,
            "ACTIVE",
        ),
        (
            "student2@student.tu-sofia.bg",
            "Елена",
            "Георгиева",
            "121219002",
            "Компютърни науки",
            2,
            "ACTIVE",
        ),
        (
            "student3@student.tu-sofia.bg",
            "Николай",
            "Тодоров",
            "121219003",
            "Софтуерно инженерство",
            3,
            "ACTIVE",
        ),
        (
            "student4@student.tu-sofia.bg",
            "Виктория",
            "Николова",
            "121219004",
            "Софтуерно инженерство",
            3,
            "INTERRUPTED",
        ),
        (
            "student5@student.tu-sofia.bg",
            "Димитър",
            "Петков",
            "121218001",
            "Компютърни науки",
            4,
            "ACTIVE",
        ),
    ];
    for (email, first, last, fnum, specialty, year, status) in students {
        let mut u = person(email, pw, first, last, Role::Student, None, false);
        u.student = Some(NewRecord {
            faculty_number: fnum.into(),
            specialty_id: spec_id(specialty),
            study_year: year,
            status: status.into(),
        });
        ignore_duplicate(create_user(&st.db, u).await)?;
    }
    let courses = [
        ("CS101", "Въведение в програмирането", 6),
        ("CS201", "Структури от данни и алгоритми", 6),
        ("CS301", "Бази данни", 5),
        ("CS302", "Уеб технологии", 5),
        ("CS401", "Софтуерно инженерство", 6),
        ("CS402", "Разпределени системи", 6),
        ("MATH1", "Линейна алгебра", 5),
        ("MATH2", "Дискретна математика", 5),
    ];
    for (code, title, ects) in courses {
        sqlx::query("INSERT INTO courses (code, title, ects_credits, faculty_code) VALUES ($1, $2, $3, 'FCST') ON CONFLICT DO NOTHING")
            .bind(code)
            .bind(title)
            .bind(ects as i16)
            .execute(&st.db)
            .await?;
    }
    if teacher_ids.len() == 3 {
        let sem = &st.cfg.current_semester;
        let offerings = [
            ("CS101", 0, 30, true),
            ("CS201", 0, 25, true),
            ("CS301", 1, 20, true),
            ("CS302", 1, 25, true),
            ("CS401", 2, 15, true),
            ("CS402", 2, 3, true),
            ("MATH1", 0, 40, false),
            ("MATH2", 1, 35, true),
        ];
        for (code, t, cap, open) in offerings {
            sqlx::query("INSERT INTO course_offerings (course_code, semester, teacher_id, capacity, is_open) VALUES ($1, $2, $3, $4, $5) ON CONFLICT DO NOTHING")
                .bind(code)
                .bind(sem)
                .bind(teacher_ids[t])
                .bind(cap)
                .bind(open)
                .execute(&st.db)
                .await?;
        }
    }
    Ok(())
}
