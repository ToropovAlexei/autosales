use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "auth.ts", rename = "PermissionResponse")
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionAdminResponse {
    pub id: i64,
    pub name: String,
    pub group: String,
    pub description: Option<String>,
}
