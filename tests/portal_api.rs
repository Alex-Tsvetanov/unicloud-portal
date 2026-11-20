//! The React app's JSON API (`/api/v1/portal`): one journey through every role, plus the rules
//! the app relies on (CSRF header, JSON errors, return path after sign-in, no-store secrets).

mod common;

use common::*;
use serde_json::json;
use unicloud_portal::config::Environment;

async fn signed_in(p: &Portal, email: &str, pw: &str) -> Browser {
    let b = Browser::new(&p.base);
    b.sign_in(email, pw).await;
    b
}

#[tokio::test]
async fn every_role_works_through_the_api() {
    let p = start(Environment::Staging, &[]).await;
    let tools = Browser::new(&p.base);
    // A faculty of its own: each faculty has at most one dean, and the test database is shared
    let fac = uniq("T").to_uppercase();
    sqlx::query("INSERT INTO faculties (code, name) VALUES ($1, 'Тестов факултет')")
        .bind(&fac)
        .execute(&p.st.db)
        .await
        .unwrap();
    let (_, ie, ip) = staging_staff(&tools, "INSPECTOR", Some(&fac)).await;
    let (teacher, te, tp) = staging_staff(&tools, "TEACHER", Some(&fac)).await;
    let (_, de, dp) = staging_staff(&tools, "DEAN", Some(&fac)).await;

    // Inspector: a course, an offering for the current semester, and a student
    let inspector = signed_in(&p, &ie, &ip).await;
    let me = inspector.me().await.json();
    assert_eq!(me["user"]["role"], "INSPECTOR");
    assert_eq!(me["semester"], "2026-27-W");
    let nav: Vec<&str> = me["nav"].as_array().unwrap().iter().map(|n| n["path"].as_str().unwrap()).collect();
    assert!(nav.contains(&"/inspect/students") && !nav.contains(&"/admin/staff"), "{nav:?}");

    let code = uniq("A").to_uppercase()[..10].to_string();
    let course = json!({ "code": code, "title": "Алгоритми", "ects_credits": 6, "faculty_code": fac });
    assert_eq!(inspector.api("POST", "/inspect/courses", Some(&course)).await.status, 201);
    let dup = inspector.api("POST", "/inspect/courses", Some(&course)).await;
    assert_eq!((dup.status, dup.json()["error"].as_str()), (409, Some("conflict")));
    let bad = inspector.api("POST", "/inspect/courses", Some(&json!({ "code": "x" }))).await;
    assert_eq!(
        (bad.status, bad.json()["error"].as_str()),
        (400, Some("invalid")),
        "a malformed body is a PortalError"
    );

    let offering = json!({ "course_code": code, "semester": "2026-27-W", "teacher_sub": teacher.to_string(), "capacity": 1 });
    assert_eq!(inspector.api("POST", "/inspect/offerings", Some(&offering)).await.status, 201);
    let catalogue = inspector.api_get("/inspect/courses").await.json();
    let off_id = catalogue["offerings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["code"] == code.as_str())
        .unwrap()["id"]
        .as_i64()
        .unwrap();

    let spec = inspector.api_get("/options").await.json()["specialties"][0]["id"].as_i64().unwrap();
    let student_email = format!("{}@test.tu-sofia.bg", uniq("n"));
    let created = inspector
        .api(
            "POST",
            "/inspect/students",
            Some(&json!({ "email": student_email, "first_name": "Нова", "last_name": "Студентка", "faculty_number": uniq("8"), "specialty_id": spec, "study_year": 1, "password": "first-password" })),
        )
        .await;
    assert_eq!(created.status, 201, "{}", created.text);
    let student_sub = created.json()["sub"].as_str().unwrap().to_string();
    let found = inspector.api_get(&format!("/inspect/students?q={student_email}")).await.json();
    assert_eq!(found[0]["sub"], student_sub.as_str());

    // The new student must change the first password before using the API
    let student = signed_in(&p, &student_email, "first-password").await;
    assert_eq!(student.api_get("/courses").await.json()["error"], "password_change_required");
    let page = student.get("/password").await;
    student
        .post_form(
            "/password",
            &[
                ("csrf", &csrf_from(&page.text)),
                ("password", "first-password"),
                ("new_password", "my-new-password"),
                ("new_password2", "my-new-password"),
            ],
        )
        .await;

    // Student: CSRF header required, then enroll; the only place is taken
    let enroll_path = format!("/courses/{off_id}/enroll");
    let no_csrf = student.api_with_csrf("POST", &enroll_path, None, "wrong").await;
    assert_eq!((no_csrf.status, no_csrf.json()["error"].as_str()), (403, Some("forbidden")));
    let ok = student.api("POST", &enroll_path, None).await;
    assert_eq!((ok.status, ok.json()["outcome"].as_str()), (200, Some("ENROLLED")));
    let again = student.api("POST", &enroll_path, None).await;
    assert_eq!((again.status, again.json()["outcome"].as_str()), (409, Some("ALREADY_ENROLLED")));
    let listed = student.api_get(&format!("/courses?q={}", code.to_lowercase())).await.json();
    assert_eq!(listed[0]["my_status"], "ENROLLED");
    assert_eq!(listed[0]["enrolled"], 1);
    let mine = student.api_get("/enrollments").await.json();
    let enrollment_id = mine[0]["id"].as_i64().unwrap();
    assert_eq!(student.api_get("/profile").await.json()["sub"], student_sub.as_str());

    // Teacher: own roster only, and reject frees the place
    let t = signed_in(&p, &te, &tp).await;
    let roster = t.api_get(&format!("/teach/offerings/{off_id}")).await.json();
    assert_eq!(roster["roster"][0]["student_sub"], student_sub.as_str());
    assert_eq!(
        t.api_get(&format!("/students/{student_sub}")).await.status,
        200,
        "an enrolled student's profile"
    );
    assert_eq!(inspector.api_get(&format!("/teach/offerings/{off_id}")).await.status, 403);
    assert_eq!(
        t.api("POST", &format!("/teach/enrollments/{enrollment_id}/reject"), None)
            .await
            .status,
        200
    );
    assert_eq!(
        t.api("POST", &format!("/teach/enrollments/{enrollment_id}/reject"), None)
            .await
            .status,
        409
    );
    let rejected = student.api("POST", &enroll_path, None).await;
    assert_eq!(rejected.json()["outcome"], "REJECTED");

    // Dean: the faculty overview includes the new offering
    let dean = signed_in(&p, &de, &dp).await;
    let overview = dean.api_get("/faculty").await.json();
    assert_eq!(overview["faculty"]["code"], fac.as_str());
    assert!(overview["offerings"].as_array().unwrap().iter().any(|o| o["code"] == code.as_str()));

    // Inspector: status change and closing the offering
    let status = inspector
        .api(
            "PUT",
            &format!("/inspect/students/{student_sub}/status"),
            Some(&json!({ "status": "INTERRUPTED" })),
        )
        .await;
    assert_eq!(status.status, 204);
    let bad_status = inspector
        .api(
            "PUT",
            &format!("/inspect/students/{student_sub}/status"),
            Some(&json!({ "status": "ASLEEP" })),
        )
        .await;
    assert_eq!(bad_status.status, 400);
    let toggled = inspector
        .api("POST", &format!("/inspect/offerings/{off_id}/toggle"), None)
        .await
        .json();
    assert_eq!(toggled["enabled"], false);
}

#[tokio::test]
async fn admin_manages_staff_and_modules() {
    let p = start(Environment::Staging, &[]).await;
    let tools = Browser::new(&p.base);
    let (admin_id, ae, ap) = staging_staff(&tools, "ADMIN", None).await;
    let admin = signed_in(&p, &ae, &ap).await;

    let new = json!({ "email": format!("{}@test.tu-sofia.bg", uniq("k")), "first_name": "Нов", "last_name": "Преподавател", "role": "TEACHER", "faculty_code": "FCST", "password": "first-password" });
    let created = admin.api("POST", "/admin/staff", Some(&new)).await;
    assert_eq!(created.status, 201, "{}", created.text);
    let sub = created.json()["sub"].as_str().unwrap().to_string();
    let staff = admin.api_get("/admin/staff").await.json();
    assert!(
        staff
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["sub"] == sub.as_str() && s["enabled"] == true)
    );
    let off = admin.api("POST", &format!("/admin/staff/{sub}/toggle"), None).await.json();
    assert_eq!(off["enabled"], false);
    assert_eq!(
        admin.api("POST", &format!("/admin/staff/{admin_id}/toggle"), None).await.status,
        400,
        "not yourself"
    );

    let id = uniq("m-");
    let reg = admin
        .api(
            "POST",
            "/admin/modules",
            Some(&json!({ "clientId": id, "name": "Модул", "redirectUris": [CB], "directory": true })),
        )
        .await;
    assert_eq!(reg.status, 201, "{}", reg.text);
    assert_eq!(reg.headers.get("cache-control").unwrap(), "no-store");
    assert_eq!(reg.json()["clientSecret"].as_str().unwrap().len(), 43);
    let modules = admin.api_get("/admin/modules").await.json();
    assert!(
        modules
            .as_array()
            .unwrap()
            .iter()
            .any(|m| m["client_id"] == id.as_str() && m["directory"] == true)
    );
    let reserved = admin
        .api(
            "POST",
            "/admin/modules",
            Some(&json!({ "clientId": "unicloud-api", "name": "x", "redirectUris": [CB] })),
        )
        .await;
    assert_eq!(reserved.status, 400);
}

#[tokio::test]
async fn sign_in_returns_to_the_asked_app_page_only() {
    let p = start(Environment::Staging, &[]).await;
    let b = Browser::new(&p.base);
    let (_, email, pw) = staging_student(&b).await;
    assert_eq!(b.me().await.status, 401, "JSON, not a redirect");

    let page = b.get("/login?next=/app/courses").await;
    assert!(page.text.contains(r#"name="next" value="/app/courses""#));
    let csrf = csrf_from(&page.text);
    let r = b
        .post_form(
            "/login",
            &[("csrf", &csrf), ("email", &email), ("password", &pw), ("next", "/app/courses")],
        )
        .await;
    assert_eq!(r.location(), "/app/courses");

    for evil in ["https://evil.example/", "//evil.example/app/", "/app/\\evil", "/staging"] {
        let other = Browser::new(&p.base);
        let csrf = csrf_from(&other.get("/login").await.text);
        let r = other
            .post_form("/login", &[("csrf", &csrf), ("email", &email), ("password", &pw), ("next", evil)])
            .await;
        assert_eq!(r.location(), "/app/", "{evil} is ignored");
    }
    assert_eq!(b.get("/").await.location(), "/app/", "signed in: / opens the app");
}

#[tokio::test]
async fn app_and_library_are_served() {
    // Uses the built app and package when they exist (`npm run build && npm run pack:ui` in web/)
    let p = start(Environment::Production, &[]).await;
    let b = Browser::new(&p.base);
    let missing = b.get("/app/assets/no-such-file.js").await;
    assert_eq!(missing.status, 404, "a missing asset never falls back to index.html");
    assert_ne!(missing.headers.get("cache-control").unwrap(), "public, max-age=31536000, immutable");
    let lib = b.get("/ui/v1/unicloud.css").await;
    assert_eq!(lib.headers.get("access-control-allow-origin").unwrap(), "*");
    if std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/web/apps/portal/dist/index.html")).exists() {
        let deep = b.get("/app/courses").await;
        assert_eq!(deep.status, 200);
        assert!(deep.text.contains(r#"<div id="root">"#), "{}", deep.text);
        assert_eq!(deep.headers.get("cache-control").unwrap(), "no-cache");
    }
}
