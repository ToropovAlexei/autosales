use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    errors::api::{ApiError, ApiResult},
    infrastructure::repositories::transaction::{
        TransactionRepository, TransactionRepositoryTrait,
    },
    models::{
        common::PaginatedResult,
        transaction::{NewTransaction, TransactionListQuery, TransactionRow},
    },
};

#[async_trait]
pub trait TransactionServiceTrait: Send + Sync {
    async fn get_list(
        &self,
        query: TransactionListQuery,
    ) -> ApiResult<PaginatedResult<TransactionRow>>;
    async fn create(&self, transaction: NewTransaction) -> ApiResult<TransactionRow>;
    async fn get_last(&self) -> ApiResult<TransactionRow>;
}

pub struct TransactionService<R> {
    repo: Arc<R>,
}

impl<R> TransactionService<R>
where
    R: TransactionRepositoryTrait + Send + Sync,
{
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl TransactionServiceTrait for TransactionService<TransactionRepository> {
    async fn get_list(
        &self,
        query: TransactionListQuery,
    ) -> ApiResult<PaginatedResult<TransactionRow>> {
        self.repo.get_list(query).await.map_err(ApiError::from)
    }

    async fn create(&self, transaction: NewTransaction) -> ApiResult<TransactionRow> {
        let created = self.repo.create(transaction).await?;
        Ok(created)
    }

    async fn get_last(&self) -> ApiResult<TransactionRow> {
        self.repo.get_last().await.map_err(ApiError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::repositories::transaction::TransactionRepository;
    use rust_decimal::Decimal;
    use shared_dtos::transaction::TransactionType;
    use sqlx::PgPool;
    use std::sync::Arc;

    async fn create_customer(pool: &PgPool, telegram_id: i64) -> i64 {
        sqlx::query_scalar!(
            "INSERT INTO customers (telegram_id, registered_with_bot, last_seen_with_bot) VALUES ($1, 1, 1) RETURNING id",
            telegram_id
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    fn build_service(pool: &PgPool) -> TransactionService<TransactionRepository> {
        let pool = Arc::new(pool.clone());
        TransactionService::new(Arc::new(TransactionRepository::new(pool)))
    }

    #[sqlx::test]
    async fn test_create_and_get_last(pool: PgPool) {
        let service = build_service(&pool);
        let customer_id = create_customer(&pool, 7001).await;

        service
            .create(NewTransaction {
                customer_id: Some(customer_id),
                order_id: None,
                r#type: TransactionType::Deposit,
                amount: Decimal::from(100),
                store_balance_delta: Decimal::from(100),
                platform_commission: Decimal::ZERO,
                gateway_commission: Decimal::ZERO,
                description: None,
                payment_gateway: None,
                details: None,
                bot_id: None,
            })
            .await
            .unwrap();

        let last = service
            .create(NewTransaction {
                customer_id: Some(customer_id),
                order_id: None,
                r#type: TransactionType::Purchase,
                amount: Decimal::from(-25),
                store_balance_delta: Decimal::from(25),
                platform_commission: Decimal::ZERO,
                gateway_commission: Decimal::ZERO,
                description: Some("purchase".to_string()),
                payment_gateway: None,
                details: None,
                bot_id: None,
            })
            .await
            .unwrap();

        let fetched = service.get_last().await.unwrap();
        assert_eq!(fetched.id, last.id);
        assert_eq!(fetched.r#type, TransactionType::Purchase);
    }

    #[sqlx::test]
    async fn test_get_list(pool: PgPool) {
        let service = build_service(&pool);
        let customer_id = create_customer(&pool, 7002).await;

        service
            .create(NewTransaction {
                customer_id: Some(customer_id),
                order_id: None,
                r#type: TransactionType::Deposit,
                amount: Decimal::from(100),
                store_balance_delta: Decimal::from(100),
                platform_commission: Decimal::ZERO,
                gateway_commission: Decimal::ZERO,
                description: None,
                payment_gateway: None,
                details: None,
                bot_id: None,
            })
            .await
            .unwrap();

        let result = service
            .get_list(TransactionListQuery::default())
            .await
            .unwrap();
        assert!(result.total >= 1);
        assert!(!result.items.is_empty());
    }
}
