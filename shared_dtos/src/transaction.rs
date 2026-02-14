use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::invoice::PaymentSystem;

#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(feature = "sqlx", sqlx(type_name = "TEXT", rename_all = "snake_case"))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export, export_to = "transaction.ts"))]
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    Deposit,
    Purchase,
    Withdrawal,
    ReferralPayout,
    ServiceCharge,
    Refund,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "transaction.ts", rename = "Transaction")
)]
#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionAdminResponse {
    pub id: i64,
    pub customer_id: Option<i64>,
    pub order_id: Option<i64>,
    pub r#type: TransactionType,
    pub amount: f64,
    pub store_balance_delta: f64,
    pub platform_commission: f64,
    pub gateway_commission: f64,
    pub created_at: DateTime<Utc>,
    pub description: Option<String>,
    pub payment_gateway: Option<PaymentSystem>,
}
