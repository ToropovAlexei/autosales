use bigdecimal::BigDecimal;
use serde::Serialize;
use sqlx::prelude::FromRow;

#[derive(FromRow, Debug, Clone, Serialize)]
pub struct OrderItemRow {
    pub id: i64,
    pub order_id: i64,
    pub product_id: i64,
    pub name_at_purchase: String,
    pub price_at_purchase: BigDecimal,
    pub quantity: i16,
    pub fulfillment_type: String,
    pub fulfillment_content: Option<String>,
    pub fulfillment_image_id: Option<uuid::Uuid>,
    pub details: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug)]
pub struct NewOrderItem {
    pub order_id: i64,
    pub product_id: i64,
    pub name_at_purchase: String,
    pub price_at_purchase: BigDecimal,
    pub quantity: i16,
    pub fulfillment_type: String,
    pub fulfillment_content: Option<String>,
    pub fulfillment_image_id: Option<uuid::Uuid>,
    pub details: Option<serde_json::Value>,
}
