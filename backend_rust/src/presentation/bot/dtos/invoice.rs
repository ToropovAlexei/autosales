use rust_decimal::prelude::ToPrimitive;
use shared_dtos::invoice::PaymentInvoiceBotResponse;

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
            gateway_invoice_id: r.gateway_invoice_id,
        }
    }
}
