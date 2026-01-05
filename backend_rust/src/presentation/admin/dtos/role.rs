use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::{ToResponse, ToSchema};
use validator::Validate;

use crate::models::role::RoleRow;

#[derive(Debug, Clone, Serialize, Deserialize, Validate, TS, ToSchema, ToResponse)]
#[ts(export, export_to = "auth.ts", rename = "NewRole")]
pub struct NewRoleRequest {
    #[validate(length(
        min = 3,
        max = 255,
        message = "Role name must be at least 3 characters long and at most 255 characters long"
    ))]
    pub name: String,
    #[ts(optional)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, TS, ToSchema, ToResponse)]
#[ts(export, export_to = "auth.ts", rename = "UpdateRole")]
pub struct UpdateRoleRequest {
    #[validate(length(
        min = 3,
        max = 255,
        message = "Role name must be at least 3 characters long and at most 255 characters long"
    ))]
    #[ts(optional)]
    pub name: Option<String>,
    #[ts(optional)]
    pub description: Option<Option<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, TS, ToSchema, ToResponse)]
#[ts(export, export_to = "auth.ts", rename = "Role")]
pub struct RoleResponse {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: i64,
}

impl From<RoleRow> for RoleResponse {
    fn from(role: RoleRow) -> Self {
        Self {
            id: role.id,
            name: role.name,
            description: role.description,
            created_at: role.created_at,
            updated_at: role.updated_at,
            created_by: role.created_by,
        }
    }
}
