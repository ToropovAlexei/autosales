use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::balance_request::StoreBalanceRequestType;

#[derive(Debug, Deserialize, Serialize)]
pub enum DispatchMessage {
    GenericMessage {
        message: String,
        image_id: Option<Uuid>,
    },
    DisputeFailedNotification,
    SubscriptionExpiringNotification {
        expires_at: DateTime<Utc>,
        product_name: Option<String>,
    },
    InvoiceTroublesNotification {
        invoice_id: i64,
        amount: f64,
        expired_at: DateTime<Utc>,
    },
    RequestReceiptNotification {
        invoice_id: i64,
        is_first_time: bool,
        expired_at: DateTime<Utc>,
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub enum DispatchAdminMessage {
    StoreBalanceRequestNotification {
        store_balance_request_id: i64,
        amount_in_rub: f64,
        amount_in_usdt: f64,
        r#type: StoreBalanceRequestType,
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DispatchMessagePayload {
    pub bot_id: i64,
    pub telegram_id: i64,
    pub message: DispatchMessage,
}
