use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{PgPool, Postgres, QueryBuilder};

use crate::{
    errors::repository::RepositoryResult,
    infrastructure::lib::query::{apply_filters, apply_list_query},
    models::{
        common::{ListQuery, PaginatedResult},
        transaction::{NewTransaction, TransactionRow},
    },
};

#[async_trait]
pub trait TransactionRepositoryTrait {
    async fn get_list(&self, query: ListQuery)
    -> RepositoryResult<PaginatedResult<TransactionRow>>;
    async fn create(&self, category: NewTransaction) -> RepositoryResult<TransactionRow>;
}

#[derive(Clone)]
pub struct TransactionRepository {
    pool: Arc<PgPool>,
}

impl TransactionRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TransactionRepositoryTrait for TransactionRepository {
    async fn get_list(
        &self,
        query: ListQuery,
    ) -> RepositoryResult<PaginatedResult<TransactionRow>> {
        let mut count_qb: QueryBuilder<Postgres> =
            QueryBuilder::new("SELECT COUNT(*) FROM transactions");
        apply_filters(&mut count_qb, &query);

        let count_query = count_qb.build_query_scalar();
        let total: i64 = count_query.fetch_one(&*self.pool).await?;

        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"
        SELECT
            id, customer_id, order_id, type as "type: _", amount, store_balance_delta, 
            platform_commission, gateway_commission, description, payment_gateway,
            details, created_at, store_balance_after, user_balance_after
        FROM transactions"#,
        );
        apply_list_query(&mut query_builder, &query);
        let query = query_builder.build_query_as::<TransactionRow>();
        let items = query.fetch_all(&*self.pool).await?;
        Ok(PaginatedResult { items, total })
    }

    async fn create(&self, transaction: NewTransaction) -> RepositoryResult<TransactionRow> {
        let result = sqlx::query_as!(
            TransactionRow,
            r#"
            INSERT INTO transactions (
                customer_id, order_id, type, amount, store_balance_delta, 
                platform_commission, gateway_commission,
                description, payment_gateway, details
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING 
                id, customer_id, order_id, type as "type: _", amount, store_balance_delta, 
                platform_commission, gateway_commission, description, payment_gateway,
                details, created_at, store_balance_after, user_balance_after
            "#,
            transaction.customer_id,
            transaction.order_id,
            transaction.r#type as _,
            transaction.amount,
            transaction.store_balance_delta,
            transaction.platform_commission,
            transaction.gateway_commission,
            transaction.description,
            transaction.payment_gateway,
            transaction.details
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(result)
    }
}
