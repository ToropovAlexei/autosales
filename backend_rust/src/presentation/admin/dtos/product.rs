use bigdecimal::ToPrimitive;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::{ToResponse, ToSchema};
use uuid::Uuid;
use validator::Validate;

use crate::models::product::{ProductRow, ProductType};

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema, ToResponse)]
#[ts(export, export_to = "product.ts", rename = "Product")]
pub struct ProductResponse {
    pub id: i64,
    pub name: String,
    pub base_price: f64,
    pub price: f64,
    pub category_id: Option<i64>,
    pub image_id: Option<Uuid>,
    pub r#type: ProductType,
    pub subscription_period_days: i16,
    pub details: Option<serde_json::Value>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub fulfillment_text: Option<String>,
    pub fulfillment_image_id: Option<Uuid>,
    pub provider_name: String,
    pub external_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: i64,
}

impl From<ProductRow> for ProductResponse {
    fn from(r: ProductRow) -> Self {
        ProductResponse {
            id: r.id,
            name: r.name,
            base_price: r.price.to_f64().unwrap_or_default(),
            // TODO calc price
            price: r.price.to_f64().unwrap_or_default(),
            category_id: r.category_id,
            image_id: r.image_id,
            r#type: r.r#type,
            subscription_period_days: r.subscription_period_days,
            details: r.details,
            deleted_at: r.deleted_at,
            fulfillment_text: r.fulfillment_text,
            fulfillment_image_id: r.fulfillment_image_id,
            provider_name: r.provider_name,
            external_id: r.external_id,
            created_at: r.created_at,
            updated_at: r.updated_at,
            created_by: r.created_by,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, TS, ToSchema, ToResponse)]
#[ts(export, export_to = "product.ts", rename = "NewProduct")]
pub struct NewProductRequest {
    #[validate(length(
        min = 3,
        max = 255,
        message = "Name must be at least 3 characters and at most 255 characters"
    ))]
    pub name: String,
    #[validate(range(
        min = 0.01,
        max = 999999.99,
        message = "Price must be between 0.01 and 999999.99"
    ))]
    pub base_price: f64,
    pub category_id: i64,
    #[ts(optional)]
    pub image_id: Option<Uuid>,
    pub r#type: ProductType,
    #[ts(optional)]
    pub subscription_period_days: Option<i16>,
    #[ts(optional)]
    pub details: Option<serde_json::Value>,
    #[ts(optional)]
    pub fulfillment_text: Option<String>,
    #[ts(optional)]
    pub fulfillment_image_id: Option<Uuid>,
    #[ts(optional)]
    pub initial_stock: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, TS, ToSchema, ToResponse)]
#[ts(export, export_to = "product.ts", rename = "UpdateProduct")]
pub struct UpdateProductRequest {
    #[validate(length(
        min = 3,
        max = 255,
        message = "Name must be at least 3 characters and at most 255 characters"
    ))]
    #[ts(optional)]
    pub name: Option<String>,
    #[validate(range(
        min = 0.01,
        max = 999999.99,
        message = "Price must be between 0.01 and 999999.99"
    ))]
    #[ts(optional)]
    pub base_price: Option<f64>,
    #[ts(optional)]
    pub category_id: Option<i64>,
    #[ts(optional)]
    pub image_id: Option<Option<Uuid>>,
    #[ts(optional)]
    pub r#type: Option<ProductType>,
    #[ts(optional)]
    pub subscription_period_days: Option<i16>,
    #[ts(optional)]
    pub details: Option<Option<serde_json::Value>>,
    #[ts(optional)]
    pub fulfillment_text: Option<Option<String>>,
    #[ts(optional)]
    pub fulfillment_image_id: Option<Option<Uuid>>,
    #[ts(optional)]
    pub external_id: Option<Option<String>>,
    #[ts(optional)]
    pub stock: Option<i64>,
}
