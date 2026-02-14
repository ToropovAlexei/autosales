use shared_dtos::permission::PermissionAdminResponse;

use crate::models::permission::PermissionRow;

impl From<PermissionRow> for PermissionAdminResponse {
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

        let response: PermissionAdminResponse = row.into();

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

        let response: PermissionAdminResponse = row.into();

        assert_eq!(response.id, 2);
        assert_eq!(response.name, "another_permission");
        assert_eq!(response.group, "another_group");
        assert_eq!(response.description, None);
    }
}
