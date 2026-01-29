use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use shared_dtos::order::OrderStatus;
use sqlx::prelude::FromRow;

use crate::define_list_query;

#[derive(FromRow, Debug, Clone, Serialize)]
pub struct OrderRow {
    pub id: i64,
    pub customer_id: i64,
    pub amount: Decimal,
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
    pub amount: Decimal,
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
