use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    errors::api::ApiResult,
    infrastructure::{
        external::payment::mock::MockPaymentsProvider,
        repositories::{
            audit_log::AuditLogRepository, payment_invoice::PaymentInvoiceRepository,
            settings::SettingsRepository, transaction::TransactionRepository,
        },
    },
    models::{
        payment_invoice::InvoiceStatus,
        transaction::{NewTransaction, TransactionType},
    },
    services::{
        audit_log::AuditLogService,
        payment_invoice::{
            PaymentInvoiceService, PaymentInvoiceServiceTrait, UpdatePaymentInvoiceCommand,
        },
        transaction::{TransactionService, TransactionServiceTrait},
    },
};
use rust_decimal_macros::dec;

#[async_trait]
pub trait PaymentProcessingServiceTrait: Send + Sync {
    async fn handle_payment_success(&self, order_id: Uuid) -> ApiResult<()>;
}

pub struct PaymentProcessingService<T, P> {
    pub transactions_service: Arc<T>,
    pub payment_invoice_service: Arc<P>,
}

impl<T, P> PaymentProcessingService<T, P>
where
    T: TransactionServiceTrait + Send + Sync,
    P: PaymentInvoiceServiceTrait + Send + Sync,
{
    pub fn new(transactions_service: Arc<T>, payment_invoice_service: Arc<P>) -> Self {
        Self {
            transactions_service,
            payment_invoice_service,
        }
    }
}

#[async_trait]
impl PaymentProcessingServiceTrait
    for PaymentProcessingService<
        TransactionService<TransactionRepository>,
        PaymentInvoiceService<
            PaymentInvoiceRepository,
            AuditLogService<AuditLogRepository>,
            MockPaymentsProvider,
            SettingsRepository,
        >,
    >
{
    async fn handle_payment_success(&self, order_id: Uuid) -> ApiResult<()> {
        let payment_invoice = self
            .payment_invoice_service
            .get_by_order_id(order_id)
            .await?;
        self.transactions_service
            .create(NewTransaction {
                amount: payment_invoice.original_amount,
                customer_id: Some(payment_invoice.customer_id),
                r#type: TransactionType::Deposit,
                store_balance_delta: dec!(0), // TODO
                platform_commission: dec!(0), // TODO
                gateway_commission: dec!(0),  // TODO
                description: None,
                payment_gateway: Some(payment_invoice.gateway),
                details: Some(payment_invoice.payment_details),
                order_id: None, // Not invoice order id
            })
            .await?;
        self.payment_invoice_service
            .update(UpdatePaymentInvoiceCommand {
                id: payment_invoice.id,
                notification_sent_at: None,
                status: Some(InvoiceStatus::Completed),
            })
            .await?;
        // TODO Notify user
        Ok(())
    }
}
