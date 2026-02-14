use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use shared_dtos::{invoice::PaymentSystem, transaction::TransactionType};
use sqlx::prelude::FromRow;

use crate::define_list_query;

#[derive(FromRow, Debug)]
pub struct TransactionRow {
    pub id: i64,
    pub customer_id: Option<i64>,
    pub order_id: Option<i64>,
    pub r#type: TransactionType,
    pub amount: Decimal,
    pub store_balance_delta: Decimal,
    pub user_balance_after: Option<Decimal>,
    pub store_balance_after: Decimal,
    pub platform_commission: Decimal,
    pub gateway_commission: Decimal,
    pub created_at: DateTime<Utc>,
    pub description: Option<String>,
    pub payment_gateway: Option<PaymentSystem>,
    pub details: Option<serde_json::Value>,
    pub bot_id: Option<i64>,
}

#[derive(Debug)]
pub struct NewTransaction {
    pub customer_id: Option<i64>,
    pub order_id: Option<i64>,
    pub r#type: TransactionType,
    pub amount: Decimal,
    pub store_balance_delta: Decimal,
    pub platform_commission: Decimal,
    pub gateway_commission: Decimal,
    pub description: Option<String>,
    pub payment_gateway: Option<PaymentSystem>,
    pub details: Option<serde_json::Value>,
    pub bot_id: Option<i64>,
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
