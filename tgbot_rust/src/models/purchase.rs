use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseResponse {
    pub product_name: String,
    pub balance: f64,
    pub details: Option<serde_json::Value>,
    pub fulfilled_text: Option<String>,
    pub fulfilled_image_id: Option<Uuid>,
    pub price: f64,
}
