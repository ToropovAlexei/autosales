use crate::services::order::EnrichedOrder;
use rust_decimal::prelude::ToPrimitive;
use shared_dtos::order::{OrderAdminResponse, OrderItemAdminResponse};

impl From<EnrichedOrder> for OrderAdminResponse {
    fn from(r: EnrichedOrder) -> Self {
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
            order_items: r
                .order_items
                .iter()
                .map(|i| OrderItemAdminResponse {
                    id: i.id,
                    name_at_purchase: i.name_at_purchase.clone(),
                    order_id: i.order_id,
                    price_at_purchase: i.price_at_purchase.to_f64().unwrap_or_default(),
                    product_id: i.product_id,
                    quantity: i.quantity,
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use rust_decimal::Decimal;
    use shared_dtos::order::OrderStatus;

    #[test]
    fn test_order_response_from_order_row() {
        let now = Utc::now();
        let order_row = EnrichedOrder {
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
            order_items: vec![],
        };

        let order_response: OrderAdminResponse = order_row.into();

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
        assert_eq!(order_response.order_items.len(), 0);
    }
}
