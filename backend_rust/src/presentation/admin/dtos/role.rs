use shared_dtos::role::RoleAdminResponse;

use crate::models::role::RoleRow;

impl From<RoleRow> for RoleAdminResponse {
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use shared_dtos::role::{NewRoleAdminRequest, UpdateRoleAdminRequest};
    use validator::Validate;

    #[test]
    fn test_new_role_request_validation() {
        // Valid data
        let req = NewRoleAdminRequest {
            name: "Valid Role".to_string(),
            description: Some("A test role".to_string()),
        };
        assert!(req.validate().is_ok());

        // Name too short
        let req = NewRoleAdminRequest {
            name: "ab".to_string(),
            description: None,
        };
        assert!(req.validate().is_err());

        // Name too long
        let req = NewRoleAdminRequest {
            name: "a".repeat(256),
            description: None,
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_update_role_request_validation() {
        // Valid: All optional fields are None
        let req = UpdateRoleAdminRequest {
            name: None,
            description: None,
        };
        assert!(req.validate().is_ok());

        // Valid: All fields provided and correct
        let req = UpdateRoleAdminRequest {
            name: Some("Updated Role".to_string()),
            description: Some(Some("Updated description".to_string())),
        };
        assert!(req.validate().is_ok());

        // Valid: Setting description to None
        let req = UpdateRoleAdminRequest {
            name: Some("Updated Role".to_string()),
            description: Some(None),
        };
        assert!(req.validate().is_ok());

        // Name too short
        let req = UpdateRoleAdminRequest {
            name: Some("ab".to_string()),
            description: None,
        };
        assert!(req.validate().is_err());

        // Name too long
        let req = UpdateRoleAdminRequest {
            name: Some("a".repeat(256)),
            description: None,
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_role_response_from_role_row_full() {
        let now = Utc::now();
        let row = RoleRow {
            id: 1,
            name: "Admin Role".to_string(),
            description: Some("Administrator privileges".to_string()),
            created_at: now,
            updated_at: now,
            created_by: 1,
        };

        let response: RoleAdminResponse = row.into();

        assert_eq!(response.id, 1);
        assert_eq!(response.name, "Admin Role");
        assert_eq!(
            response.description,
            Some("Administrator privileges".to_string())
        );
        assert_eq!(response.created_at, now);
        assert_eq!(response.updated_at, now);
        assert_eq!(response.created_by, 1);
    }

    #[test]
    fn test_role_response_from_role_row_minimal() {
        let now = Utc::now();
        let row = RoleRow {
            id: 2,
            name: "User Role".to_string(),
            description: None,
            created_at: now,
            updated_at: now,
            created_by: 1,
        };

        let response: RoleAdminResponse = row.into();

        assert_eq!(response.id, 2);
        assert_eq!(response.name, "User Role");
        assert_eq!(response.description, None);
        assert_eq!(response.created_at, now);
        assert_eq!(response.updated_at, now);
        assert_eq!(response.created_by, 1);
    }
}
