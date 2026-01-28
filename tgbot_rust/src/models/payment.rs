use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
pub use shared_dtos::InvoiceStatus;

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PaymentDetails {
    Mock {
        pay_url: String,
    },
    PlatformCard {
        bank_name: String,
        account_name: String,
        card_number: String,
        amount: f64,
    },
    PlatformSBP {
        bank_name: String,
        account_name: String,
        sbp_number: String,
        amount: f64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentInvoiceResponse {
    pub id: i64,
    pub customer_id: i64,
    pub original_amount: f64,
    pub amount: f64,
    pub order_id: uuid::Uuid,
    pub payment_details: Option<PaymentDetails>,
    pub status: InvoiceStatus,
    pub gateway: PaymentSystem,
    pub created_at: DateTime<Utc>,
}
