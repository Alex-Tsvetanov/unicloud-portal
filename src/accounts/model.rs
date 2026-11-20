//! The user as the rest of the portal sees it.

use sqlx::{FromRow, PgPool};

use crate::access::{self, Permission, Role};

#[derive(Debug, Clone, FromRow)]
pub struct UserRow {
    pub id: i64,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub role: String,
    pub permissions_mask: i64,
    pub faculty_code: Option<String>,
    pub must_change_password: bool,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub role: Role,
    pub mask: i64,
    /// The user's own faculty, or the faculty of a student's specialty
    pub faculty_code: Option<String>,
    pub must_change_password: bool,
}

impl User {
    pub fn can(&self, p: Permission) -> bool {
        access::has(self.mask, p)
    }

    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }
}

pub const USER_COLUMNS: &str = "u.id, u.email, u.first_name, u.last_name, u.role, u.permissions_mask,
     COALESCE(u.faculty_code, s.faculty_code) AS faculty_code, u.must_change_password, u.enabled";

pub const USER_FROM: &str = "users u LEFT JOIN academic_records r ON r.user_id = u.id LEFT JOIN specialties s ON s.id = r.specialty_id";

impl TryFrom<UserRow> for User {
    type Error = anyhow::Error;
    fn try_from(r: UserRow) -> Result<Self, Self::Error> {
        let role = Role::parse(&r.role).ok_or_else(|| anyhow::anyhow!("unknown role {} for user {}", r.role, r.id))?;
        Ok(User {
            id: r.id,
            email: r.email,
            first_name: r.first_name,
            last_name: r.last_name,
            role,
            mask: r.permissions_mask,
            faculty_code: r.faculty_code,
            must_change_password: r.must_change_password,
        })
    }
}

/// An enabled user by id, or None
pub async fn find_enabled(db: &PgPool, id: i64) -> anyhow::Result<Option<User>> {
    let sql = format!("SELECT {USER_COLUMNS} FROM {USER_FROM} WHERE u.id = $1 AND u.enabled");
    let row: Option<UserRow> = sqlx::query_as(sqlx::AssertSqlSafe(sql)).bind(id).fetch_optional(db).await?;
    row.map(User::try_from).transpose()
}
