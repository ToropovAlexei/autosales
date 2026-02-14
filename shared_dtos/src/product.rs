use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::rust::double_option;
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

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "product.ts", rename = "Product")
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductAdminResponse {
    pub id: i64,
    pub name: String,
    pub base_price: f64,
    pub price: f64,
    pub stock: i32,
    pub category_id: Option<i64>,
    pub image_id: Option<Uuid>,
    pub r#type: ProductType,
    pub subscription_period_days: i16,
    pub details: Option<ProductDetails>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub fulfillment_text: Option<String>,
    pub fulfillment_image_id: Option<Uuid>,
    pub provider_name: String,
    pub external_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: i64,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "validate", derive(validator::Validate))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "product.ts", rename = "NewProduct")
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewProductAdminRequest {
    #[cfg_attr(
        feature = "validate",
        validate(length(
            min = 3,
            max = 255,
            message = "Name must be at least 3 characters and at most 255 characters"
        ))
    )]
    pub name: String,
    #[cfg_attr(
        feature = "validate",
        validate(range(
            min = 0.01,
            max = 999999.99,
            message = "Price must be between 0.01 and 999999.99"
        ))
    )]
    pub base_price: f64,
    pub category_id: i64,
    #[cfg_attr(feature = "ts", ts(optional))]
    pub image_id: Option<Uuid>,
    pub r#type: ProductType,
    #[cfg_attr(feature = "ts", ts(optional))]
    pub subscription_period_days: Option<i16>,
    #[cfg_attr(feature = "ts", ts(optional))]
    pub details: Option<serde_json::Value>,
    #[cfg_attr(feature = "ts", ts(optional))]
    pub fulfillment_text: Option<String>,
    #[cfg_attr(feature = "ts", ts(optional))]
    pub fulfillment_image_id: Option<Uuid>,
    #[cfg_attr(feature = "ts", ts(optional))]
    pub initial_stock: Option<i64>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "validate", derive(validator::Validate))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "product.ts", rename = "UpdateProduct")
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProductAdminRequest {
    #[cfg_attr(
        feature = "validate",
        validate(length(
            min = 3,
            max = 255,
            message = "Name must be at least 3 characters and at most 255 characters"
        ))
    )]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub name: Option<String>,
    #[cfg_attr(
        feature = "validate",
        validate(range(
            min = 0.01,
            max = 999999.99,
            message = "Price must be between 0.01 and 999999.99"
        ))
    )]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub base_price: Option<f64>,
    #[cfg_attr(feature = "ts", ts(optional))]
    pub category_id: Option<i64>,
    #[cfg_attr(feature = "ts", ts(optional))]
    #[cfg_attr(feature = "ts", ts(type = "string | null"))]
    #[serde(default, with = "double_option")]
    pub image_id: Option<Option<Uuid>>,
    #[cfg_attr(feature = "ts", ts(optional))]
    pub r#type: Option<ProductType>,
    #[cfg_attr(feature = "ts", ts(optional))]
    pub subscription_period_days: Option<i16>,
    #[cfg_attr(feature = "ts", ts(optional))]
    #[cfg_attr(feature = "ts", ts(type = "Record<string, any> | null"))]
    #[serde(default, with = "double_option")]
    pub details: Option<Option<serde_json::Value>>,
    #[cfg_attr(feature = "ts", ts(optional))]
    #[cfg_attr(feature = "ts", ts(type = "string | null"))]
    #[serde(default, with = "double_option")]
    pub fulfillment_text: Option<Option<String>>,
    #[cfg_attr(feature = "ts", ts(optional))]
    #[cfg_attr(feature = "ts", ts(type = "string | null"))]
    #[serde(default, with = "double_option")]
    pub fulfillment_image_id: Option<Option<Uuid>>,
    #[cfg_attr(feature = "ts", ts(optional))]
    #[cfg_attr(feature = "ts", ts(type = "string | null"))]
    #[serde(default, with = "double_option")]
    pub external_id: Option<Option<String>>,
    #[cfg_attr(feature = "ts", ts(optional))]
    pub stock: Option<i64>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "validate", derive(validator::Validate))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "product.ts", rename = "UploadProductsResponse")
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductsUploadResponse {
    pub created: i64,
    pub failed: i64,
    pub skipped: i64,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadedProductCSV {
    pub name: String,
    pub category: String,
    pub price: f64,
    pub initial_stock: i64,
}
