use chrono::{DateTime, Utc};
use sqlx::prelude::FromRow;

#[derive(FromRow, Debug)]
pub struct UserRoleRow {
    pub user_id: i64,
    pub role_id: i64,
    pub created_at: DateTime<Utc>,
    pub created_by: i64,
}

#[derive(Debug)]
pub struct NewUserRole {
    pub user_id: i64,
    pub role_id: i64,
    pub created_by: i64,
}

#[derive(Debug)]
pub struct AssignUserRoles {
    pub user_id: i64,
    pub roles: Vec<i64>,
    pub created_by: i64,
}
