pub mod common;
pub mod user;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct DispatchMessagePayload {
    pub bot_name: String,
    pub telegram_id: i64,
    pub message: String,
    pub message_to_edit: Option<i32>,
    pub message_to_delete: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackendResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListResponse<T> {
    pub items: Vec<T>,
    pub total: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BalanceResponse {
    pub balance: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentGateway {
    pub name: String,
    pub display_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InvoiceResponse {
    pub pay_url: Option<String>,
    pub gateway_invoice_id: String,
    pub order_id: String,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserOrder {
    pub id: i64,
    pub product_name: String,
    pub amount: f64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub fulfilled_content: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Product {
    pub id: i64,
    pub name: String,
    pub price: f64,
    pub category_id: i64,
    pub image_url: Option<String>,
    pub image_id: Option<String>,
    pub stock: i64,
    #[serde(rename = "type")]
    pub type_: String,
    pub subscription_period_days: i64,
    pub provider: Option<String>,
    pub external_id: Option<String>,
    pub visible: bool,
    pub fulfillment_type: Option<String>,
    pub fulfillment_content: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSubscription {
    pub product: Product,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
    pub image_id: Option<String>,
    pub sub_categories: Option<Vec<Category>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuyResponse {
    pub balance: f64,
    pub product_name: String,
    pub product_price: f64,
    pub fulfilled_content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bot {
    pub id: i64,
    pub token: String,
    pub username: String,
    pub is_active: bool,
    pub is_primary: bool,
    pub referral_percentage: f64,
}
