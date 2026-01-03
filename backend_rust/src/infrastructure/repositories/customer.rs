use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{PgPool, Postgres, QueryBuilder};

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

        if let Some(last_seen_at) = customer.last_seen_at {
            query_builder.push(", last_seen_at = ");
            query_builder.push_bind(last_seen_at);
        }

        if let Some(has_passed_captcha) = customer.has_passed_captcha {
            query_builder.push(", has_passed_captcha = ");
            query_builder.push_bind(has_passed_captcha);
        }

        if let Some(bot_is_blocked_by_user) = customer.bot_is_blocked_by_user {
            query_builder.push(", bot_is_blocked_by_user = ");
            query_builder.push_bind(bot_is_blocked_by_user);
        }

        if let Some(is_blocked) = customer.is_blocked {
            query_builder.push(", is_blocked = ");
            query_builder.push_bind(is_blocked);
        }

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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use sqlx::PgPool;

    async fn create_test_customer(
        pool: &PgPool,
        telegram_id: i64,
        registered_with_bot: i64,
    ) -> CustomerRow {
        sqlx::query_as!(
            CustomerRow,
            r#"
            INSERT INTO customers (telegram_id, registered_with_bot, last_seen_with_bot)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
            telegram_id,
            registered_with_bot,
            registered_with_bot
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    #[sqlx::test]
    async fn test_update_customer_all_some(pool: PgPool) {
        let repo = CustomerRepository::new(Arc::new(pool.clone()));

        // Create a customer
        let initial_customer = create_test_customer(&pool, 12345, 1).await;

        let updated_at = Utc::now();
        // Update all fields with Some values
        let update_data = UpdateCustomer {
            is_blocked: Some(true),
            bot_is_blocked_by_user: Some(true),
            has_passed_captcha: Some(true),
            last_seen_with_bot: Some(2),
            last_seen_at: Some(updated_at),
        };

        let _updated_customer = repo.update(initial_customer.id, update_data).await.unwrap();

        // Fetch the customer again to verify
        let fetched_customer = repo.get_by_id(initial_customer.id).await.unwrap();

        assert_eq!(fetched_customer.id, initial_customer.id);
        assert_eq!(fetched_customer.is_blocked, true);
        assert_eq!(fetched_customer.bot_is_blocked_by_user, true);
        assert_eq!(fetched_customer.has_passed_captcha, true);
        assert_eq!(fetched_customer.last_seen_with_bot, 2);
        assert_eq!(
            fetched_customer.last_seen_at.timestamp(),
            updated_at.timestamp()
        );
    }

    #[sqlx::test]
    async fn test_update_customer_some_none_mix(pool: PgPool) {
        let repo = CustomerRepository::new(Arc::new(pool.clone()));

        // Create a customer with initial values
        let initial_customer = create_test_customer(&pool, 54321, 3).await;
        let last_seen_at_initial = initial_customer.last_seen_at;

        // Update some fields with Some, others with None
        let updated_last_seen_with_bot = Some(4);
        let update_data = UpdateCustomer {
            is_blocked: None, // Should remain false
            bot_is_blocked_by_user: Some(true),
            has_passed_captcha: None, // Should remain false
            last_seen_with_bot: updated_last_seen_with_bot,
            last_seen_at: None, // Should remain unchanged
        };

        let _updated_customer = repo.update(initial_customer.id, update_data).await.unwrap();

        // Fetch the customer again to verify
        let fetched_customer = repo.get_by_id(initial_customer.id).await.unwrap();

        assert_eq!(fetched_customer.id, initial_customer.id);
        assert_eq!(fetched_customer.telegram_id, initial_customer.telegram_id);
        assert_eq!(fetched_customer.is_blocked, initial_customer.is_blocked); // Should be false
        assert_eq!(fetched_customer.bot_is_blocked_by_user, true);
        assert_eq!(
            fetched_customer.has_passed_captcha,
            initial_customer.has_passed_captcha
        ); // Should be false
        assert_eq!(
            fetched_customer.last_seen_with_bot,
            updated_last_seen_with_bot.unwrap()
        );
        // last_seen_at is special because its always updated by COALESCE in the query.
        // It should be close to the time of update, not the initial value.
        // However, if the macro logic is correct, and we pass None, it should not be part of push_updates!
        // so it will be only affected by the COALESCE in the base query.
        assert!(fetched_customer.last_seen_at.timestamp() >= last_seen_at_initial.timestamp());
    }
}
