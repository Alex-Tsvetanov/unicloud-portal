//! Portal behaviour: staging tools, accounts, CSRF, roles and the enrollment race.

mod common;

use common::*;
use serde_json::json;
use unicloud_portal::config::Environment;
use unicloud_portal::enrollment::{self, EnrollOutcome};

#[tokio::test]
async fn staging_api_keeps_the_java_shapes() {
    let p = start(Environment::Staging, &[]).await;
    let b = Browser::new(&p.base);
    let roles = b.get_with("/api/v1/staging/roles", &[]).await.json();
    assert_eq!(roles["permissions"].as_array().unwrap().len(), 11);
    assert_eq!(roles["roleDefaults"]["ADMIN"], json!(["STAFF_CREATE"]));
    assert_eq!(
        roles["roleDefaults"]["DEAN"],
        json!(["COURSE_OFFERING_READ", "STUDENT_PROFILE_READ_FACULTY"])
    );

    let bad_role = b.post_json("/api/v1/staging/users", &json!({ "email": format!("{}@x.bg", uniq("r")), "password": "a-long-password", "firstName": "A", "lastName": "B", "role": "KING" })).await;
    assert_eq!(bad_role.status, 400);
    assert!(bad_role.json()["message"].as_str().unwrap().starts_with("role: must be one of"));

    let bad_perm = b.post_json("/api/v1/staging/users", &json!({ "email": format!("{}@x.bg", uniq("r")), "password": "a-long-password", "firstName": "A", "lastName": "B", "role": "TEACHER", "faculty": "FCST", "permissions": ["FLY"] })).await;
    assert!(
        bad_perm.json()["message"]
            .as_str()
            .unwrap()
            .starts_with("permissions: unknown permission 'FLY'")
    );

    let teacher_email = format!("{}@x.bg", uniq("t"));
    let t = b.post_json("/api/v1/staging/users", &json!({ "email": teacher_email.to_uppercase(), "password": "a-long-password", "firstName": "A", "lastName": "B", "role": "teacher", "faculty": "FCST", "permissions": [] })).await;
    assert_eq!(t.status, 201, "{}", t.text);
    assert_eq!(t.json()["email"], teacher_email, "stored lower-cased");
    assert_eq!(t.json()["permissions"], json!([]));
    assert_eq!(t.json()["student"], false);
    let dup = b.post_json("/api/v1/staging/users", &json!({ "email": teacher_email, "password": "a-long-password", "firstName": "A", "lastName": "B", "role": "TEACHER", "faculty": "FCST" })).await;
    assert_eq!(dup.status, 409);

    let id = t.json()["id"].as_i64().unwrap();
    let put = b
        .put_json(
            &format!("/api/v1/staging/users/{id}/permissions"),
            &json!({ "permissions": ["STAFF_CREATE", "COURSE_OFFERING_READ"] }),
        )
        .await;
    assert_eq!(put.json()["permissions"], json!(["STAFF_CREATE", "COURSE_OFFERING_READ"]));
    assert_eq!(
        b.put_json("/api/v1/staging/users/999999999/permissions", &json!({ "permissions": [] }))
            .await
            .status,
        404
    );

    let bad_id = b
        .post_json(
            "/api/v1/staging/sso-clients",
            &json!({ "clientId": "-bad", "name": "x", "redirectUris": [CB] }),
        )
        .await;
    assert_eq!(bad_id.status, 400);
    assert_eq!(bad_id.json()["fieldErrors"][0]["field"], "clientId");
    let reserved = b
        .post_json(
            "/api/v1/staging/sso-clients",
            &json!({ "clientId": "unicloud-api", "name": "x", "redirectUris": [CB] }),
        )
        .await;
    assert_eq!(reserved.status, 400);

    // Staging users sign in without a forced password change
    let (_, email, pw) = staging_student(&b).await;
    let s = b.sign_in(&email, &pw).await;
    assert_eq!(s.location(), "/");
}

#[tokio::test]
async fn staging_tools_do_not_exist_in_production() {
    let p = start(Environment::Production, &[]).await;
    let b = Browser::new(&p.base);
    for path in ["/staging", "/api/v1/staging/roles", "/api/v1/staging/users"] {
        assert_eq!(b.get(path).await.status, 404, "{path}");
    }
    assert_eq!(
        b.post_json("/api/v1/auth/login", &json!({})).await.status,
        404,
        "the old token API is gone"
    );
}

#[tokio::test]
async fn sign_in_csrf_and_forced_password_change() {
    let p = start(Environment::Staging, &[]).await;
    let b = Browser::new(&p.base);
    // A user created through the UI must change the first password
    let email = format!("{}@test.tu-sofia.bg", uniq("f"));
    let spec: (i32,) = sqlx::query_as("SELECT id FROM specialties LIMIT 1")
        .fetch_one(&p.st.db)
        .await
        .unwrap();
    unicloud_portal::accounts::admin::create_user(
        &p.st.db,
        unicloud_portal::accounts::admin::NewUser {
            email: email.clone(),
            password: "first-password".into(),
            first_name: "Нов".into(),
            last_name: "Студент".into(),
            role: unicloud_portal::access::Role::Student,
            mask: unicloud_portal::access::Role::Student.default_mask(),
            faculty_code: None,
            student: Some(unicloud_portal::accounts::admin::NewRecord {
                faculty_number: uniq("7"),
                specialty_id: spec.0,
                study_year: 1,
                status: "ACTIVE".into(),
            }),
            must_change_password: true,
        },
    )
    .await
    .unwrap();

    let wrong = b.sign_in(&email, "wrong-password").await;
    assert_eq!(wrong.status, 401);
    assert!(wrong.text.contains("Грешен имейл или парола"));

    b.sign_in(&email, "first-password").await;
    assert_eq!(b.get("/profile").await.location(), "/password");
    let (client, _) = staging_client(&b, false).await;
    assert_eq!(
        b.get(&authorize_url(&client, "openid", &pkce(), &[])).await.location(),
        "/password",
        "SSO waits for the change too"
    );

    // CSRF: a form post without the token is refused
    assert_eq!(
        b.post_form(
            "/password",
            &[("password", "first-password"), ("new_password", "x"), ("new_password2", "x")]
        )
        .await
        .status,
        403
    );
    let page = b.get("/password").await;
    let csrf = csrf_from(&page.text);
    let changed = b
        .post_form(
            "/password",
            &[
                ("csrf", &csrf),
                ("password", "first-password"),
                ("new_password", "my-new-password"),
                ("new_password2", "my-new-password"),
            ],
        )
        .await;
    assert!(
        changed.location().starts_with("/oauth2/authorize?"),
        "continues the waiting SSO login: {}",
        changed.location()
    );
    assert_eq!(b.get("/profile").await.status, 200);

    // Sign-out ends the session
    let csrf = csrf_from(&b.get("/profile").await.text);
    b.post_form("/logout", &[("csrf", &csrf)]).await;
    assert_eq!(b.get("/profile").await.location(), "/login");
}

#[tokio::test]
async fn permissions_decide_pages() {
    let p = start(Environment::Staging, &[]).await;
    let b = Browser::new(&p.base);
    let (_, email, pw) = staging_student(&b).await;
    b.sign_in(&email, &pw).await;
    assert_eq!(b.get("/courses").await.status, 200);
    assert_eq!(b.get("/admin/users").await.status, 403);
    assert_eq!(b.get("/inspect/students").await.status, 403);
    assert_eq!(b.get("/admin/modules").await.status, 403);
    let other = staging_student(&Browser::new(&p.base)).await.0;
    assert_eq!(
        b.get(&format!("/students/{other}")).await.status,
        404,
        "another student's profile is hidden"
    );
}

#[tokio::test]
async fn enrollment_never_oversells_under_a_rush() {
    let p = start(Environment::Staging, &[]).await;
    let db = &p.st.db;
    let b = Browser::new(&p.base);
    let teacher = b.post_json("/api/v1/staging/users", &json!({ "email": format!("{}@x.bg", uniq("t")), "password": "a-long-password", "firstName": "T", "lastName": "T", "role": "TEACHER", "faculty": "FCST" })).await.json()["id"].as_i64().unwrap();
    let code = uniq("R").to_uppercase()[..10].to_string();
    sqlx::query("INSERT INTO courses (code, title, ects_credits, faculty_code) VALUES ($1, 'Rush', 5, 'FCST')")
        .bind(&code)
        .execute(db)
        .await
        .unwrap();
    let (offering,): (i64,) = sqlx::query_as(
        "INSERT INTO course_offerings (course_code, semester, teacher_id, capacity) VALUES ($1, '2026-27-W', $2, 10) RETURNING id",
    )
    .bind(&code)
    .bind(teacher)
    .fetch_one(db)
    .await
    .unwrap();
    let mut students = Vec::new();
    for _ in 0..40 {
        students.push(staging_student(&b).await.0);
    }
    let tasks: Vec<_> = students
        .iter()
        .map(|s| {
            let db = db.clone();
            let s = *s;
            tokio::spawn(async move { enrollment::enroll(&db, s, offering).await.unwrap() })
        })
        .collect();
    let mut enrolled = 0;
    for t in tasks {
        match t.await.unwrap() {
            EnrollOutcome::Enrolled => enrolled += 1,
            EnrollOutcome::Full => {}
            other => panic!("unexpected {other:?}"),
        }
    }
    assert_eq!(enrolled, 10);
    let (count, rows): (i32, i64) = sqlx::query_as("SELECT o.enrolled_count, (SELECT count(*) FROM enrollments WHERE offering_id = o.id AND status = 'ENROLLED') FROM course_offerings o WHERE o.id = $1")
        .bind(offering)
        .fetch_one(db)
        .await
        .unwrap();
    assert_eq!((count, rows), (10, 10));

    // Rejection frees the place and blocks the student from this offering
    let (eid, sid): (i64, i64) =
        sqlx::query_as("SELECT id, student_id FROM enrollments WHERE offering_id = $1 AND status = 'ENROLLED' LIMIT 1")
            .bind(offering)
            .fetch_one(db)
            .await
            .unwrap();
    assert!(enrollment::reject(db, teacher, eid).await.unwrap());
    assert_eq!(enrollment::enroll(db, sid, offering).await.unwrap(), EnrollOutcome::Rejected);
    let late = students.iter().find(|s| **s != sid).copied().unwrap();
    let again = enrollment::enroll(db, late, offering).await.unwrap();
    assert!(matches!(again, EnrollOutcome::Enrolled | EnrollOutcome::AlreadyEnrolled));
}

#[tokio::test]
async fn health_and_replica_header() {
    let p = start(Environment::Production, &[]).await;
    let b = Browser::new(&p.base);
    let h = b.get_with("/health", &[]).await;
    assert_eq!(h.json()["version"], "test");
    assert_eq!(h.headers.get("x-served-by").unwrap(), "test-replica");
    let root = b.get_with("/", &[("accept", "application/json")]).await;
    assert_eq!(root.json()["replica"], "test-replica");
    assert_eq!(b.get("/").await.location(), "/login");
}
