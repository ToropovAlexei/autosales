use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(feature = "sqlx", sqlx(type_name = "TEXT", rename_all = "snake_case"))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export, export_to = "product.ts"))]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductType {
    Item,
    Subscription,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductBotResponse {
    pub id: i64,
    pub name: String,
    pub price: f64,
    pub category_id: Option<i64>,
    pub image_id: Option<Uuid>,
    pub r#type: ProductType,
    pub subscription_period_days: i16,
    pub details: Option<ProductDetails>,
    pub fulfillment_text: Option<String>,
    pub fulfillment_image_id: Option<Uuid>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export, export_to = "product.ts"))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductDetails {
    ContMs { host: String, port: u16 },
}
