use std::net::IpAddr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::ipnetwork::IpNetwork};
use ts_rs::TS;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::define_list_query;

#[derive(sqlx::Type, Debug, Clone, Copy, PartialEq, Deserialize, Serialize, TS, ToSchema)]
#[sqlx(type_name = "audit_action", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "audit_log.ts")]
pub enum AuditAction {
    UserLogin,
    UserLogout,
    UserCreate,
    UserUpdate,
    UserDelete,
    RoleGrant,
    RoleRevoke,
    PermissionGrant,
    PermissionRevoke,
    ProductCreate,
    ProductUpdate,
    ProductDelete,
    ProductHide,
    StockMovementCreate,
    BalanceDeposit,
    BalanceWithdrawal,
    ReferralPayout,
    InvoiceCreate,
    InvoicePay,
    InvoiceExpire,
    CategoryCreate,
    CategoryUpdate,
    CategoryDelete,
    CustomerCreate,
    CustomerUpdate,
    CustomerDelete,
    BotCreate,
    BotUpdate,
    BotDelete,
    ImageCreate,
    ImageUpdate,
    ImageDelete,
    SystemSettingsUpdate,
}

#[derive(sqlx::Type, Debug, Clone, Copy, PartialEq, Deserialize, Serialize, TS, ToSchema)]
#[sqlx(type_name = "audit_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "audit_log.ts")]
pub enum AuditStatus {
    Success,
    Failed,
    Denied,
}

#[derive(Debug, FromRow)]
pub struct AuditLogRow {
    pub id: i64,
    pub admin_user_id: Option<i64>,
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
