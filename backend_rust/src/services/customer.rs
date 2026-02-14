use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use shared_dtos::audit_log::{AuditAction, AuditStatus};

use crate::{
    errors::api::{ApiError, ApiResult},
    infrastructure::repositories::{
        audit_log::AuditLogRepository,
        customer::{CustomerRepository, CustomerRepositoryTrait},
    },
    middlewares::context::RequestContext,
    models::{
        audit_log::NewAuditLog,
        common::PaginatedResult,
        customer::{CustomerListQuery, CustomerRow, NewCustomer, UpdateCustomer},
    },
    services::audit_log::{AuditLogService, AuditLogServiceTrait},
};

#[derive(Debug, Default)]
pub struct UpdateCustomerCommand {
    pub id: i64,
    pub is_blocked: Option<bool>,
    pub bot_is_blocked_by_user: Option<bool>,
    pub has_passed_captcha: Option<bool>,
    pub last_seen_with_bot: Option<i64>,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub updated_by: Option<i64>,
    pub blocked_until: Option<Option<DateTime<Utc>>>,
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
    async fn get_list_by_ids(&self, ids: &[i64]) -> ApiResult<Vec<CustomerRow>>;
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
                    blocked_until: command.blocked_until,
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
                    blocked_until: None,
                },
            )
            .await?;

        Ok(updated)
    }

    async fn get_list_by_ids(&self, ids: &[i64]) -> ApiResult<Vec<CustomerRow>> {
        let res = self.customer_repo.get_list_by_ids(ids).await?;
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::repositories::{
        audit_log::AuditLogRepository, customer::CustomerRepository,
    };
    use rust_decimal::Decimal;
    use sqlx::PgPool;
    use std::sync::Arc;

    fn build_service(
        pool: &PgPool,
    ) -> CustomerService<CustomerRepository, AuditLogService<AuditLogRepository>> {
        let pool = Arc::new(pool.clone());
        let audit_log_service = Arc::new(AuditLogService::new(Arc::new(AuditLogRepository::new(
            pool.clone(),
        ))));
        CustomerService::new(
            Arc::new(CustomerRepository::new(pool.clone())),
            audit_log_service,
        )
    }

    async fn create_customer(pool: &PgPool, telegram_id: i64) -> CustomerRow {
        sqlx::query_as!(
            CustomerRow,
            r#"
            INSERT INTO customers (
                telegram_id, registered_with_bot, last_seen_with_bot, balance
            )
            VALUES ($1, 1, 1, $2)
            RETURNING *
            "#,
            telegram_id,
            Decimal::ZERO
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    #[sqlx::test]
    async fn test_update_customer_flags(pool: PgPool) {
        let service = build_service(&pool);
        let customer = create_customer(&pool, 5001).await;

        let updated = service
            .update(UpdateCustomerCommand {
                id: customer.id,
                is_blocked: Some(true),
                bot_is_blocked_by_user: Some(true),
                has_passed_captcha: Some(true),
                last_seen_with_bot: None,
                last_seen_at: None,
                updated_by: None,
                ctx: None,
                blocked_until: None,
            })
            .await
            .unwrap();

        assert!(updated.is_blocked);
        assert!(updated.bot_is_blocked_by_user);
        assert!(updated.has_passed_captcha);
    }

    #[sqlx::test]
    async fn test_update_last_seen(pool: PgPool) {
        let service = build_service(&pool);
        let customer = create_customer(&pool, 5002).await;

        let updated = service.update_last_seen(customer.id, 777).await.unwrap();

        assert_eq!(updated.last_seen_with_bot, 777);
        assert!(updated.last_seen_at > customer.last_seen_at);
    }

    #[sqlx::test]
    async fn test_get_list_by_ids(pool: PgPool) {
        let service = build_service(&pool);
        let c1 = create_customer(&pool, 6001).await;
        let c2 = create_customer(&pool, 6002).await;

        let customers = service.get_list_by_ids(&[c1.id, c2.id]).await.unwrap();
        let ids: Vec<i64> = customers.into_iter().map(|c| c.id).collect();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&c1.id));
        assert!(ids.contains(&c2.id));
    }
}
