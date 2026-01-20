use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal_macros::dec;
use uuid::Uuid;

use crate::{
    errors::api::{ApiError, ApiResult},
    infrastructure::{
        external::payment::mock::{
            MockPaymentsProvider, MockPaymentsProviderTrait, dto::MockProviderCreateInvoiceRequest,
        },
        repositories::{
            audit_log::AuditLogRepository,
            payment_invoice::{PaymentInvoiceRepository, PaymentInvoiceRepositoryTrait},
            settings::{SettingsRepository, SettingsRepositoryTrait},
        },
    },
    models::{
        common::PaginatedResult,
        payment::PaymentSystem,
        payment_invoice::{
            InvoiceStatus, NewPaymentInvoice, PaymentInvoiceListQuery, PaymentInvoiceRow,
            UpdatePaymentInvoice,
        },
    },
    services::audit_log::{AuditLogService, AuditLogServiceTrait},
};
use serde_json::json;

#[derive(Debug)]
pub struct CreatePaymentInvoiceCommand {
    pub customer_id: i64,
    pub amount: Decimal,
    pub gateway: PaymentSystem,
}

#[derive(Debug)]
pub struct UpdatePaymentInvoiceCommand {
    pub id: i64,
    pub status: Option<InvoiceStatus>,
    pub notification_sent_at: Option<Option<DateTime<Utc>>>,
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
}

pub struct PaymentInvoiceService<R, A, M, S> {
    repo: Arc<R>,
    settings_repo: Arc<S>,
    mock_payments_provider: Arc<M>,
    #[allow(dead_code)]
    audit_log_service: Arc<A>,
}

impl<R, A, M, S> PaymentInvoiceService<R, A, M, S>
where
    R: PaymentInvoiceRepositoryTrait + Send + Sync,
    A: AuditLogServiceTrait + Send + Sync,
    M: MockPaymentsProviderTrait + Send + Sync,
    S: SettingsRepositoryTrait + Send + Sync,
{
    pub fn new(
        repo: Arc<R>,
        settings_repo: Arc<S>,
        mock_payments_provider: Arc<M>,
        audit_log_service: Arc<A>,
    ) -> Self {
        Self {
            repo,
            settings_repo,
            mock_payments_provider,
            audit_log_service,
        }
    }
}

#[async_trait]
impl PaymentInvoiceServiceTrait
    for PaymentInvoiceService<
        PaymentInvoiceRepository,
        AuditLogService<AuditLogRepository>,
        MockPaymentsProvider,
        SettingsRepository,
    >
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
                .map(|r| (r.invoice_id.to_string(), json!({"pay_url": r.pay_url})))
                .map_err(ApiError::InternalServerError)?,
            PaymentSystem::PlatformCard => {
                return Err(ApiError::InternalServerError("Not implemented".to_string()));
            }
            PaymentSystem::PlatformSBP => {
                return Err(ApiError::InternalServerError("Not implemented".to_string()));
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
                payment_details,
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
        // TODO External providers
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

    async fn cancel_invoice(&self, id: i64) -> ApiResult<PaymentInvoiceRow> {
        // TODO External providers
        let res = self
            .repo
            .update(
                id,
                UpdatePaymentInvoice {
                    status: Some(InvoiceStatus::Failed),
                    notification_sent_at: None,
                },
            )
            .await?;

        Ok(res)
    }
}
