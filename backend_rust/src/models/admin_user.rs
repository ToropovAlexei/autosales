use chrono::{DateTime, Utc};
use sqlx::prelude::FromRow;

#[derive(Debug, FromRow)]
pub struct AdminUserRow {
    pub id: i64,
    pub login: String,
    pub hashed_password: String,
    pub two_fa_secret: Option<String>,
    pub telegram_id: Option<i64>,
    pub is_system: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_by: i64,
}

#[derive(Debug)]
pub struct NewAdminUser {
    pub login: String,
    pub hashed_password: String,
    pub two_fa_secret: Option<String>,
    pub telegram_id: Option<i64>,
    pub created_by: i64,
}

#[derive(Debug)]
pub struct UpdateAdminUser {
    pub login: Option<String>,
    pub hashed_password: Option<String>,
    pub two_fa_secret: Option<String>,
    pub telegram_id: Option<i64>,
}
