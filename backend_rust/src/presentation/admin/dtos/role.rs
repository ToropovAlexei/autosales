use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::rust::double_option;
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
    #[ts(type = "string | null")]
    #[serde(default, with = "double_option")]
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

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn test_new_role_request_validation() {
        // Valid data
        let req = NewRoleRequest {
            name: "Valid Role".to_string(),
            description: Some("A test role".to_string()),
        };
        assert!(req.validate().is_ok());

        // Name too short
        let req = NewRoleRequest {
            name: "ab".to_string(),
            description: None,
        };
        assert!(req.validate().is_err());

        // Name too long
        let req = NewRoleRequest {
            name: "a".repeat(256),
            description: None,
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_update_role_request_validation() {
        // Valid: All optional fields are None
        let req = UpdateRoleRequest {
            name: None,
            description: None,
        };
        assert!(req.validate().is_ok());

        // Valid: All fields provided and correct
        let req = UpdateRoleRequest {
            name: Some("Updated Role".to_string()),
            description: Some(Some("Updated description".to_string())),
        };
        assert!(req.validate().is_ok());

        // Valid: Setting description to None
        let req = UpdateRoleRequest {
            name: Some("Updated Role".to_string()),
            description: Some(None),
        };
        assert!(req.validate().is_ok());

        // Name too short
        let req = UpdateRoleRequest {
            name: Some("ab".to_string()),
            description: None,
        };
        assert!(req.validate().is_err());

        // Name too long
        let req = UpdateRoleRequest {
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

        let response: RoleResponse = row.into();

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

        let response: RoleResponse = row.into();

        assert_eq!(response.id, 2);
        assert_eq!(response.name, "User Role");
        assert_eq!(response.description, None);
        assert_eq!(response.created_at, now);
        assert_eq!(response.updated_at, now);
        assert_eq!(response.created_by, 1);
    }
}
