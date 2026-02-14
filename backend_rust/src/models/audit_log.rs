use std::net::IpAddr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_dtos::audit_log::{AuditAction, AuditStatus};
use sqlx::{prelude::FromRow, types::ipnetwork::IpNetwork};
use uuid::Uuid;

use crate::define_list_query;

#[derive(Debug, FromRow)]
pub struct AuditLogRow {
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
    pub ip_address: Option<IpNetwork>,
    pub user_agent: Option<String>,
    pub request_id: Option<String>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct NewAuditLog {
    pub admin_user_id: Option<i64>,
    pub customer_id: Option<i64>,
    pub action: AuditAction,
    pub status: AuditStatus,
    pub target_table: String,
    pub target_id: String,
    pub old_values: Option<serde_json::Value>,
    pub new_values: Option<serde_json::Value>,
    pub ip_address: Option<IpAddr>,
    pub user_agent: Option<String>,
    pub request_id: Option<Uuid>,
    pub error_message: Option<String>,
}

define_list_query! {
    query_name: AuditLogListQuery,
    filter_fields: {
        AuditLogFilterFields,
        [
            Id => "id",
            AdminUserId => "admin_user_id",
            CustomerId => "customer_id",
            Action => "action",
            Status => "status",
            TargetTable => "target_table",
            TargetId => "target_id",
            IpAddress => "ip_address",
            UserAgent => "user_agent",
            RequestId => "request_id",
            ErrorMessage => "error_message",
            CreatedAt => "created_at",
        ]
    },
    order_fields: {
        AuditLogOrderFields,
        [
            Id => "id",
            AdminUserId => "admin_user_id",
            CustomerId => "customer_id",
            Action => "action",
            Status => "status",
            TargetTable => "target_table",
            TargetId => "target_id",
            IpAddress => "ip_address",
            UserAgent => "user_agent",
            RequestId => "request_id",
            ErrorMessage => "error_message",
            CreatedAt => "created_at",
        ]
    }
}
