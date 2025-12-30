use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::{ToResponse, ToSchema};

use crate::models::permission::PermissionRow;

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema, ToResponse)]
#[ts(export, export_to = "auth.ts", rename = "NewCategory")]
pub struct PermissionResponse {
    pub id: i64,
    pub name: String,
    pub group: String,
    pub description: Option<String>,
}

impl From<PermissionRow> for PermissionResponse {
    fn from(row: PermissionRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            group: row.group,
            description: row.description,
        }
    }
}
