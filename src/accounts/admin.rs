//! Creating accounts. Inspectors create students, admins create staff. There is no public sign-up.

use sqlx::PgPool;

use super::password;
use crate::access::Role;
use crate::error::is_unique_violation;

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
