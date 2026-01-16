use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::{
    errors::api::{ApiError, ApiResult},
    infrastructure::repositories::{
        audit_log::AuditLogRepository,
        customer::{CustomerRepository, CustomerRepositoryTrait},
    },
    middlewares::context::RequestContext,
    models::{
        audit_log::{AuditAction, AuditStatus, NewAuditLog},
        common::PaginatedResult,
        customer::{CustomerListQuery, CustomerRow, NewCustomer, UpdateCustomer},
    },
    services::audit_log::{AuditLogService, AuditLogServiceTrait},
};

#[derive(Debug)]
pub struct UpdateCustomerCommand {
    pub id: i64,
    pub is_blocked: Option<bool>,
    pub bot_is_blocked_by_user: Option<bool>,
    pub has_passed_captcha: Option<bool>,
    pub last_seen_with_bot: Option<i64>,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub updated_by: Option<i64>,
    pub ctx: Option<RequestContext>,
}

#[async_trait]
pub trait CustomerServiceTrait: Send + Sync {
    async fn get_list(&self, query: CustomerListQuery) -> ApiResult<PaginatedResult<CustomerRow>>;
    async fn create(&self, customer: NewCustomer) -> ApiResult<CustomerRow>;
    async fn get_by_id(&self, id: i64) -> ApiResult<CustomerRow>;
    async fn get_by_telegram_id(&self, id: i64) -> ApiResult<CustomerRow>;
    async fn update(&self, command: UpdateCustomerCommand) -> ApiResult<CustomerRow>;
    async fn update_last_seen(&self, id: i64, bot_id: i64) -> ApiResult<CustomerRow>;
}

pub struct CustomerService<R, A> {
    customer_repo: Arc<R>,
    audit_log_service: Arc<A>,
}

impl<R, A> CustomerService<R, A>
where
    R: CustomerRepositoryTrait + Send + Sync,
    A: AuditLogServiceTrait + Send + Sync,
{
    pub fn new(customer_repo: Arc<R>, audit_log_service: Arc<A>) -> Self {
        Self {
            customer_repo,
            audit_log_service,
        }
    }
}

#[async_trait]
impl CustomerServiceTrait
    for CustomerService<CustomerRepository, AuditLogService<AuditLogRepository>>
{
    async fn get_list(&self, query: CustomerListQuery) -> ApiResult<PaginatedResult<CustomerRow>> {
        self.customer_repo
            .get_list(query)
            .await
            .map_err(ApiError::from)
    }

    async fn create(&self, customer: NewCustomer) -> ApiResult<CustomerRow> {
        let created = self.customer_repo.create(customer).await?;

        Ok(created)
    }

    async fn get_by_id(&self, id: i64) -> ApiResult<CustomerRow> {
        let res = self.customer_repo.get_by_id(id).await?;
        Ok(res)
    }

    async fn get_by_telegram_id(&self, id: i64) -> ApiResult<CustomerRow> {
        let res = self.customer_repo.get_by_telegram_id(id).await?;
        Ok(res)
    }

    async fn update(&self, command: UpdateCustomerCommand) -> ApiResult<CustomerRow> {
        let prev = self.customer_repo.get_by_id(command.id).await?;
        let updated = self
            .customer_repo
            .update(
                command.id,
                UpdateCustomer {
                    bot_is_blocked_by_user: command.bot_is_blocked_by_user,
                    has_passed_captcha: command.has_passed_captcha,
                    is_blocked: command.is_blocked,
                    last_seen_at: command.last_seen_at,
                    last_seen_with_bot: command.last_seen_with_bot,
                },
            )
            .await?;

        self.audit_log_service
            .create(NewAuditLog {
                action: AuditAction::CustomerUpdate,
                status: AuditStatus::Success,
                admin_user_id: command.updated_by,
                customer_id: None,
                error_message: None,
                new_values: serde_json::to_value(updated.clone()).ok(),
                old_values: serde_json::to_value(prev.clone()).ok(),
                target_id: command.id.to_string(),
                target_table: "customers".to_string(),
                ip_address: command.ctx.clone().and_then(|ctx| ctx.ip_address),
                request_id: command.ctx.clone().map(|ctx| ctx.request_id),
                user_agent: command.ctx.and_then(|ctx| ctx.user_agent),
            })
            .await?;

        Ok(updated)
    }

    async fn update_last_seen(&self, id: i64, bot_id: i64) -> ApiResult<CustomerRow> {
        let updated = self
            .customer_repo
            .update(
                id,
                UpdateCustomer {
                    bot_is_blocked_by_user: None,
                    has_passed_captcha: None,
                    is_blocked: None,
                    last_seen_at: Some(Utc::now()),
                    last_seen_with_bot: Some(bot_id),
                },
            )
            .await?;

        Ok(updated)
    }
}
