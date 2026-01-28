use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use utoipa::ToSchema;

use crate::{define_list_query, models::payment::PaymentSystem};
use shared_dtos::InvoiceStatus;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
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

#[derive(FromRow, Debug, Clone, Serialize)]
pub struct PaymentInvoiceRow {
    pub id: i64,
    pub customer_id: i64,
    pub original_amount: Decimal,
    pub amount: Decimal,
    pub status: InvoiceStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub gateway: PaymentSystem,
    pub gateway_invoice_id: String,
    pub order_id: uuid::Uuid,
    pub payment_details: serde_json::Value,
    pub bot_message_id: Option<i64>,
    pub notification_sent_at: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct NewPaymentInvoice {
    pub customer_id: i64,
    pub original_amount: Decimal,
    pub amount: Decimal,
    pub status: InvoiceStatus,
    pub expires_at: DateTime<Utc>,
    pub gateway: PaymentSystem,
    pub gateway_invoice_id: String,
    pub order_id: uuid::Uuid,
    pub payment_details: Option<PaymentDetails>,
    pub bot_message_id: Option<i64>,
}

#[derive(Debug)]
pub struct UpdatePaymentInvoice {
    pub status: Option<InvoiceStatus>,
    pub notification_sent_at: Option<Option<DateTime<Utc>>>,
}

define_list_query! {
    query_name: PaymentInvoiceListQuery,
    filter_fields: {
        PaymentInvoiceFilterFields,
        [
            Id => "id",
        ]
    },
    order_fields: {
        PaymentInvoiceOrderFields,
        [
            Id => "id",
        ]
    }
}
