use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    errors::api::ApiResult,
    infrastructure::{
        external::payment::{
            autosales_platform::AutosalesPlatformPaymentsProvider, mock::MockPaymentsProvider,
        },
        repositories::{
            audit_log::AuditLogRepository, customer::CustomerRepository,
            payment_invoice::PaymentInvoiceRepository, settings::SettingsRepository,
            transaction::TransactionRepository,
        },
    },
    models::{
        payment_invoice::InvoiceStatus,
        transaction::{NewTransaction, TransactionType},
    },
    services::{
        audit_log::AuditLogService,
        customer::{CustomerService, CustomerServiceTrait},
        notification_service::{
            DispatchMessage, DispatchMessageCommand, NotificationService, NotificationServiceTrait,
        },
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

pub struct PaymentProcessingService<T, P, N, C> {
    pub transactions_service: Arc<T>,
    pub payment_invoice_service: Arc<P>,
    pub notification_service: Arc<N>,
    pub customer_service: Arc<C>,
}

impl<T, P, N, C> PaymentProcessingService<T, P, N, C>
where
    T: TransactionServiceTrait + Send + Sync,
    P: PaymentInvoiceServiceTrait + Send + Sync,
    N: NotificationServiceTrait + Send + Sync,
    C: CustomerServiceTrait + Send + Sync,
{
    pub fn new(
        transactions_service: Arc<T>,
        payment_invoice_service: Arc<P>,
        notification_service: Arc<N>,
        customer_service: Arc<C>,
    ) -> Self {
        Self {
            transactions_service,
            payment_invoice_service,
            notification_service,
            customer_service,
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
            AutosalesPlatformPaymentsProvider,
        >,
        NotificationService,
        CustomerService<CustomerRepository, AuditLogService<AuditLogRepository>>,
    >
{
    async fn handle_payment_success(&self, order_id: Uuid) -> ApiResult<()> {
        let payment_invoice = self
            .payment_invoice_service
            .get_by_order_id(order_id)
            .await?;
        let customer = self
            .customer_service
            .get_by_id(payment_invoice.customer_id)
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
        self.notification_service
            .dispatch_message(DispatchMessageCommand {
                bot_id: customer.last_seen_with_bot,
                message: DispatchMessage::GenericMessage {
                    image_id: None,
                    message: format!(
                        "✅ Баланс пополнен на {} RUB",
                        payment_invoice.original_amount.trunc_with_scale(2)
                    ),
                },
                telegram_id: customer.telegram_id,
            })
            .await?;
        Ok(())
    }
}
