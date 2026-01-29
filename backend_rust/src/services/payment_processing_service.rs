use std::sync::Arc;

use async_trait::async_trait;
use shared_dtos::{
    invoice::InvoiceStatus,
    notification::{DispatchMessage, DispatchMessagePayload},
};
use uuid::Uuid;

use crate::{
    errors::api::ApiResult,
    models::transaction::{NewTransaction, TransactionType},
    services::{
        customer::CustomerServiceTrait,
        notification_service::NotificationServiceTrait,
        payment_invoice::{PaymentInvoiceServiceTrait, UpdatePaymentInvoiceCommand},
        transaction::TransactionServiceTrait,
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
impl<T, P, N, C> PaymentProcessingServiceTrait for PaymentProcessingService<T, P, N, C>
where
    T: TransactionServiceTrait + Send + Sync,
    P: PaymentInvoiceServiceTrait + Send + Sync,
    N: NotificationServiceTrait + Send + Sync,
    C: CustomerServiceTrait + Send + Sync,
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

        let gateway_commission_percent = dec!(0.2); // TODO get from settings
        let platform_commission_percent = dec!(0.01); // TODO get from settings
        let platform_commission = payment_invoice.original_amount * platform_commission_percent;
        let gateway_commission = payment_invoice.original_amount * gateway_commission_percent;
        let store_balance_delta = payment_invoice.original_amount
            * (dec!(1) - platform_commission_percent - gateway_commission_percent);

        self.transactions_service
            .create(NewTransaction {
                amount: payment_invoice.original_amount,
                customer_id: Some(payment_invoice.customer_id),
                r#type: TransactionType::Deposit,
                store_balance_delta,
                platform_commission,
                gateway_commission,
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
            .dispatch_message(DispatchMessagePayload {
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

#[cfg(test)]
mod tests {
    use super::*;

    use async_trait::async_trait;
    use chrono::Utc;
    use rust_decimal_macros::dec;
    use serde_json::json;
    use shared_dtos::invoice::PaymentSystem;
    use std::sync::Mutex;

    use crate::{
        errors::api::{ApiError, ApiResult},
        models::{
            customer::CustomerRow,
            payment_invoice::PaymentInvoiceRow,
            transaction::{TransactionRow, TransactionType},
        },
        services::{
            customer::UpdateCustomerCommand,
            payment_invoice::{CreatePaymentInvoiceCommand, SendInvoiceReceiptCommand},
        },
    };

    struct FakeTransactionService {
        last_created: Mutex<Option<NewTransaction>>,
        created_row: Mutex<Option<TransactionRow>>,
    }

    #[async_trait]
    impl TransactionServiceTrait for FakeTransactionService {
        async fn get_list(
            &self,
            _query: crate::models::transaction::TransactionListQuery,
        ) -> ApiResult<crate::models::common::PaginatedResult<TransactionRow>> {
            Err(ApiError::InternalServerError("not used".to_string()))
        }

        async fn create(&self, transaction: NewTransaction) -> ApiResult<TransactionRow> {
            *self.last_created.lock().unwrap() = Some(transaction);
            Ok(self
                .created_row
                .lock()
                .unwrap()
                .take()
                .expect("transaction row"))
        }

        async fn get_last(&self) -> ApiResult<TransactionRow> {
            Err(ApiError::InternalServerError("not used".to_string()))
        }
    }

    struct FakePaymentInvoiceService {
        invoice: PaymentInvoiceRow,
        updated: Mutex<Option<UpdatePaymentInvoiceCommand>>,
    }

    #[async_trait]
    impl PaymentInvoiceServiceTrait for FakePaymentInvoiceService {
        async fn get_list(
            &self,
            _query: crate::models::payment_invoice::PaymentInvoiceListQuery,
        ) -> ApiResult<crate::models::common::PaginatedResult<PaymentInvoiceRow>> {
            Err(ApiError::InternalServerError("not used".to_string()))
        }

        async fn create(
            &self,
            _command: CreatePaymentInvoiceCommand,
        ) -> ApiResult<PaymentInvoiceRow> {
            Err(ApiError::InternalServerError("not used".to_string()))
        }

        async fn get_by_id(&self, _id: i64) -> ApiResult<PaymentInvoiceRow> {
            Err(ApiError::InternalServerError("not used".to_string()))
        }

        async fn get_by_order_id(&self, order_id: Uuid) -> ApiResult<PaymentInvoiceRow> {
            assert_eq!(order_id, self.invoice.order_id);
            Ok(self.invoice.clone())
        }

        async fn update(
            &self,
            command: UpdatePaymentInvoiceCommand,
        ) -> ApiResult<PaymentInvoiceRow> {
            *self.updated.lock().unwrap() = Some(command);
            let mut updated = self.invoice.clone();
            if let Some(cmd) = self.updated.lock().unwrap().as_ref() {
                if let Some(status) = cmd.status {
                    updated.status = status;
                }
                if let Some(notification_sent_at) = cmd.notification_sent_at {
                    updated.notification_sent_at = notification_sent_at;
                }
            }
            Ok(updated)
        }

        async fn get_for_customer(&self, _customer_id: i64) -> ApiResult<Vec<PaymentInvoiceRow>> {
            Err(ApiError::InternalServerError("not used".to_string()))
        }

        async fn expire_old_invoices(&self) -> ApiResult<u64> {
            Err(ApiError::InternalServerError("not used".to_string()))
        }

        async fn get_pending_invoices(
            &self,
            _older_than: chrono::DateTime<chrono::Utc>,
        ) -> ApiResult<Vec<PaymentInvoiceRow>> {
            Err(ApiError::InternalServerError("not used".to_string()))
        }

        async fn mark_invoices_notified(&self, _ids: &[i64]) -> ApiResult<u64> {
            Err(ApiError::InternalServerError("not used".to_string()))
        }

        async fn confirm_invoice(&self, _id: i64) -> ApiResult<PaymentInvoiceRow> {
            Err(ApiError::InternalServerError("not used".to_string()))
        }

        async fn cancel_invoice(&self, _id: i64) -> ApiResult<PaymentInvoiceRow> {
            Err(ApiError::InternalServerError("not used".to_string()))
        }

        async fn send_invoice_receipt(
            &self,
            _command: SendInvoiceReceiptCommand,
        ) -> ApiResult<PaymentInvoiceRow> {
            Err(ApiError::InternalServerError("not used".to_string()))
        }
    }

    struct FakeNotificationService {
        last: Mutex<Option<DispatchMessagePayload>>,
    }

    #[async_trait]
    impl NotificationServiceTrait for FakeNotificationService {
        async fn dispatch_message(&self, payload: DispatchMessagePayload) -> ApiResult<()> {
            *self.last.lock().unwrap() = Some(payload);
            Ok(())
        }
    }

    struct FakeCustomerService {
        customer: CustomerRow,
    }

    #[async_trait]
    impl CustomerServiceTrait for FakeCustomerService {
        async fn get_list(
            &self,
            _query: crate::models::customer::CustomerListQuery,
        ) -> ApiResult<crate::models::common::PaginatedResult<CustomerRow>> {
            Err(ApiError::InternalServerError("not used".to_string()))
        }

        async fn create(
            &self,
            _customer: crate::models::customer::NewCustomer,
        ) -> ApiResult<CustomerRow> {
            Err(ApiError::InternalServerError("not used".to_string()))
        }

        async fn get_by_id(&self, id: i64) -> ApiResult<CustomerRow> {
            assert_eq!(id, self.customer.id);
            Ok(self.customer.clone())
        }

        async fn get_by_telegram_id(&self, _id: i64) -> ApiResult<CustomerRow> {
            Err(ApiError::InternalServerError("not used".to_string()))
        }

        async fn update(&self, _command: UpdateCustomerCommand) -> ApiResult<CustomerRow> {
            Err(ApiError::InternalServerError("not used".to_string()))
        }

        async fn update_last_seen(&self, _id: i64, _bot_id: i64) -> ApiResult<CustomerRow> {
            Err(ApiError::InternalServerError("not used".to_string()))
        }

        async fn get_list_by_ids(&self, _ids: &[i64]) -> ApiResult<Vec<CustomerRow>> {
            Err(ApiError::InternalServerError("not used".to_string()))
        }
    }

    #[tokio::test]
    async fn test_handle_payment_success_creates_transaction_updates_invoice_and_notifies() {
        let order_id = Uuid::new_v4();
        let now = Utc::now();
        let invoice = PaymentInvoiceRow {
            id: 1,
            customer_id: 10,
            original_amount: dec!(100),
            amount: dec!(100),
            status: InvoiceStatus::Pending,
            created_at: now,
            updated_at: now,
            expires_at: now,
            deleted_at: None,
            gateway: PaymentSystem::PlatformCard,
            gateway_invoice_id: "gw-1".to_string(),
            order_id,
            payment_details: json!({"ref": "abc"}),
            bot_message_id: None,
            notification_sent_at: None,
        };
        let customer = CustomerRow {
            id: 10,
            telegram_id: 555,
            balance: dec!(0),
            is_blocked: false,
            bot_is_blocked_by_user: false,
            has_passed_captcha: true,
            registered_with_bot: 1,
            last_seen_with_bot: 9,
            last_seen_at: now,
            created_at: now,
            updated_at: now,
        };

        let transaction_row = TransactionRow {
            id: 99,
            customer_id: Some(10),
            order_id: None,
            r#type: TransactionType::Deposit,
            amount: dec!(100),
            store_balance_delta: dec!(79),
            user_balance_after: Some(dec!(200)),
            store_balance_after: dec!(1000),
            platform_commission: dec!(1),
            gateway_commission: dec!(20),
            created_at: now,
            description: None,
            payment_gateway: Some(PaymentSystem::PlatformCard),
            details: Some(json!({"ref": "abc"})),
        };

        let service = PaymentProcessingService::new(
            Arc::new(FakeTransactionService {
                last_created: Mutex::new(None),
                created_row: Mutex::new(Some(transaction_row)),
            }),
            Arc::new(FakePaymentInvoiceService {
                invoice: invoice.clone(),
                updated: Mutex::new(None),
            }),
            Arc::new(FakeNotificationService {
                last: Mutex::new(None),
            }),
            Arc::new(FakeCustomerService {
                customer: customer.clone(),
            }),
        );

        service.handle_payment_success(order_id).await.unwrap();

        let tx = service
            .transactions_service
            .as_ref()
            .last_created
            .lock()
            .unwrap()
            .take()
            .expect("transaction created");
        assert_eq!(tx.amount, invoice.original_amount);
        assert_eq!(tx.r#type, TransactionType::Deposit);
        assert_eq!(tx.customer_id, Some(customer.id));
        assert_eq!(tx.payment_gateway, Some(invoice.gateway));
        assert_eq!(tx.details, Some(invoice.payment_details.clone()));
        assert_eq!(tx.platform_commission, dec!(1));
        assert_eq!(tx.gateway_commission, dec!(20));
        assert_eq!(tx.store_balance_delta, dec!(79));

        let updated_guard = service
            .payment_invoice_service
            .as_ref()
            .updated
            .lock()
            .unwrap();
        let updated = updated_guard.as_ref().expect("invoice updated");
        assert_eq!(updated.id, invoice.id);
        assert_eq!(updated.status, Some(InvoiceStatus::Completed));

        let notify = service
            .notification_service
            .as_ref()
            .last
            .lock()
            .unwrap()
            .take()
            .expect("notification sent");
        assert_eq!(notify.bot_id, customer.last_seen_with_bot);
        assert_eq!(notify.telegram_id, customer.telegram_id);
        match notify.message {
            DispatchMessage::GenericMessage { message, .. } => {
                assert!(message.contains("100"));
                assert!(message.contains("RUB"));
            }
            _ => panic!("unexpected notification type"),
        }
    }
}
