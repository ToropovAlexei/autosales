use chrono::{DateTime, Utc};
use sqlx::prelude::FromRow;

#[derive(FromRow, Debug)]
pub struct RoleRow {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: i64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct NewRole {
    pub name: String,
    pub description: Option<String>,
    pub created_by: i64,
}

#[derive(Debug)]
pub struct UpdateRole {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
}
