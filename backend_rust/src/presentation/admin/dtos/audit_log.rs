use shared_dtos::audit_log::AuditLogAdminResponse;

use crate::models::audit_log::AuditLogRow;

impl From<AuditLogRow> for AuditLogAdminResponse {
    fn from(r: AuditLogRow) -> Self {
        AuditLogAdminResponse {
            id: r.id,
            admin_user_id: r.admin_user_id,
            admin_user_login: r.admin_user_login,
            customer_id: r.customer_id,
            action: r.action,
            status: r.status,
            target_table: r.target_table,
            target_id: r.target_id,
            old_values: r.old_values,
            new_values: r.new_values,
            ip_address: r.ip_address.map(|s| s.to_string()),
            user_agent: r.user_agent,
            request_id: r.request_id,
            error_message: r.error_message,
            created_at: r.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use serde_json::json;
    use shared_dtos::audit_log::{AuditAction, AuditStatus};

    #[test]
    fn test_audit_log_response_from_audit_log_row_full() {
        let now = Utc::now();
        let row = AuditLogRow {
            id: 1,
            admin_user_id: Some(101),
            admin_user_login: Some("admin_test".to_string()),
            customer_id: Some(202),
            action: AuditAction::UserCreate,
            status: AuditStatus::Success,
            target_table: "users".to_string(),
            target_id: "303".to_string(),
            old_values: Some(json!({"field1": "old"})),
            new_values: Some(json!({"field1": "new"})),
            ip_address: Some("192.168.1.1".parse().unwrap()),
            user_agent: Some("TestAgent".to_string()),
            request_id: Some("req123".to_string()),
            error_message: None,
            created_at: now,
        };

        let response: AuditLogAdminResponse = row.into();

        assert_eq!(response.id, 1);
        assert_eq!(response.admin_user_id, Some(101));
        assert_eq!(response.admin_user_login, Some("admin_test".to_string()));
        assert_eq!(response.customer_id, Some(202));
        assert_eq!(response.action, AuditAction::UserCreate);
        assert_eq!(response.status, AuditStatus::Success);
        assert_eq!(response.target_table, "users");
        assert_eq!(response.target_id, "303");
        assert_eq!(response.old_values, Some(json!({"field1": "old"})));
        assert_eq!(response.new_values, Some(json!({"field1": "new"})));
        assert_eq!(response.ip_address, Some("192.168.1.1/32".to_string()));
        assert_eq!(response.user_agent, Some("TestAgent".to_string()));
        assert_eq!(response.request_id, Some("req123".to_string()));
        assert_eq!(response.error_message, None);
        assert_eq!(response.created_at, now);
    }

    #[test]
    fn test_audit_log_response_from_audit_log_row_minimal() {
        let now = Utc::now();
        let row = AuditLogRow {
            id: 2,
            admin_user_id: None,
            admin_user_login: None,
            customer_id: None,
            action: AuditAction::ProductUpdate,
            status: AuditStatus::Failed,
            target_table: "products".to_string(),
            target_id: "404".to_string(),
            old_values: None,
            new_values: None,
            ip_address: None,
            user_agent: None,
            request_id: None,
            error_message: Some("Permission denied".to_string()),
            created_at: now,
        };

        let response: AuditLogAdminResponse = row.into();

        assert_eq!(response.id, 2);
        assert_eq!(response.admin_user_id, None);
        assert_eq!(response.admin_user_login, None);
        assert_eq!(response.customer_id, None);
        assert_eq!(response.action, AuditAction::ProductUpdate);
        assert_eq!(response.status, AuditStatus::Failed);
        assert_eq!(response.target_table, "products");
        assert_eq!(response.target_id, "404");
        assert_eq!(response.old_values, None);
        assert_eq!(response.new_values, None);
        assert_eq!(response.ip_address, None);
        assert_eq!(response.user_agent, None);
        assert_eq!(response.request_id, None);
        assert_eq!(
            response.error_message,
            Some("Permission denied".to_string())
        );
        assert_eq!(response.created_at, now);
    }
}
