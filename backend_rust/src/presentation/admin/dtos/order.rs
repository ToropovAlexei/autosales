use crate::models::order::{OrderRow, OrderStatus};
use bigdecimal::ToPrimitive;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::{ToResponse, ToSchema};

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema, ToResponse)]
#[ts(export, export_to = "order.ts", rename = "Order")]
pub struct OrderResponse {
    pub id: i64,
    pub customer_id: i64,
    pub amount: f64,
    pub currency: String,
    pub status: OrderStatus,
    pub bot_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub paid_at: Option<DateTime<Utc>>,
    pub fulfilled_at: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
}

impl From<OrderRow> for OrderResponse {
    fn from(r: OrderRow) -> Self {
        Self {
            id: r.id,
            customer_id: r.customer_id,
            amount: r.amount.to_f64().unwrap_or_default(),
            currency: r.currency,
            status: r.status,
            bot_id: r.bot_id,
            created_at: r.created_at,
            updated_at: r.updated_at,
            paid_at: r.paid_at,
            fulfilled_at: r.fulfilled_at,
            cancelled_at: r.cancelled_at,
        }
    }
}
