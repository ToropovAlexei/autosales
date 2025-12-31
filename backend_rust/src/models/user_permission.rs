use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use ts_rs::TS;
use utoipa::ToSchema;

#[derive(sqlx::Type, Debug, Clone, Copy, PartialEq, Deserialize, Serialize, TS, ToSchema)]
#[sqlx(type_name = "permission_effect", rename_all = "snake_case")]
pub enum PermissionEffect {
    Allow,
    Deny,
}

#[derive(FromRow, Debug)]
pub struct UserPermissionRow {
    pub user_id: i64,
    pub permission_id: i64,
    pub effect: PermissionEffect,
    pub created_at: DateTime<Utc>,
    pub created_by: i64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct NewUserPermission {
    pub user_id: i64,
    pub permission_id: i64,
    pub effect: PermissionEffect,
    pub created_by: i64,
}

#[derive(Debug)]
pub struct UpdateUserPermission {
    pub id: i64,
    pub effect: Option<PermissionEffect>,
}

#[derive(Debug)]
pub struct UpsertUserPermission {
    pub id: i64,
    pub effect: PermissionEffect,
}

#[derive(Debug)]
pub struct UpdateUserPermissions {
    pub user_id: i64,
    pub removed: Vec<i64>,
    pub upserted: Vec<UpsertUserPermission>,
    pub created_by: i64,
}
