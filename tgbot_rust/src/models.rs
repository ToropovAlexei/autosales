pub mod bot;
pub mod category;
pub mod common;
pub mod customer;
pub mod order;
pub mod payment;
pub mod product;
pub mod purchase;
pub mod settings;
pub mod user_subscription;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub enum DispatchMessage {
    GenericMessage {
        message: String,
        image_id: Option<Uuid>,
    },
    InvoiceTroublesNotification {
        invoice_id: i64,
        amount: f64,
    },
    RequestReceiptNotification {
        invoice_id: i64,
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DispatchMessagePayload {
    pub bot_id: i64,
    pub telegram_id: i64,
    pub message: DispatchMessage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListResponse<T> {
    pub items: Vec<T>,
    pub total: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InvoiceResponse {
    pub id: i64,
    pub customer_id: i64,
    pub original_amount: f64,
    pub amount: f64,
    pub order_id: uuid::Uuid,
    pub payment_details: serde_json::Value,
}
