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
