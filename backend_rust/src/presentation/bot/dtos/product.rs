use bigdecimal::ToPrimitive;
use serde::{Deserialize, Serialize};
use utoipa::{ToResponse, ToSchema};
use uuid::Uuid;

use crate::models::product::{ProductRow, ProductType};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, ToResponse)]
pub struct ProductResponse {
    pub id: i64,
    pub name: String,
    pub price: f64,
    pub category_id: Option<i64>,
    pub image_id: Option<Uuid>,
    pub r#type: ProductType,
    pub subscription_period_days: i16,
    pub details: Option<serde_json::Value>,
    pub fulfillment_text: Option<String>,
    pub fulfillment_image_id: Option<Uuid>,
}

impl From<ProductRow> for ProductResponse {
    fn from(r: ProductRow) -> Self {
        ProductResponse {
            id: r.id,
            name: r.name,
            // TODO calc price
            price: r.price.to_f64().unwrap_or_default(),
            category_id: r.category_id,
            image_id: r.image_id,
            r#type: r.r#type,
            subscription_period_days: r.subscription_period_days,
            details: r.details,
            fulfillment_text: r.fulfillment_text,
            fulfillment_image_id: r.fulfillment_image_id,
        }
    }
}
