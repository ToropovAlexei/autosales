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
