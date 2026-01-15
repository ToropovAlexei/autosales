use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    Created,
    Paid,
    Fulfilled,
    Cancelled,
    Refunded,
}

#[derive(Debug, Clone, Deserialize)]
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

#[derive(Debug, Clone, Deserialize)]
pub struct OrderResponse {
    pub id: i64,
    pub customer_id: i64,
    pub amount: f64,
    pub currency: String,
    pub status: OrderStatus,
    pub created_at: DateTime<Utc>,
    pub order_items: Vec<OrderItemResponse>,
}
