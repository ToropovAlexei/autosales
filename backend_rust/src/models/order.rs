use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use ts_rs::TS;
use utoipa::ToSchema;

use crate::define_list_query;

#[derive(sqlx::Type, Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS, ToSchema)]
#[sqlx(type_name = "order_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "order.ts")]
pub enum OrderStatus {
    Created,
    Paid,
    Fulfilled,
    Cancelled,
    Refunded,
}

#[derive(FromRow, Debug, Clone, Serialize)]
pub struct OrderRow {
    pub id: i64,
    pub customer_id: i64,
    pub amount: BigDecimal,
    pub currency: String,
    pub status: OrderStatus,
    pub bot_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub paid_at: Option<DateTime<Utc>>,
    pub fulfilled_at: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct NewOrder {
    pub customer_id: i64,
    pub amount: BigDecimal,
    pub currency: String,
    pub status: OrderStatus,
    pub bot_id: i64,
    pub paid_at: Option<DateTime<Utc>>,
    pub fulfilled_at: Option<DateTime<Utc>>,
}

define_list_query! {
    query_name: OrderListQuery,
    filter_fields: {
        OrderFilterFields,
        [
            Id => "id",
            CustomerId => "customer_id",
            Amount => "amount",
            BotId => "bot_id",
            CreatedAt => "created_at",
        ]
    },
    order_fields: {
        OrderOrderFields,
        [
            Id => "id",
            CustomerId => "customer_id",
            Amount => "amount",
            BotId => "bot_id",
            CreatedAt => "created_at",
        ]
    }
}
