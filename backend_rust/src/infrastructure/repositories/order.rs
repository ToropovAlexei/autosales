use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{PgPool, Postgres, QueryBuilder};

use crate::{
    errors::repository::RepositoryResult,
    infrastructure::lib::query::{apply_filters, apply_list_query},
    models::{
        common::PaginatedResult,
        order::{NewOrder, OrderListQuery, OrderRow, OrderStatus},
    },
};

#[async_trait]
pub trait OrderRepositoryTrait {
    async fn get_list(&self, query: OrderListQuery) -> RepositoryResult<PaginatedResult<OrderRow>>;
    async fn create(&self, order: NewOrder) -> RepositoryResult<OrderRow>;
    async fn get_by_id(&self, id: i64) -> RepositoryResult<OrderRow>;
}

#[derive(Clone)]
pub struct OrderRepository {
    pool: Arc<PgPool>,
}

impl OrderRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OrderRepositoryTrait for OrderRepository {
    async fn get_list(&self, query: OrderListQuery) -> RepositoryResult<PaginatedResult<OrderRow>> {
        let mut count_qb: QueryBuilder<Postgres> = QueryBuilder::new("SELECT COUNT(*) FROM orders");
        apply_filters(&mut count_qb, &query);

        let count_query = count_qb.build_query_scalar();
        let total: i64 = count_query.fetch_one(&*self.pool).await?;

        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new("SELECT * FROM orders");
        apply_list_query(&mut query_builder, &query);
        let query = query_builder.build_query_as::<OrderRow>();
        let items = query.fetch_all(&*self.pool).await?;
        Ok(PaginatedResult { items, total })
    }

    async fn create(&self, order: NewOrder) -> RepositoryResult<OrderRow> {
        let result = sqlx::query_as!(
            OrderRow,
            r#"
            INSERT INTO orders (
                customer_id, amount, currency, status, bot_id, paid_at, fulfilled_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING
                id, customer_id, amount, currency, status as "status: _", bot_id,
                created_at, updated_at, paid_at, fulfilled_at, cancelled_at
            "#,
            order.customer_id,
            order.amount,
            order.currency,
            order.status as OrderStatus,
            order.bot_id,
            order.paid_at,
            order.fulfilled_at,
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(result)
    }

    async fn get_by_id(&self, id: i64) -> RepositoryResult<OrderRow> {
        let result = sqlx::query_as!(
            OrderRow,
            r#"
        SELECT 
            id, customer_id, amount, currency, status as "status: _", bot_id,
            created_at, updated_at, paid_at, fulfilled_at, cancelled_at
        FROM orders WHERE id = $1"#,
            id
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::common::{OrderDir, Pagination};
    use bigdecimal::BigDecimal;
    use sqlx::PgPool;

    async fn create_test_customer(pool: &PgPool, telegram_id: i64) -> i64 {
        sqlx::query_scalar!(
            "INSERT INTO customers (telegram_id, registered_with_bot, last_seen_with_bot) VALUES ($1, 1, 1) RETURNING id",
            telegram_id
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn create_test_bot(pool: &PgPool, token: &str, username: &str) -> i64 {
        sqlx::query_scalar!(
            r#"INSERT INTO bots (owner_id, token, username, type, is_active, is_primary, referral_percentage, created_by) VALUES (1, $1, $2, 'main', true, false, 0.1, 1) RETURNING id"#,
            token,
            username
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    #[sqlx::test]
    async fn test_create_and_get_order(pool: PgPool) {
        let repo = OrderRepository::new(Arc::new(pool.clone()));
        let customer_id = create_test_customer(&pool, 54321).await;
        let bot_id = create_test_bot(&pool, "order_bot", "order_bot").await;

        let new_order = NewOrder {
            customer_id,
            amount: BigDecimal::from(100),
            currency: "USD".to_string(),
            status: OrderStatus::Created,
            bot_id,
            paid_at: None,
            fulfilled_at: None,
        };

        // Create an order
        let created_order = repo.create(new_order).await.unwrap();
        assert_eq!(created_order.customer_id, customer_id);

        // Get by id
        let fetched_order = repo.get_by_id(created_order.id).await.unwrap();
        assert_eq!(fetched_order.id, created_order.id);
    }

    #[sqlx::test]
    async fn test_get_list_orders(pool: PgPool) {
        let repo = OrderRepository::new(Arc::new(pool.clone()));
        let customer_id1 = create_test_customer(&pool, 11111).await;
        let customer_id2 = create_test_customer(&pool, 22222).await;
        let bot_id = create_test_bot(&pool, "order_bot_2", "order_bot_2").await;

        // Create some orders
        repo.create(NewOrder {
            customer_id: customer_id1,
            amount: BigDecimal::from(100),
            currency: "USD".to_string(),
            status: OrderStatus::Created,
            bot_id,
            paid_at: None,
            fulfilled_at: None,
        })
        .await
        .unwrap();

        repo.create(NewOrder {
            customer_id: customer_id2,
            amount: BigDecimal::from(200),
            currency: "USD".to_string(),
            status: OrderStatus::Created,
            bot_id,
            paid_at: None,
            fulfilled_at: None,
        })
        .await
        .unwrap();

        // Get the list of orders
        let query = OrderListQuery {
            filters: vec![],
            pagination: Pagination {
                page: 1,
                page_size: 10,
            },
            order_by: None,
            order_dir: OrderDir::default(),
            _phantom: std::marker::PhantomData,
        };
        let orders = repo.get_list(query).await.unwrap();
        assert!(!orders.items.is_empty());
        assert!(orders.total >= 2);
    }
}
