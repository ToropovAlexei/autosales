use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(feature = "sqlx", sqlx(type_name = "TEXT", rename_all = "snake_case"))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export, export_to = "balance_request.ts"))]
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StoreBalanceRequestType {
    Withdrawal,
    Deposit,
}

#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(feature = "sqlx", sqlx(type_name = "TEXT", rename_all = "snake_case"))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export, export_to = "balance_request.ts"))]
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StoreBalanceRequestStatus {
    PendingOperator,
    Completed,
    Rejected,
    Canceled,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "validate", derive(validator::Validate))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(
        export,
        export_to = "balance_request.ts",
        rename = "CreateBalanceRequest"
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateStoreBalanceRequestAdminRequest {
    pub request_type: StoreBalanceRequestType,
    #[cfg_attr(feature = "validate", validate(length(min = 1, max = 255)))]
    pub wallet_address: String,
    #[cfg_attr(
        feature = "validate",
        validate(range(min = 0.000001, max = 1_000_000.0))
    )]
    pub amount_rub: f64,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "balance_request.ts", rename = "BalanceRequest")
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreBalanceRequestAdminResponse {
    pub id: i64,
    pub request_type: StoreBalanceRequestType,
    pub wallet_address: String,
    pub amount_usdt: f64,
    pub fx_rate_rub_to_usdt: f64,
    pub amount_rub: f64,
    pub status: StoreBalanceRequestStatus,
    pub operator_tg_user_id: Option<i64>,
    pub operator_comment: Option<String>,
    pub operator_action_at: Option<DateTime<Utc>>,
    pub telegram_message_id: Option<i64>,
    pub telegram_chat_id: Option<i64>,
    pub debit_transaction_id: Option<i64>,
    pub credit_transaction_id: Option<i64>,
    pub refund_transaction_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "validate", derive(validator::Validate))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteStoreBalanceRequestBotRequest {
    pub tg_user_id: i64,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "validate", derive(validator::Validate))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RejectStoreBalanceRequestBotRequest {
    pub tg_user_id: i64,
}
