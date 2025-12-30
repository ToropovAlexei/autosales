use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::{Decode, prelude::FromRow};

#[derive(Debug, Deserialize, Decode)]
pub struct RoleSummaryRow {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, FromRow)]
pub struct AdminUserWithRolesRow {
    pub id: i64,
    pub login: String,
    pub hashed_password: String,
    pub two_fa_secret: String,
    pub telegram_id: Option<i64>,
    pub is_system: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_by: i64,
    #[sqlx(json)]
    pub roles: Vec<RoleSummaryRow>,
}
