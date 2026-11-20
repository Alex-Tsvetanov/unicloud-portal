//! User claims by scope, read from the database when a token is issued or `/userinfo` is called.
//! The directory API returns the same shapes.

use serde_json::{Map, Value, json};
use sqlx::{FromRow, PgPool};

use crate::access::names_of;

#[derive(Debug, FromRow)]
pub struct ClaimRow {
    pub id: i64,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub role: String,
    pub permissions_mask: i64,
    pub faculty_code: Option<String>,
    pub faculty_name: Option<String>,
    pub faculty_number: Option<String>,
    pub specialty: Option<String>,
    pub study_year: Option<i16>,
    pub status: Option<String>,
}

pub const CLAIM_SELECT: &str = "SELECT u.id, u.email, u.first_name, u.last_name, u.role, u.permissions_mask,
        f.code AS faculty_code, f.name AS faculty_name,
        r.faculty_number, s.name AS specialty, r.study_year, r.status
    FROM users u
    LEFT JOIN academic_records r ON r.user_id = u.id
    LEFT JOIN specialties s ON s.id = r.specialty_id
    LEFT JOIN faculties f ON f.code = COALESCE(u.faculty_code, s.faculty_code)";

/// An enabled user's claim data, or None when the user is disabled or gone
pub async fn load(db: &PgPool, user_id: i64) -> sqlx::Result<Option<ClaimRow>> {
    sqlx::query_as(sqlx::AssertSqlSafe(format!("{CLAIM_SELECT} WHERE u.id = $1 AND u.enabled")))
        .bind(user_id)
        .fetch_optional(db)
        .await
}

impl ClaimRow {
    pub fn faculty(&self) -> Option<Value> {
        Some(json!({ "code": self.faculty_code.as_ref()?, "name": self.faculty_name.as_ref()? }))
    }

    pub fn student(&self) -> Option<Value> {
        Some(json!({
            "faculty_number": self.faculty_number.as_ref()?,
            "specialty": self.specialty.as_ref()?,
            "study_year": self.study_year?,
            "status": self.status.as_ref()?,
        }))
    }

    pub fn name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }

    /// The claims of the granted scopes only. `sub` is always present.
    pub fn for_scopes(&self, scopes: &[&str]) -> Map<String, Value> {
        let mut m = Map::new();
        m.insert("sub".into(), Value::String(self.id.to_string()));
        if scopes.contains(&"email") {
            m.insert("email".into(), json!(self.email));
        }
        if scopes.contains(&"profile") {
            m.insert("given_name".into(), json!(self.first_name));
            m.insert("family_name".into(), json!(self.last_name));
            m.insert("name".into(), json!(self.name()));
        }
        if scopes.contains(&"unicloud") {
            m.insert("role".into(), json!(self.role));
            m.insert("permissions".into(), json!(names_of(self.permissions_mask)));
            if let Some(f) = self.faculty() {
                m.insert("faculty".into(), f);
            }
            if let Some(s) = self.student() {
                m.insert("student".into(), s);
            }
        }
        m
    }
}
