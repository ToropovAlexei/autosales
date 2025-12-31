use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::{ToResponse, ToSchema};
use validator::Validate;

use crate::models::user_permission::PermissionEffect;

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema, ToResponse)]
#[ts(export, export_to = "auth.ts", rename = "UpdateUserPermission")]
pub struct UpdateUserPermissionRequest {
    pub id: i64,
    pub effect: Option<PermissionEffect>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema, ToResponse)]
#[ts(export, export_to = "auth.ts", rename = "UpsertUserPermission")]
pub struct UpsertUserPermissionRequest {
    pub id: i64,
    pub effect: PermissionEffect,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema, ToResponse, Validate)]
#[ts(export, export_to = "auth.ts", rename = "UpdateUserPermissions")]
pub struct UpdateUserPermissionsRequest {
    pub removed: Vec<i64>,
    pub upserted: Vec<UpsertUserPermissionRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema, ToResponse)]
#[ts(export, export_to = "auth.ts", rename = "UserPermission")]
pub struct UserPermissionResponse {
    pub id: i64,
    pub effect: PermissionEffect,
}
