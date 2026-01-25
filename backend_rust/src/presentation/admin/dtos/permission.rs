use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::{ToResponse, ToSchema};

use crate::models::permission::PermissionRow;

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema, ToResponse)]
#[ts(export, export_to = "auth.ts", rename = "PermissionResponse")]
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

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;

    #[test]
    fn test_permission_response_from_permission_row_full() {
        let row = PermissionRow {
            id: 1,
            name: "test_permission".to_string(),
            group: "test_group".to_string(),
            description: Some("This is a test permission".to_string()),
            created_at: Utc::now(),
        };

        let response: PermissionResponse = row.into();

        assert_eq!(response.id, 1);
        assert_eq!(response.name, "test_permission");
        assert_eq!(response.group, "test_group");
        assert_eq!(
            response.description,
            Some("This is a test permission".to_string())
        );
    }

    #[test]
    fn test_permission_response_from_permission_row_minimal() {
        let row = PermissionRow {
            id: 2,
            name: "another_permission".to_string(),
            group: "another_group".to_string(),
            description: None,
            created_at: Utc::now(),
        };

        let response: PermissionResponse = row.into();

        assert_eq!(response.id, 2);
        assert_eq!(response.name, "another_permission");
        assert_eq!(response.group, "another_group");
        assert_eq!(response.description, None);
    }
}
