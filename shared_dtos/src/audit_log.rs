use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(feature = "sqlx", sqlx(type_name = "TEXT", rename_all = "snake_case"))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export, export_to = "audit_log.ts"))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
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
    BroadcastCreate,
    BroadcastUpdate,
}

#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(feature = "sqlx", sqlx(type_name = "TEXT", rename_all = "snake_case"))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export, export_to = "audit_log.ts"))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditStatus {
    Success,
    Failed,
    Denied,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "audit_log.ts", rename = "AuditLog")
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogAdminResponse {
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
