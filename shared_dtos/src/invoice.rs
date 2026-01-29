use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(feature = "sqlx", sqlx(type_name = "TEXT", rename_all = "snake_case"))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export, export_to = "invoice.ts"))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
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

#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(feature = "sqlx", sqlx(type_name = "TEXT", rename_all = "snake_case"))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export, export_to = "payment.ts"))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentSystem {
    PlatformCard,
    PlatformSBP,
    Mock,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Serialize, Deserialize)]
pub struct GatewayBotResponse {
    pub name: PaymentSystem,
    pub display_name: String,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentInvoiceBotResponse {
    pub id: i64,
    pub customer_id: i64,
    pub original_amount: f64,
    pub amount: f64,
    pub order_id: uuid::Uuid,
    pub payment_details: Option<PaymentDetails>,
    pub gateway: PaymentSystem,
    pub status: InvoiceStatus,
    pub created_at: DateTime<Utc>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "validate", derive(validator::Validate))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPaymentInvoiceBotRequest {
    pub telegram_id: i64,
    #[cfg_attr(
        feature = "validate",
        validate(range(
            min = 0.0,
            max = 1000000.0,
            message = "Amount must be between 0 and 1000000"
        ))
    )]
    pub amount: f64,
    pub gateway: PaymentSystem,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "validate", derive(validator::Validate))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePaymentInvoiceBotRequest {
    pub status: Option<InvoiceStatus>,
    pub notification_sent_at: Option<Option<DateTime<Utc>>>,
}
