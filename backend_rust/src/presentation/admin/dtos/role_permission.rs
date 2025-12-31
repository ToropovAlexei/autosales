use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::{ToResponse, ToSchema};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate, TS, ToSchema, ToResponse)]
#[ts(export, export_to = "auth.ts", rename = "UpdateRolePermissions")]
pub struct UpdateRolePermissionsRequest {
    pub added: Vec<i64>,
    pub removed: Vec<i64>,
}
