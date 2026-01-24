use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentSystem {
    PlatformCard,
    PlatformSBP,
    Mock,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaymentGateway {
    pub name: PaymentSystem,
    pub display_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvoiceStatus {
    Pending,
    Processing,
    AwaitingReceipt,
    ReceiptSubmitted,
    Disputed,
    Completed,
    Failed,
    Expired,
    Cancelled,
    Refunded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentInvoiceResponse {
    pub id: i64,
    pub customer_id: i64,
    pub original_amount: f64,
    pub amount: f64,
    pub order_id: uuid::Uuid,
    pub payment_details: serde_json::Value,
    pub status: InvoiceStatus,
    pub gateway: PaymentSystem,
    pub created_at: DateTime<Utc>,
}
