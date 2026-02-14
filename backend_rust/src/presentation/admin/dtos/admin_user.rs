use shared_dtos::admin_user::{
    AdminUserAdminResponse, AdminUserWithRolesAdminResponse, RoleSummaryAdminResponse,
};

use crate::models::{admin_user::AdminUserRow, admin_user_with_roles::AdminUserWithRolesRow};

impl From<AdminUserRow> for AdminUserAdminResponse {
    fn from(r: AdminUserRow) -> Self {
        AdminUserAdminResponse {
            id: r.id,
            login: r.login,
            telegram_id: r.telegram_id,
            deleted_at: r.deleted_at,
            created_by: r.created_by,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

impl From<AdminUserWithRolesRow> for AdminUserWithRolesAdminResponse {
    fn from(r: AdminUserWithRolesRow) -> Self {
        AdminUserWithRolesAdminResponse {
            id: r.id,
            login: r.login,
            telegram_id: r.telegram_id,
            deleted_at: r.deleted_at,
            created_by: r.created_by,
            created_at: r.created_at,
            updated_at: r.updated_at,
            roles: r
                .roles
                .iter()
                .map(|r| RoleSummaryAdminResponse {
                    id: r.id,
                    name: r.name.clone(),
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use shared_dtos::admin_user::{NewAdminUserAdminRequest, UpdateAdminUserAdminRequest};
    use validator::Validate;

    #[test]
    fn test_new_admin_user_request_validation() {
        // Valid data
        let req = NewAdminUserAdminRequest {
            login: "goodlogin".to_string(),
            password: "goodpassword".to_string(),
            roles: vec![1],
        };
        assert!(req.validate().is_ok());

        // Invalid login (too short)
        let req = NewAdminUserAdminRequest {
            login: "a".to_string(),
            password: "goodpassword".to_string(),
            roles: vec![1],
        };
        assert!(req.validate().is_err());

        // Invalid password (too long)
        let req = NewAdminUserAdminRequest {
            login: "goodlogin".to_string(),
            password: "a".repeat(21),
            roles: vec![1],
        };
        assert!(req.validate().is_err());

        // Invalid roles (empty)
        let req = NewAdminUserAdminRequest {
            login: "goodlogin".to_string(),
            password: "goodpassword".to_string(),
            roles: vec![],
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_update_admin_user_request_validation() {
        // Valid: All optional fields are None
        let req = UpdateAdminUserAdminRequest {
            login: None,
            password: None,
            telegram_id: None,
            roles: None,
        };
        assert!(req.validate().is_ok());

        // Valid: All fields provided and correct
        let req = UpdateAdminUserAdminRequest {
            login: Some("newlogin".to_string()),
            password: Some("newpassword".to_string()),
            telegram_id: Some(12345),
            roles: Some(vec![1, 2]),
        };
        assert!(req.validate().is_ok());

        // Invalid login (too short)
        let req = UpdateAdminUserAdminRequest {
            login: Some("a".to_string()),
            password: None,
            telegram_id: None,
            roles: None,
        };
        assert!(req.validate().is_err());

        // Invalid password (too long)
        let req = UpdateAdminUserAdminRequest {
            login: None,
            password: Some("a".repeat(21)),
            telegram_id: None,
            roles: None,
        };
        assert!(req.validate().is_err());

        // Invalid roles (empty vec)
        let req = UpdateAdminUserAdminRequest {
            login: None,
            password: None,
            telegram_id: None,
            roles: Some(vec![]),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_from_admin_user_row_for_admin_user_response() {
        let now = Utc::now();
        let row = AdminUserRow {
            id: 1,
            login: "test_user".to_string(),
            hashed_password: "hash".to_string(),
            two_fa_secret: "secret".to_string(),
            telegram_id: Some(123),
            is_system: false,
            deleted_at: None,
            created_at: now,
            updated_at: now,
            created_by: 1,
        };

        let response: AdminUserAdminResponse = row.into();

        assert_eq!(response.id, 1);
        assert_eq!(response.login, "test_user");
        assert_eq!(response.telegram_id, Some(123));
        assert_eq!(response.created_at, now);
    }

    #[test]
    fn test_from_admin_user_with_roles_row_for_admin_user_with_roles_response() {
        let now = Utc::now();
        let row = AdminUserWithRolesRow {
            id: 1,
            login: "test_user".to_string(),
            hashed_password: "hash".to_string(),
            two_fa_secret: "secret".to_string(),
            is_system: false,
            telegram_id: Some(123),
            deleted_at: None,
            created_at: now,
            updated_at: now,
            created_by: 1,
            roles: serde_json::from_str(
                r#"[{"id": 1, "name": "Admin"}, {"id": 2, "name": "User"}]"#,
            )
            .unwrap(),
        };

        let response: AdminUserWithRolesAdminResponse = row.into();

        assert_eq!(response.id, 1);
        assert_eq!(response.login, "test_user");
        assert_eq!(response.roles.len(), 2);
        assert_eq!(response.roles[0].name, "Admin");
        assert_eq!(response.roles[1].id, 2);
    }
}
