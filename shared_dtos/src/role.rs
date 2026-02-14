use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::rust::double_option;

#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export, export_to = "auth.ts", rename = "NewRole"))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "validate", derive(validator::Validate))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewRoleAdminRequest {
    #[cfg_attr(
        feature = "validate",
        validate(length(
            min = 3,
            max = 255,
            message = "Role name must be at least 3 characters long and at most 255 characters long"
        ))
    )]
    pub name: String,
    #[cfg_attr(feature = "ts", ts(optional))]
    pub description: Option<String>,
}

#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "auth.ts", rename = "UpdateRole")
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "validate", derive(validator::Validate))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRoleAdminRequest {
    #[cfg_attr(
        feature = "validate",
        validate(length(
            min = 3,
            max = 255,
            message = "Role name must be at least 3 characters long and at most 255 characters long"
        ))
    )]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub name: Option<String>,
    #[cfg_attr(feature = "ts", ts(optional))]
    #[cfg_attr(feature = "ts", ts(type = "string | null"))]
    #[serde(default, with = "double_option")]
    pub description: Option<Option<String>>,
}

#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export, export_to = "auth.ts", rename = "Role"))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleAdminResponse {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: i64,
}
