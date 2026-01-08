use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{PgPool, Postgres, QueryBuilder};

use crate::{
    errors::repository::RepositoryResult,
    infrastructure::lib::query::{apply_filters, apply_list_query},
    models::{
        common::PaginatedResult,
        transaction::{NewTransaction, TransactionListQuery, TransactionRow},
    },
};

#[async_trait]
pub trait TransactionRepositoryTrait {
    async fn get_list(
        &self,
        query: TransactionListQuery,
    ) -> RepositoryResult<PaginatedResult<TransactionRow>>;
    async fn create(&self, category: NewTransaction) -> RepositoryResult<TransactionRow>;
    async fn get_last(&self) -> RepositoryResult<TransactionRow>;
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
        query: TransactionListQuery,
    ) -> RepositoryResult<PaginatedResult<TransactionRow>> {
        let mut count_qb: QueryBuilder<Postgres> =
            QueryBuilder::new("SELECT COUNT(*) FROM transactions");
        apply_filters(&mut count_qb, &query);

        let count_query = count_qb.build_query_scalar();
        let total: i64 = count_query.fetch_one(&*self.pool).await?;

        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"
        SELECT
            id, customer_id, order_id, type, amount, store_balance_delta,
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

    async fn get_last(&self) -> RepositoryResult<TransactionRow> {
        let result = sqlx::query_as!(
            TransactionRow,
            r#"
            SELECT
                id, customer_id, order_id, type as "type: _", amount, store_balance_delta,
                platform_commission, gateway_commission, description, payment_gateway,
                details, created_at, store_balance_after, user_balance_after
            FROM transactions
            ORDER BY id DESC
            LIMIT 1
            "#,
        )
        .fetch_one(&*self.pool)
        .await?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::models::transaction::TransactionType;

    use super::*;
    use bigdecimal::BigDecimal;

    #[sqlx::test]
    async fn test_balance_calculation_triggers(pool: PgPool) {
        let customer_id: i64 = sqlx::query!(
            r#"INSERT INTO customers (telegram_id, registered_with_bot, last_seen_with_bot) VALUES ($1, $2, $3) RETURNING id"#,
            12345,
            1,
            1
        )
        .fetch_one(&pool)
        .await
        .unwrap()
        .id;

        let repo = TransactionRepository::new(Arc::new(pool.clone()));

        // Deposit transaction
        let deposit_tx = NewTransaction {
            customer_id: Some(customer_id),
            order_id: None,
            r#type: TransactionType::Deposit,
            amount: BigDecimal::from(1000),
            store_balance_delta: BigDecimal::from(1000),
            platform_commission: BigDecimal::from(0),
            gateway_commission: BigDecimal::from(0),
            description: Some("Test deposit".to_string()),
            payment_gateway: Some("test_gw".to_string()),
            details: None,
        };

        let result1 = repo.create(deposit_tx).await.unwrap();

        assert_eq!(result1.user_balance_after, Some(BigDecimal::from(1000)));
        assert_eq!(result1.store_balance_after, BigDecimal::from(1000));

        // Verify customer balance after deposit
        let customer = sqlx::query_as!(
            crate::models::customer::CustomerRow, // Full path for CustomerRow
            "SELECT * FROM customers WHERE id = $1",
            customer_id
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(customer.balance, BigDecimal::from(1000));

        // Purchase transaction
        let purchase_tx = NewTransaction {
            customer_id: Some(customer_id),
            order_id: None,
            r#type: TransactionType::Purchase,
            amount: BigDecimal::from(-150),
            store_balance_delta: BigDecimal::from(150),
            platform_commission: BigDecimal::from(0),
            gateway_commission: BigDecimal::from(0),
            description: Some("Test purchase".to_string()),
            payment_gateway: None,
            details: None,
        };

        let result2 = repo.create(purchase_tx).await.unwrap();

        assert_eq!(result2.user_balance_after, Some(BigDecimal::from(850))); // 1000 - 150
        assert_eq!(result2.store_balance_after, BigDecimal::from(1150)); // 1000 + 150

        // Verify customer balance after purchase
        let customer = sqlx::query_as!(
            crate::models::customer::CustomerRow, // Full path for CustomerRow
            "SELECT * FROM customers WHERE id = $1",
            customer_id
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(customer.balance, BigDecimal::from(850));
    }
}
