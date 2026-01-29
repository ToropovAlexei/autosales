use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal_macros::dec;
use shared_dtos::invoice::{InvoiceStatus, PaymentDetails, PaymentSystem};
use uuid::Uuid;

use crate::{
    errors::api::{ApiError, ApiResult},
    infrastructure::{
        external::payment::{
            autosales_platform::{
                AutosalesPlatformPaymentsProviderTrait,
                dto::{
                    AutosalesPlatformInitializeOrderRequest, AutosalesPlatformPaymentMethod,
                    AutosalesPlatformSendReceiptRequest,
                },
            },
            mock::{MockPaymentsProviderTrait, dto::MockProviderCreateInvoiceRequest},
        },
        repositories::{
            payment_invoice::PaymentInvoiceRepositoryTrait, settings::SettingsRepositoryTrait,
        },
    },
    models::{
        common::PaginatedResult,
        payment_invoice::{
            NewPaymentInvoice, PaymentInvoiceListQuery, PaymentInvoiceRow, UpdatePaymentInvoice,
        },
    },
    services::audit_log::AuditLogServiceTrait,
};

#[derive(Debug)]
pub struct CreatePaymentInvoiceCommand {
    pub customer_id: i64,
    pub amount: Decimal,
    pub gateway: PaymentSystem,
}

#[derive(Debug, Default)]
pub struct UpdatePaymentInvoiceCommand {
    pub id: i64,
    pub status: Option<InvoiceStatus>,
    pub notification_sent_at: Option<Option<DateTime<Utc>>>,
}

pub struct SendInvoiceReceiptCommand {
    pub id: i64,
    pub receipt_url: String,
}

#[async_trait]
pub trait PaymentInvoiceServiceTrait: Send + Sync {
    async fn get_list(
        &self,
        query: PaymentInvoiceListQuery,
    ) -> ApiResult<PaginatedResult<PaymentInvoiceRow>>;
    async fn create(&self, command: CreatePaymentInvoiceCommand) -> ApiResult<PaymentInvoiceRow>;
    async fn get_by_id(&self, id: i64) -> ApiResult<PaymentInvoiceRow>;
    async fn get_by_order_id(&self, order_id: Uuid) -> ApiResult<PaymentInvoiceRow>;
    async fn update(&self, command: UpdatePaymentInvoiceCommand) -> ApiResult<PaymentInvoiceRow>;
    async fn get_for_customer(&self, customer_id: i64) -> ApiResult<Vec<PaymentInvoiceRow>>;
    async fn expire_old_invoices(&self) -> ApiResult<u64>;
    async fn get_pending_invoices(
        &self,
        older_than: DateTime<Utc>,
    ) -> ApiResult<Vec<PaymentInvoiceRow>>;
    async fn mark_invoices_notified(&self, ids: &[i64]) -> ApiResult<u64>;
    async fn confirm_invoice(&self, id: i64) -> ApiResult<PaymentInvoiceRow>;
    async fn cancel_invoice(&self, id: i64) -> ApiResult<PaymentInvoiceRow>;
    async fn send_invoice_receipt(
        &self,
        command: SendInvoiceReceiptCommand,
    ) -> ApiResult<PaymentInvoiceRow>;
}

pub struct PaymentInvoiceService<R, A, M, S, P> {
    repo: Arc<R>,
    settings_repo: Arc<S>,
    mock_payments_provider: Arc<M>,
    platform_payments_provider: Arc<P>,
    #[allow(dead_code)]
    audit_log_service: Arc<A>,
}

impl<R, A, M, S, P> PaymentInvoiceService<R, A, M, S, P>
where
    R: PaymentInvoiceRepositoryTrait + Send + Sync,
    A: AuditLogServiceTrait + Send + Sync,
    M: MockPaymentsProviderTrait + Send + Sync,
    S: SettingsRepositoryTrait + Send + Sync,
    P: AutosalesPlatformPaymentsProviderTrait + Send + Sync,
{
    pub fn new(
        repo: Arc<R>,
        settings_repo: Arc<S>,
        mock_payments_provider: Arc<M>,
        audit_log_service: Arc<A>,
        platform_payments_provider: Arc<P>,
    ) -> Self {
        Self {
            repo,
            settings_repo,
            mock_payments_provider,
            audit_log_service,
            platform_payments_provider,
        }
    }
}

#[async_trait]
impl<R, A, M, S, P> PaymentInvoiceServiceTrait for PaymentInvoiceService<R, A, M, S, P>
where
    R: PaymentInvoiceRepositoryTrait + Send + Sync,
    A: AuditLogServiceTrait + Send + Sync,
    M: MockPaymentsProviderTrait + Send + Sync,
    S: SettingsRepositoryTrait + Send + Sync,
    P: AutosalesPlatformPaymentsProviderTrait + Send + Sync,
{
    async fn get_list(
        &self,
        query: PaymentInvoiceListQuery,
    ) -> ApiResult<PaginatedResult<PaymentInvoiceRow>> {
        self.repo.get_list(query).await.map_err(ApiError::from)
    }

    async fn create(&self, command: CreatePaymentInvoiceCommand) -> ApiResult<PaymentInvoiceRow> {
        let settings = self.settings_repo.load_settings().await?;
        let order_id = Uuid::new_v4();
        let discount = match command.gateway {
            PaymentSystem::Mock => settings.pricing_gateway_bonus_mock_provider,
            PaymentSystem::PlatformCard => settings.pricing_gateway_bonus_platform_card,
            PaymentSystem::PlatformSBP => settings.pricing_gateway_bonus_platform_sbp,
        };
        let amount = command.amount * (dec!(1) - discount / dec!(100));
        let amount_parsed = amount.to_f64().ok_or(ApiError::InternalServerError(
            "Failed to convert decimal".to_string(),
        ))?;
        let (gateway_invoice_id, payment_details) = match command.gateway {
            PaymentSystem::Mock => self
                .mock_payments_provider
                .create_invoide(MockProviderCreateInvoiceRequest {
                    amount: amount_parsed,
                    order_id,
                    user_id: command.customer_id,
                })
                .await
                .map(|r| {
                    (
                        r.invoice_id.to_string(),
                        PaymentDetails::Mock { pay_url: r.pay_url },
                    )
                })
                .map_err(ApiError::InternalServerError)?,
            PaymentSystem::PlatformCard | PaymentSystem::PlatformSBP => {
                let id_pay_method = match command.gateway {
                    PaymentSystem::PlatformCard => AutosalesPlatformPaymentMethod::Card,
                    PaymentSystem::PlatformSBP => AutosalesPlatformPaymentMethod::SBP,
                    _ => unreachable!(),
                };
                let invoice = self
                    .platform_payments_provider
                    .init_order(AutosalesPlatformInitializeOrderRequest {
                        amount: amount_parsed as i64,
                        id_pay_method,
                    })
                    .await
                    .map_err(ApiError::InternalServerError)?;

                let payment_details = match command.gateway {
                    PaymentSystem::PlatformCard => PaymentDetails::PlatformCard {
                        account_name: format!(
                            "{} {} {}",
                            invoice.data_people.surname,
                            invoice.data_people.name,
                            invoice.data_people.patronymic
                        ),
                        amount: invoice.data_mathematics.amount_pay,
                        bank_name: invoice.data_bank.name,
                        card_number: invoice.value,
                    },
                    PaymentSystem::PlatformSBP => PaymentDetails::PlatformSBP {
                        account_name: format!(
                            "{} {} {}",
                            invoice.data_people.surname,
                            invoice.data_people.name,
                            invoice.data_people.patronymic
                        ),
                        amount: invoice.data_mathematics.amount_pay,
                        bank_name: invoice.data_bank.name,
                        sbp_number: invoice.value,
                    },
                    _ => unreachable!(),
                };
                (invoice.object_token.clone(), payment_details)
            }
        };
        let created = self
            .repo
            .create(NewPaymentInvoice {
                amount,
                original_amount: command.amount,
                customer_id: command.customer_id,
                bot_message_id: None,
                expires_at: Utc::now() + chrono::Duration::days(1), // TODO
                gateway: command.gateway,
                gateway_invoice_id,
                order_id,
                payment_details: Some(payment_details),
                status: InvoiceStatus::Pending,
            })
            .await?;

        Ok(created)
    }

    async fn get_by_id(&self, id: i64) -> ApiResult<PaymentInvoiceRow> {
        let res = self.repo.get_by_id(id).await?;
        Ok(res)
    }

    async fn get_by_order_id(&self, order_id: Uuid) -> ApiResult<PaymentInvoiceRow> {
        let res = self.repo.get_by_order_id(order_id).await?;
        Ok(res)
    }

    async fn update(&self, command: UpdatePaymentInvoiceCommand) -> ApiResult<PaymentInvoiceRow> {
        let updated = self
            .repo
            .update(
                command.id,
                UpdatePaymentInvoice {
                    notification_sent_at: command.notification_sent_at,
                    status: command.status,
                },
            )
            .await?;

        Ok(updated)
    }

    async fn get_for_customer(&self, customer_id: i64) -> ApiResult<Vec<PaymentInvoiceRow>> {
        let res = self.repo.get_for_customer(customer_id).await?;
        Ok(res)
    }

    async fn expire_old_invoices(&self) -> ApiResult<u64> {
        let res = self.repo.expire_old_invoices().await?;
        Ok(res)
    }

    async fn get_pending_invoices(
        &self,
        older_than: DateTime<Utc>,
    ) -> ApiResult<Vec<PaymentInvoiceRow>> {
        let res = self.repo.get_pending_invoices(older_than).await?;
        Ok(res)
    }

    async fn mark_invoices_notified(&self, ids: &[i64]) -> ApiResult<u64> {
        let res = self.repo.mark_invoices_notified(ids).await?;
        Ok(res)
    }

    async fn confirm_invoice(&self, id: i64) -> ApiResult<PaymentInvoiceRow> {
        let invoice = self.get_by_id(id).await?;
        match invoice.gateway {
            PaymentSystem::PlatformCard | PaymentSystem::PlatformSBP => {
                self.platform_payments_provider
                    .process_order(invoice.gateway_invoice_id)
                    .await
                    .map_err(ApiError::InternalServerError)?;
                let res = self
                    .repo
                    .update(
                        id,
                        UpdatePaymentInvoice {
                            status: Some(InvoiceStatus::Processing),
                            notification_sent_at: None,
                        },
                    )
                    .await?;

                Ok(res)
            }
            PaymentSystem::Mock => {
                let res = self
                    .repo
                    .update(
                        id,
                        UpdatePaymentInvoice {
                            status: Some(InvoiceStatus::Completed),
                            notification_sent_at: None,
                        },
                    )
                    .await?;

                Ok(res)
            }
        }
    }

    async fn cancel_invoice(&self, id: i64) -> ApiResult<PaymentInvoiceRow> {
        let invoice = self.get_by_id(id).await?;
        match invoice.gateway {
            PaymentSystem::PlatformCard | PaymentSystem::PlatformSBP => {
                self.platform_payments_provider
                    .cancel_order(invoice.gateway_invoice_id)
                    .await
                    .map_err(ApiError::InternalServerError)?;
                let res = self
                    .repo
                    .update(
                        id,
                        UpdatePaymentInvoice {
                            status: Some(InvoiceStatus::Cancelled),
                            notification_sent_at: None,
                        },
                    )
                    .await?;

                Ok(res)
            }
            PaymentSystem::Mock => {
                let res = self
                    .repo
                    .update(
                        id,
                        UpdatePaymentInvoice {
                            status: Some(InvoiceStatus::Cancelled),
                            notification_sent_at: None,
                        },
                    )
                    .await?;

                Ok(res)
            }
        }
    }

    async fn send_invoice_receipt(
        &self,
        command: SendInvoiceReceiptCommand,
    ) -> ApiResult<PaymentInvoiceRow> {
        let invoice = self.get_by_id(command.id).await?;
        match invoice.gateway {
            PaymentSystem::Mock => Ok(invoice),
            PaymentSystem::PlatformCard | PaymentSystem::PlatformSBP => {
                self.platform_payments_provider
                    .send_receipt(AutosalesPlatformSendReceiptRequest {
                        object_token: invoice.gateway_invoice_id,
                        url_file: command.receipt_url,
                    })
                    .await
                    .map_err(ApiError::InternalServerError)?;

                let res = self
                    .repo
                    .update(
                        command.id,
                        UpdatePaymentInvoice {
                            status: Some(InvoiceStatus::ReceiptSubmitted),
                            notification_sent_at: None,
                        },
                    )
                    .await?;

                Ok(res)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use async_trait::async_trait;
    use chrono::Utc;
    use rust_decimal_macros::dec;
    use serde_json::json;
    use std::sync::Mutex;
    use uuid::Uuid;

    use crate::{
        errors::api::ApiError,
        infrastructure::{
            external::payment::{
                autosales_platform::dto::{
                    AutosalesPlatformOrderInitializedDataRequisite,
                    AutosalesPlatformOrderInitializedDataRequisiteBank,
                    AutosalesPlatformOrderInitializedDataRequisiteMathematics,
                    AutosalesPlatformOrderInitializedDataRequisitePeople,
                },
                mock::dto::MockProviderCreateInvoiceResponse,
            },
            repositories::settings::SettingsRepositoryTrait,
        },
        models::{common::PaginatedResult, settings::Settings},
        services::audit_log::AuditLogServiceTrait,
    };

    #[derive(Clone)]
    struct FakeSettingsRepo {
        settings: Settings,
    }

    #[async_trait]
    impl SettingsRepositoryTrait for FakeSettingsRepo {
        async fn load_settings(
            &self,
        ) -> Result<Settings, crate::errors::repository::RepositoryError> {
            Ok(self.settings.clone())
        }

        async fn update(
            &self,
            _update: crate::models::settings::UpdateSettings,
        ) -> Result<Settings, crate::errors::repository::RepositoryError> {
            Ok(self.settings.clone())
        }
    }

    struct FakeAuditLogService;

    #[async_trait]
    impl AuditLogServiceTrait for FakeAuditLogService {
        async fn create(
            &self,
            _log: crate::models::audit_log::NewAuditLog,
        ) -> crate::errors::api::ApiResult<crate::models::audit_log::AuditLogRow> {
            Err(ApiError::InternalServerError("not used".to_string()))
        }

        async fn get_list(
            &self,
            _query: crate::models::audit_log::AuditLogListQuery,
        ) -> crate::errors::api::ApiResult<PaginatedResult<crate::models::audit_log::AuditLogRow>>
        {
            Err(ApiError::InternalServerError("not used".to_string()))
        }
    }

    #[derive(Clone, Debug)]
    #[allow(dead_code)]
    struct PaymentInvoiceSnapshot {
        customer_id: i64,
        original_amount: Decimal,
        amount: Decimal,
        status: InvoiceStatus,
        expires_at: chrono::DateTime<chrono::Utc>,
        gateway: PaymentSystem,
        gateway_invoice_id: String,
        order_id: Uuid,
        payment_details: Option<PaymentDetails>,
        bot_message_id: Option<i64>,
    }

    struct FakeRepo {
        last_created: Mutex<Option<PaymentInvoiceSnapshot>>,
    }

    #[async_trait]
    impl PaymentInvoiceRepositoryTrait for FakeRepo {
        async fn get_list(
            &self,
            _query: PaymentInvoiceListQuery,
        ) -> Result<PaginatedResult<PaymentInvoiceRow>, crate::errors::repository::RepositoryError>
        {
            Err(crate::errors::repository::RepositoryError::QueryFailed(
                "not used".to_string(),
            ))
        }

        async fn create(
            &self,
            payment_invoice: NewPaymentInvoice,
        ) -> Result<PaymentInvoiceRow, crate::errors::repository::RepositoryError> {
            let now = Utc::now();
            let payment_details = payment_invoice
                .payment_details
                .as_ref()
                .map(|p| serde_json::to_value(p).unwrap_or_default())
                .unwrap_or_else(|| json!({}));
            *self.last_created.lock().unwrap() = Some(PaymentInvoiceSnapshot {
                customer_id: payment_invoice.customer_id,
                original_amount: payment_invoice.original_amount,
                amount: payment_invoice.amount,
                status: payment_invoice.status,
                expires_at: payment_invoice.expires_at,
                gateway: payment_invoice.gateway,
                gateway_invoice_id: payment_invoice.gateway_invoice_id.clone(),
                order_id: payment_invoice.order_id,
                payment_details: payment_invoice.payment_details.clone(),
                bot_message_id: payment_invoice.bot_message_id,
            });
            Ok(PaymentInvoiceRow {
                id: 1,
                customer_id: payment_invoice.customer_id,
                original_amount: payment_invoice.original_amount,
                amount: payment_invoice.amount,
                status: payment_invoice.status,
                created_at: now,
                updated_at: now,
                expires_at: payment_invoice.expires_at,
                deleted_at: None,
                gateway: payment_invoice.gateway,
                gateway_invoice_id: payment_invoice.gateway_invoice_id,
                order_id: payment_invoice.order_id,
                payment_details,
                bot_message_id: payment_invoice.bot_message_id,
                notification_sent_at: None,
            })
        }

        async fn update(
            &self,
            _id: i64,
            _payment_invoice: UpdatePaymentInvoice,
        ) -> Result<PaymentInvoiceRow, crate::errors::repository::RepositoryError> {
            Err(crate::errors::repository::RepositoryError::QueryFailed(
                "not used".to_string(),
            ))
        }

        async fn get_by_id(
            &self,
            _id: i64,
        ) -> Result<PaymentInvoiceRow, crate::errors::repository::RepositoryError> {
            Err(crate::errors::repository::RepositoryError::NotFound(
                "not used".to_string(),
            ))
        }

        async fn get_by_order_id(
            &self,
            _order_id: Uuid,
        ) -> Result<PaymentInvoiceRow, crate::errors::repository::RepositoryError> {
            Err(crate::errors::repository::RepositoryError::NotFound(
                "not used".to_string(),
            ))
        }

        async fn get_for_customer(
            &self,
            _customer_id: i64,
        ) -> Result<Vec<PaymentInvoiceRow>, crate::errors::repository::RepositoryError> {
            Err(crate::errors::repository::RepositoryError::QueryFailed(
                "not used".to_string(),
            ))
        }

        async fn expire_old_invoices(
            &self,
        ) -> Result<u64, crate::errors::repository::RepositoryError> {
            Err(crate::errors::repository::RepositoryError::QueryFailed(
                "not used".to_string(),
            ))
        }

        async fn get_pending_invoices(
            &self,
            _older_than: chrono::DateTime<chrono::Utc>,
        ) -> Result<Vec<PaymentInvoiceRow>, crate::errors::repository::RepositoryError> {
            Err(crate::errors::repository::RepositoryError::QueryFailed(
                "not used".to_string(),
            ))
        }

        async fn mark_invoices_notified(
            &self,
            _ids: &[i64],
        ) -> Result<u64, crate::errors::repository::RepositoryError> {
            Err(crate::errors::repository::RepositoryError::QueryFailed(
                "not used".to_string(),
            ))
        }
    }

    #[cfg(feature = "mock-payments-provider")]
    struct FakeMockProvider {
        last_request: Mutex<Option<MockProviderCreateInvoiceRequest>>,
    }

    #[cfg(feature = "mock-payments-provider")]
    #[async_trait]
    impl MockPaymentsProviderTrait for FakeMockProvider {
        async fn create_invoide(
            &self,
            req: MockProviderCreateInvoiceRequest,
        ) -> Result<MockProviderCreateInvoiceResponse, String> {
            *self.last_request.lock().unwrap() = Some(req);
            Ok(MockProviderCreateInvoiceResponse {
                invoice_id: Uuid::new_v4(),
                pay_url: "https://pay.example/test".to_string(),
            })
        }

        async fn get_invoice_status(
            &self,
            _invoice_id: Uuid,
        ) -> Result<
            crate::infrastructure::external::payment::mock::dto::MockProviderInvoiceStatus,
            String,
        > {
            Err("not used".to_string())
        }

        async fn handle_webhook(
            &self,
            _req: crate::infrastructure::external::payment::mock::dto::MockProviderInvoiceWebhookPayload,
        ) -> Result<Uuid, String> {
            Err("not used".to_string())
        }
    }

    struct DummyMockProvider;

    #[async_trait]
    impl MockPaymentsProviderTrait for DummyMockProvider {
        async fn create_invoide(
            &self,
            _req: MockProviderCreateInvoiceRequest,
        ) -> Result<MockProviderCreateInvoiceResponse, String> {
            Err("not used".to_string())
        }

        async fn get_invoice_status(
            &self,
            _invoice_id: Uuid,
        ) -> Result<
            crate::infrastructure::external::payment::mock::dto::MockProviderInvoiceStatus,
            String,
        > {
            Err("not used".to_string())
        }

        async fn handle_webhook(
            &self,
            _req: crate::infrastructure::external::payment::mock::dto::MockProviderInvoiceWebhookPayload,
        ) -> Result<Uuid, String> {
            Err("not used".to_string())
        }
    }

    struct FakePlatformProvider {
        last_request: Mutex<Option<AutosalesPlatformInitializeOrderRequest>>,
        response: Mutex<Option<AutosalesPlatformOrderInitializedDataRequisite>>,
    }

    #[async_trait]
    impl AutosalesPlatformPaymentsProviderTrait for FakePlatformProvider {
        async fn init_order(
            &self,
            req: AutosalesPlatformInitializeOrderRequest,
        ) -> Result<AutosalesPlatformOrderInitializedDataRequisite, String> {
            *self.last_request.lock().unwrap() = Some(req);
            Ok(self
                .response
                .lock()
                .unwrap()
                .take()
                .expect("platform response"))
        }

        async fn cancel_order(&self, _object_token: String) -> Result<(), String> {
            Err("not used".to_string())
        }

        async fn process_order(&self, _object_token: String) -> Result<(), String> {
            Err("not used".to_string())
        }

        async fn send_receipt(
            &self,
            _req: AutosalesPlatformSendReceiptRequest,
        ) -> Result<(), String> {
            Err("not used".to_string())
        }

        async fn get_order_status(
            &self,
            _object_token: String,
        ) -> Result<crate::infrastructure::external::payment::autosales_platform::dto::AutosalesPlatformOrderStatus, String>{
            Err("not used".to_string())
        }
    }

    fn base_settings() -> Settings {
        Settings {
            bot_messages_support: "support".to_string(),
            bot_messages_support_image_id: None,
            bot_messages_new_user_welcome: "welcome".to_string(),
            bot_messages_new_user_welcome_image_id: None,
            bot_messages_returning_user_welcome: "welcome back".to_string(),
            bot_messages_returning_user_welcome_image_id: None,
            pricing_global_markup: dec!(0),
            pricing_platform_commission: dec!(0),
            pricing_gateway_markup: dec!(0),
            pricing_gateway_bonus_mock_provider: dec!(0),
            pricing_gateway_bonus_platform_card: dec!(0),
            pricing_gateway_bonus_platform_sbp: dec!(0),
            referral_program_enabled: false,
            referral_percentage: dec!(0),
        }
    }

    #[cfg(feature = "mock-payments-provider")]
    #[tokio::test]
    async fn test_create_invoice_with_mock_discount() {
        let mut settings = base_settings();
        settings.pricing_gateway_bonus_mock_provider = dec!(10);
        let repo = Arc::new(FakeRepo {
            last_created: Mutex::new(None),
        });
        let provider = Arc::new(FakeMockProvider {
            last_request: Mutex::new(None),
        });
        let service = PaymentInvoiceService::new(
            repo.clone(),
            Arc::new(FakeSettingsRepo {
                settings: settings.clone(),
            }),
            provider.clone(),
            Arc::new(FakeAuditLogService),
            Arc::new(FakePlatformProvider {
                last_request: Mutex::new(None),
                response: Mutex::new(Some(AutosalesPlatformOrderInitializedDataRequisite {
                    object_token: "token".to_string(),
                    value: "value".to_string(),
                    data_bank: AutosalesPlatformOrderInitializedDataRequisiteBank {
                        name: "bank".to_string(),
                    },
                    data_people: AutosalesPlatformOrderInitializedDataRequisitePeople {
                        surname: "Doe".to_string(),
                        name: "John".to_string(),
                        patronymic: "P".to_string(),
                    },
                    data_mathematics: AutosalesPlatformOrderInitializedDataRequisiteMathematics {
                        currency: "RUB".to_string(),
                        country: "RU".to_string(),
                        amount_pay: 90.0,
                        amount_transfer: 90.0,
                    },
                })),
            }),
        );

        let res = service
            .create(CreatePaymentInvoiceCommand {
                customer_id: 10,
                amount: dec!(100),
                gateway: PaymentSystem::Mock,
            })
            .await
            .unwrap();

        let created = repo
            .last_created
            .lock()
            .unwrap()
            .take()
            .expect("invoice created");
        assert_eq!(created.original_amount, dec!(100));
        assert_eq!(created.amount, dec!(90));
        assert_eq!(created.gateway, PaymentSystem::Mock);
        match created.payment_details.unwrap() {
            PaymentDetails::Mock { pay_url } => {
                assert!(pay_url.contains("https://pay.example"));
            }
            _ => panic!("expected mock payment details"),
        }
        assert_eq!(res.amount, dec!(90));
    }

    #[tokio::test]
    async fn test_create_invoice_platform_card_details() {
        let mut settings = base_settings();
        settings.pricing_gateway_bonus_platform_card = dec!(0);
        let repo = Arc::new(FakeRepo {
            last_created: Mutex::new(None),
        });
        let platform = Arc::new(FakePlatformProvider {
            last_request: Mutex::new(None),
            response: Mutex::new(Some(AutosalesPlatformOrderInitializedDataRequisite {
                object_token: "token-123".to_string(),
                value: "4111111111111111".to_string(),
                data_bank: AutosalesPlatformOrderInitializedDataRequisiteBank {
                    name: "TestBank".to_string(),
                },
                data_people: AutosalesPlatformOrderInitializedDataRequisitePeople {
                    surname: "Doe".to_string(),
                    name: "Jane".to_string(),
                    patronymic: "Q".to_string(),
                },
                data_mathematics: AutosalesPlatformOrderInitializedDataRequisiteMathematics {
                    currency: "RUB".to_string(),
                    country: "RU".to_string(),
                    amount_pay: 100.0,
                    amount_transfer: 100.0,
                },
            })),
        });

        let service = PaymentInvoiceService::new(
            repo.clone(),
            Arc::new(FakeSettingsRepo {
                settings: settings.clone(),
            }),
            Arc::new(DummyMockProvider),
            Arc::new(FakeAuditLogService),
            platform.clone(),
        );

        let res = service
            .create(CreatePaymentInvoiceCommand {
                customer_id: 10,
                amount: dec!(100),
                gateway: PaymentSystem::PlatformCard,
            })
            .await
            .unwrap();

        let created = repo
            .last_created
            .lock()
            .unwrap()
            .take()
            .expect("invoice created");
        match created.payment_details.unwrap() {
            PaymentDetails::PlatformCard {
                bank_name,
                account_name,
                card_number,
                amount,
            } => {
                assert_eq!(bank_name, "TestBank");
                assert_eq!(account_name, "Doe Jane Q");
                assert_eq!(card_number, "4111111111111111");
                assert_eq!(amount, 100.0);
            }
            _ => panic!("expected platform card details"),
        }

        let request = platform
            .last_request
            .lock()
            .unwrap()
            .take()
            .expect("platform request");
        assert_eq!(request.amount, 100);
        assert!(matches!(
            request.id_pay_method,
            AutosalesPlatformPaymentMethod::Card
        ));
        assert_eq!(res.gateway, PaymentSystem::PlatformCard);
    }
}
