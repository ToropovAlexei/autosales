use chrono::{DateTime, Utc};
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use shared_dtos::invoice::{InvoiceStatus, PaymentSystem};
use ts_rs::TS;
use utoipa::ToSchema;

use crate::models::payment_invoice::PaymentInvoiceRow;

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "invoice.ts", rename = "PaymentInvoice")]
pub struct PaymentInvoiceResponse {
    pub id: i64,
    pub customer_id: i64,
    pub original_amount: f64,
    pub amount: f64,
    pub status: InvoiceStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub gateway: PaymentSystem,
    pub gateway_invoice_id: String,
}

impl From<PaymentInvoiceRow> for PaymentInvoiceResponse {
    fn from(r: PaymentInvoiceRow) -> Self {
        PaymentInvoiceResponse {
            id: r.id,
            customer_id: r.customer_id,
            original_amount: r.original_amount.to_f64().unwrap_or_default(),
            amount: r.amount.to_f64().unwrap_or_default(),
            status: r.status,
            created_at: r.created_at,
            updated_at: r.updated_at,
            expires_at: r.expires_at,
            gateway: r.gateway,
            gateway_invoice_id: r.gateway_invoice_id,
        }
    }
}
