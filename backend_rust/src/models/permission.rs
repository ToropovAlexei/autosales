use chrono::{DateTime, Utc};
use sqlx::prelude::FromRow;

#[derive(FromRow, Debug)]
pub struct PermissionRow {
    pub id: i64,
    pub name: String,
    pub group: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}
