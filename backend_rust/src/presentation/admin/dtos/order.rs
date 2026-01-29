use crate::models::order::OrderRow;
use chrono::{DateTime, Utc};
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use shared_dtos::order::OrderStatus;
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

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    #[test]
    fn test_order_response_from_order_row() {
        let now = Utc::now();
        let order_row = OrderRow {
            id: 1,
            customer_id: 10,
            amount: Decimal::new(12345, 2), // 123.45
            currency: "USD".to_string(),
            status: OrderStatus::Paid,
            bot_id: 100,
            created_at: now,
            updated_at: now,
            paid_at: Some(now),
            fulfilled_at: Some(now),
            cancelled_at: None,
        };

        let order_response: OrderResponse = order_row.into();

        assert_eq!(order_response.id, 1);
        assert_eq!(order_response.customer_id, 10);
        assert_eq!(order_response.amount, 123.45);
        assert_eq!(order_response.currency, "USD");
        assert_eq!(order_response.status, OrderStatus::Paid);
        assert_eq!(order_response.bot_id, 100);
        assert_eq!(order_response.created_at, now);
        assert_eq!(order_response.updated_at, now);
        assert_eq!(order_response.paid_at, Some(now));
        assert_eq!(order_response.fulfilled_at, Some(now));
        assert_eq!(order_response.cancelled_at, None);
    }
}
