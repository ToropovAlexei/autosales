use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{product::ProductDetails, user_subscription::UserSubscriptionDetails};

#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(feature = "sqlx", sqlx(type_name = "TEXT", rename_all = "snake_case"))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export, export_to = "order.ts"))]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    Created,
    Paid,
    Fulfilled,
    Cancelled,
    Refunded,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseBotResponse {
    pub product_name: String,
    pub balance: f64,
    pub details: Option<PurchaseDetails>,
    pub fulfilled_text: Option<String>,
    pub fulfilled_image_id: Option<Uuid>,
    pub price: f64,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "validate", derive(validator::Validate))]
#[derive(Debug, Serialize, Deserialize)]
pub struct PurchaseBotRequest {
    pub product_id: i64,
    pub telegram_id: i64,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItemBotResponse {
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

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichedOrderBotResponse {
    pub id: i64,
    pub customer_id: i64,
    pub amount: f64,
    pub currency: String,
    pub status: OrderStatus,
    pub created_at: DateTime<Utc>,
    pub order_items: Vec<OrderItemBotResponse>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PurchaseDetails {
    ProductDetails(ProductDetails),
    UserSubscriptionDetails(UserSubscriptionDetails),
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "order.ts", rename = "OrderItem")
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItemAdminResponse {
    pub id: i64,
    pub order_id: i64,
    pub product_id: i64,
    pub name_at_purchase: String,
    pub price_at_purchase: f64,
    pub quantity: i16,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export, export_to = "order.ts", rename = "Order"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderAdminResponse {
    pub id: i64,
    pub customer_id: i64,
    pub amount: f64,
    pub currency: String,
    pub status: OrderStatus,
    pub order_items: Vec<OrderItemAdminResponse>,
    pub bot_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub paid_at: Option<DateTime<Utc>>,
    pub fulfilled_at: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
}
