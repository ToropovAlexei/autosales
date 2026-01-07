use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::BigDecimal};
use ts_rs::TS;
use utoipa::ToSchema;

use crate::define_list_query;

#[derive(sqlx::Type, Debug, Clone, Copy, PartialEq, Deserialize, Serialize, TS, ToSchema)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "transaction.ts")]
pub enum TransactionType {
    Deposit,
    Purchase,
    Withdrawal,
    ReferralPayout,
    ServiceCharge,
    Refund,
}

#[derive(FromRow, Debug)]
pub struct TransactionRow {
    pub id: i64,
    pub customer_id: Option<i64>,
    pub order_id: Option<i64>,
    pub r#type: TransactionType,
    pub amount: BigDecimal,
    pub store_balance_delta: BigDecimal,
    pub user_balance_after: Option<BigDecimal>,
    pub store_balance_after: BigDecimal,
    pub platform_commission: BigDecimal,
    pub gateway_commission: BigDecimal,
    pub created_at: DateTime<Utc>,
    pub description: Option<String>,
    pub payment_gateway: Option<String>,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug)]
pub struct NewTransaction {
    pub customer_id: Option<i64>,
    pub order_id: Option<i64>,
    pub r#type: TransactionType,
    pub amount: BigDecimal,
    pub store_balance_delta: BigDecimal,
    pub platform_commission: BigDecimal,
    pub gateway_commission: BigDecimal,
    pub description: Option<String>,
    pub payment_gateway: Option<String>,
    pub details: Option<serde_json::Value>,
}

define_list_query! {
    query_name: TransactionListQuery,
    filter_fields: {
        TransactionFilterFields,
        [
            Id => "id",
            CustomerId => "customer_id",
            OrderId => "order_id",
            Type => "type",
            Amount => "amount",
            StoreBalanceDelta => "store_balance_delta",
            PlatformCommission => "platform_commission",
            GatewayCommission => "gateway_commission",
            CreatedAt => "created_at",
        ]
    },
    order_fields: {
        TransactionOrderFields,
        [
            Id => "id",
            CustomerId => "customer_id",
            OrderId => "order_id",
            Type => "type",
            Amount => "amount",
            StoreBalanceDelta => "store_balance_delta",
            PlatformCommission => "platform_commission",
            GatewayCommission => "gateway_commission",
            CreatedAt => "created_at",
        ]
    }
}
