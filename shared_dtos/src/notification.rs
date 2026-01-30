use chrono::{DateTime, Utc};
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
        expired_at: DateTime<Utc>,
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
