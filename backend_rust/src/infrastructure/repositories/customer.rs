use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{PgPool, Postgres, QueryBuilder};

use crate::push_updates;
use crate::{
    errors::repository::{RepositoryError, RepositoryResult},
    infrastructure::lib::query::{apply_filters, apply_list_query},
    models::{
        common::PaginatedResult,
        customer::{CustomerListQuery, CustomerRow, NewCustomer, UpdateCustomer},
    },
};

#[async_trait]
pub trait CustomerRepositoryTrait {
    async fn get_list(
        &self,
        query: CustomerListQuery,
    ) -> RepositoryResult<PaginatedResult<CustomerRow>>;
    async fn create(&self, customer: NewCustomer) -> RepositoryResult<CustomerRow>;
    async fn get_by_id(&self, id: i64) -> RepositoryResult<CustomerRow>;
    async fn get_by_telegram_id(&self, id: i64) -> RepositoryResult<CustomerRow>;
    async fn update(&self, id: i64, customer: UpdateCustomer) -> RepositoryResult<CustomerRow>;
}

#[derive(Clone)]
pub struct CustomerRepository {
    pool: Arc<PgPool>,
}

impl CustomerRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CustomerRepositoryTrait for CustomerRepository {
    async fn get_list(
        &self,
        query: CustomerListQuery,
    ) -> RepositoryResult<PaginatedResult<CustomerRow>> {
        let mut count_qb: QueryBuilder<Postgres> =
            QueryBuilder::new("SELECT COUNT(*) FROM customers");
        apply_filters(&mut count_qb, &query);

        let count_query = count_qb.build_query_scalar();
        let total: i64 = count_query.fetch_one(&*self.pool).await?;

        let mut query_builder: QueryBuilder<Postgres> =
            QueryBuilder::new("SELECT * FROM customers");
        apply_list_query(&mut query_builder, &query);
        let items_query = query_builder.build_query_as::<CustomerRow>();
        let items = items_query.fetch_all(&*self.pool).await?;
        Ok(PaginatedResult { items, total })
    }

    async fn create(&self, customer: NewCustomer) -> RepositoryResult<CustomerRow> {
        let result = sqlx::query_as!(
            CustomerRow,
            r#"
            INSERT INTO customers (telegram_id, registered_with_bot, last_seen_with_bot)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
            customer.telegram_id,
            customer.registered_with_bot,
            customer.registered_with_bot
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(result)
    }

    async fn get_by_id(&self, id: i64) -> RepositoryResult<CustomerRow> {
        let result = sqlx::query_as!(CustomerRow, "SELECT * FROM customers WHERE id = $1", id)
            .fetch_one(&*self.pool)
            .await?;

        Ok(result)
    }

    async fn get_by_telegram_id(&self, id: i64) -> RepositoryResult<CustomerRow> {
        let result = sqlx::query_as!(
            CustomerRow,
            "SELECT * FROM customers WHERE telegram_id = $1",
            id
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(result)
    }

    async fn update(&self, id: i64, customer: UpdateCustomer) -> RepositoryResult<CustomerRow> {
        let mut query_builder: QueryBuilder<Postgres> =
            QueryBuilder::new("UPDATE customers SET last_seen_with_bot = COALESCE(");

        query_builder.push_bind(customer.last_seen_with_bot);
        query_builder.push(", last_seen_with_bot)");

        push_updates!(
            query_builder,
            is_blocked => customer.is_blocked,
            bot_is_blocked_by_user => customer.bot_is_blocked_by_user,
            last_seen_at => customer.last_seen_at,
            has_passed_captcha => customer.has_passed_captcha
        );

        query_builder.push(" WHERE id = ");
        query_builder.push_bind(id);
        query_builder.push(" RETURNING *");

        let query = query_builder.build_query_as::<CustomerRow>();

        query
            .fetch_one(&*self.pool)
            .await
            .map_err(RepositoryError::from)
    }
}
