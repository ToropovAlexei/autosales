use chrono::{DateTime, Utc};
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use shared_dtos::invoice::{InvoiceStatus, PaymentInvoiceBotResponse, PaymentSystem};
use utoipa::ToSchema;
use validator::Validate;

use crate::models::payment_invoice::PaymentInvoiceRow;

impl From<PaymentInvoiceRow> for PaymentInvoiceBotResponse {
    fn from(r: PaymentInvoiceRow) -> Self {
        Self {
            id: r.id,
            customer_id: r.customer_id,
            original_amount: r.original_amount.to_f64().unwrap_or_default(),
            amount: r.amount.to_f64().unwrap_or_default(),
            order_id: r.order_id,
            payment_details: serde_json::from_value(r.payment_details).unwrap_or_default(),
            status: r.status,
            created_at: r.created_at,
            gateway: r.gateway,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct NewPaymentInvoiceRequest {
    pub telegram_id: i64,
    #[validate(range(
        min = 0.0,
        max = 1000000.0,
        message = "Amount must be between 0 and 1000000"
    ))]
    pub amount: f64,
    pub gateway: PaymentSystem,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdatePaymentInvoiceRequest {
    pub status: Option<InvoiceStatus>,
    pub notification_sent_at: Option<Option<DateTime<Utc>>>,
}
