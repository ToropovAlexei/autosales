use chrono::{DateTime, Utc};
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::{models::order::OrderStatus, services::order::EnrichedOrder};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PurchaseResponse {
    pub product_name: String,
    pub balance: f64,
    pub details: Option<serde_json::Value>,
    pub fulfilled_text: Option<String>,
    pub fulfilled_image_id: Option<Uuid>,
    pub price: f64,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct PurchaseRequest {
    pub product_id: i64,
    pub telegram_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OrderItemResponse {
    pub id: i64,
    pub order_id: i64,
    pub product_id: i64,
    pub name_at_purchase: String,
    pub price_at_purchase: f64,
    pub quantity: i16,
    pub fulfillment_type: String,
    pub fulfillment_content: Option<String>,
    pub fulfillment_image_id: Option<uuid::Uuid>,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EnrichedOrderResponse {
    pub id: i64,
    pub customer_id: i64,
    pub amount: f64,
    pub currency: String,
    pub status: OrderStatus,
    pub created_at: DateTime<Utc>,
    pub order_items: Vec<OrderItemResponse>,
}

impl From<EnrichedOrder> for EnrichedOrderResponse {
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
                .map(|o| OrderItemResponse {
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
