use chrono::{DateTime, Utc};
use sqlx::prelude::FromRow;

#[derive(FromRow, Debug)]
pub struct RolePermissionRow {
    pub role_id: i64,
    pub permission_id: i64,
    pub created_at: DateTime<Utc>,
    pub created_by: i64,
}

#[derive(Debug)]
pub struct NewRolePermission {
    pub role_id: i64,
    pub permission_id: i64,
    pub created_by: i64,
}
