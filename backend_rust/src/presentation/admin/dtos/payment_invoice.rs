use rust_decimal::prelude::ToPrimitive;
use shared_dtos::invoice::PaymentInvoiceAdminResponse;

use crate::models::payment_invoice::PaymentInvoiceRow;

impl From<PaymentInvoiceRow> for PaymentInvoiceAdminResponse {
    fn from(r: PaymentInvoiceRow) -> Self {
        PaymentInvoiceAdminResponse {
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
