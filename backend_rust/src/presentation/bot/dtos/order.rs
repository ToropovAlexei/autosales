use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

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
