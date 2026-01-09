use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::{ToResponse, ToSchema};

use crate::models::audit_log::{AuditAction, AuditLogRow, AuditStatus};

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema, ToResponse)]
#[ts(export, export_to = "audit_log.ts", rename = "AuditLog")]
pub struct AuditLogResponse {
    pub id: i64,
    pub admin_user_id: Option<i64>,
    pub admin_user_login: Option<String>,
    pub customer_id: Option<i64>,
    pub action: AuditAction,
    pub status: AuditStatus,
    pub target_table: String,
    pub target_id: String,
    pub old_values: Option<serde_json::Value>,
    pub new_values: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_id: Option<String>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<AuditLogRow> for AuditLogResponse {
    fn from(r: AuditLogRow) -> Self {
        AuditLogResponse {
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
