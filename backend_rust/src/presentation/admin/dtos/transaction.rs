use bigdecimal::ToPrimitive;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::{ToResponse, ToSchema};

use crate::models::transaction::{TransactionRow, TransactionType};

#[derive(Debug, Serialize, Deserialize, ToSchema, ToResponse, TS)]
#[ts(export, export_to = "transaction.ts", rename = "Transaction")]
pub struct TransactionResponse {
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
    pub payment_gateway: Option<String>,
}

impl From<TransactionRow> for TransactionResponse {
    fn from(row: TransactionRow) -> Self {
        Self {
            id: row.id,
            customer_id: row.customer_id,
            order_id: row.order_id,
            r#type: row.r#type,
            amount: row.amount.to_f64().unwrap_or_default(),
            store_balance_delta: row.store_balance_delta.to_f64().unwrap_or_default(),
            platform_commission: row.platform_commission.to_f64().unwrap_or_default(),
            gateway_commission: row.gateway_commission.to_f64().unwrap_or_default(),
            created_at: row.created_at,
            description: row.description,
            payment_gateway: row.payment_gateway,
        }
    }
}
