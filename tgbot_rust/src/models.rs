pub mod bot;
pub mod category;
pub mod common;
pub mod customer;
pub mod product;
pub mod settings;
pub mod user_subscription;

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
pub struct ListResponse<T> {
    pub items: Vec<T>,
    pub total: i64,
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
pub struct BuyResponse {
    pub balance: f64,
    pub product_name: String,
    pub product_price: f64,
    pub fulfilled_content: Option<String>,
}
