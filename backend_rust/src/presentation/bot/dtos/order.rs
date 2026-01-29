use rust_decimal::prelude::ToPrimitive;
use shared_dtos::order::{EnrichedOrderBotResponse, OrderItemBotResponse};

use crate::services::order::EnrichedOrder;

impl From<EnrichedOrder> for EnrichedOrderBotResponse {
    fn from(value: EnrichedOrder) -> Self {
        Self {
            id: value.id,
            customer_id: value.customer_id,
            amount: value.amount.to_f64().unwrap_or_default(),
            currency: value.currency,
            status: value.status,
            created_at: value.created_at,
            order_items: value
                .order_items
                .iter()
                .map(|o| OrderItemBotResponse {
                    fulfillment_content: o.fulfillment_content.clone(),
                    fulfillment_image_id: o.fulfillment_image_id,
                    fulfillment_type: o.fulfillment_type.clone(),
                    name_at_purchase: o.name_at_purchase.clone(),
                    order_id: o.order_id,
                    price_at_purchase: o.price_at_purchase.to_f64().unwrap_or_default(),
                    product_id: o.product_id,
                    quantity: o.quantity,
                    details: o.details.clone(),
                    id: o.id,
                })
                .collect(),
        }
    }
}
