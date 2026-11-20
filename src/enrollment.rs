//! Enrollment rules (GLOSSARY: Enrollment). A request takes a free place at once, in one statement,
//! so a rush at 08:00 cannot oversell. The teacher may reject, which frees the place.

use sqlx::PgPool;

#[derive(Debug, PartialEq, Eq)]
pub enum EnrollOutcome {
    Enrolled,
    AlreadyEnrolled,
    Full,
    Closed,
    Rejected,
    NotActive,
    NoSuchOffering,
}

impl EnrollOutcome {
    pub fn message_bg(&self) -> &'static str {
        match self {
            EnrollOutcome::Enrolled => "Записахте се успешно.",
            EnrollOutcome::AlreadyEnrolled => "Вече сте записани в този курс.",
            EnrollOutcome::Full => "Няма свободни места.",
            EnrollOutcome::Closed => "Записването за този курс е затворено.",
            EnrollOutcome::Rejected => "Преподавателят е отхвърлил записването ви в този курс.",
            EnrollOutcome::NotActive => "Само действащи студенти могат да се записват.",
            EnrollOutcome::NoSuchOffering => "Курсът не съществува.",
        }
    }
}

/// Enrolls in an offering of `semester` (the current one): the catalogue hides other semesters,
/// and a request for a hidden offering is refused as closed
pub async fn enroll(db: &PgPool, student_id: i64, offering_id: i64, semester: &str) -> sqlx::Result<EnrollOutcome> {
    let mut tx = db.begin().await?;
    let status: Option<(String,)> = sqlx::query_as("SELECT status FROM academic_records WHERE user_id = $1")
        .bind(student_id)
        .fetch_optional(&mut *tx)
        .await?;
    if status.as_ref().map(|s| s.0.as_str()) != Some("ACTIVE") {
        return Ok(EnrollOutcome::NotActive);
    }
    // Lock this student's row for the offering (if any) so a double click cannot take two places
    let existing: Option<(String,)> =
        sqlx::query_as("SELECT status FROM enrollments WHERE student_id = $1 AND offering_id = $2 FOR UPDATE")
            .bind(student_id)
            .bind(offering_id)
            .fetch_optional(&mut *tx)
            .await?;
    match existing.as_ref().map(|e| e.0.as_str()) {
        Some("ENROLLED") => return Ok(EnrollOutcome::AlreadyEnrolled),
        Some("REJECTED") => return Ok(EnrollOutcome::Rejected),
        _ => {}
    }
    // Take a place: one conditional UPDATE, safe across replicas
    let took = sqlx::query(
        "UPDATE course_offerings SET enrolled_count = enrolled_count + 1 WHERE id = $1 AND is_open AND semester = $2 AND enrolled_count < capacity",
    )
    .bind(offering_id)
    .bind(semester)
    .execute(&mut *tx)
    .await?
    .rows_affected();
    if took == 0 {
        let o: Option<(bool, i32, i32)> =
            sqlx::query_as("SELECT is_open AND semester = $2, enrolled_count, capacity FROM course_offerings WHERE id = $1")
                .bind(offering_id)
                .bind(semester)
                .fetch_optional(&mut *tx)
                .await?;
        return Ok(match o {
            None => EnrollOutcome::NoSuchOffering,
            Some((false, _, _)) => EnrollOutcome::Closed,
            Some(_) => EnrollOutcome::Full,
        });
    }
    let res = if existing.is_some() {
        sqlx::query(
            "UPDATE enrollments SET status = 'ENROLLED', created_at = now(), decided_by = NULL WHERE student_id = $1 AND offering_id = $2",
        )
        .bind(student_id)
        .bind(offering_id)
        .execute(&mut *tx)
        .await
    } else {
        sqlx::query("INSERT INTO enrollments (student_id, offering_id) VALUES ($1, $2)")
            .bind(student_id)
            .bind(offering_id)
            .execute(&mut *tx)
            .await
    };
    match res {
        Ok(_) => {}
        // A parallel first request from the same student won: give the place back
        Err(e) if crate::error::is_unique_violation(&e) => return Ok(EnrollOutcome::AlreadyEnrolled),
        Err(e) => return Err(e),
    }
    tx.commit().await?;
    Ok(EnrollOutcome::Enrolled)
}

/// Student cancels own enrollment. Returns false when there was nothing to cancel.
pub async fn cancel(db: &PgPool, student_id: i64, enrollment_id: i64) -> sqlx::Result<bool> {
    change_status(db, enrollment_id, "CANCELLED", None, Some(student_id), None).await
}

/// Teacher rejects an enrollment in an offering they teach
pub async fn reject(db: &PgPool, teacher_id: i64, enrollment_id: i64) -> sqlx::Result<bool> {
    change_status(db, enrollment_id, "REJECTED", Some(teacher_id), None, Some(teacher_id)).await
}

async fn change_status(
    db: &PgPool,
    enrollment_id: i64,
    to: &str,
    decided_by: Option<i64>,
    student: Option<i64>,
    teacher: Option<i64>,
) -> sqlx::Result<bool> {
    let mut tx = db.begin().await?;
    let row: Option<(i64,)> = sqlx::query_as(
        "UPDATE enrollments e SET status = $2, decided_by = $3
         FROM course_offerings o
         WHERE e.id = $1 AND e.offering_id = o.id AND e.status = 'ENROLLED'
           AND ($4::bigint IS NULL OR e.student_id = $4) AND ($5::bigint IS NULL OR o.teacher_id = $5)
         RETURNING e.offering_id",
    )
    .bind(enrollment_id)
    .bind(to)
    .bind(decided_by)
    .bind(student)
    .bind(teacher)
    .fetch_optional(&mut *tx)
    .await?;
    let Some((offering_id,)) = row else { return Ok(false) };
    sqlx::query("UPDATE course_offerings SET enrolled_count = enrolled_count - 1 WHERE id = $1")
        .bind(offering_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(true)
}
