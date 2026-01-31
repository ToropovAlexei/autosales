use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{PgPool, Postgres, QueryBuilder};

use crate::{
    errors::repository::RepositoryResult,
    infrastructure::lib::query::{apply_filters, apply_list_query},
    models::{
        common::PaginatedResult,
        user_subscription::{
            NewUserSubscription, UserSubscriptionEnrichedRow, UserSubscriptionListQuery,
            UserSubscriptionRow,
        },
    },
};

#[async_trait]
pub trait UserSubscriptionRepositoryTrait {
    async fn get_list(
        &self,
        query: UserSubscriptionListQuery,
    ) -> RepositoryResult<PaginatedResult<UserSubscriptionRow>>;
    async fn create(
        &self,
        user_subscription: NewUserSubscription,
    ) -> RepositoryResult<UserSubscriptionRow>;
    async fn get_for_customer(&self, id: i64)
    -> RepositoryResult<Vec<UserSubscriptionEnrichedRow>>;
}

#[derive(Clone)]
pub struct UserSubscriptionRepository {
    pool: Arc<PgPool>,
}

impl UserSubscriptionRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserSubscriptionRepositoryTrait for UserSubscriptionRepository {
    async fn get_list(
        &self,
        query: UserSubscriptionListQuery,
    ) -> RepositoryResult<PaginatedResult<UserSubscriptionRow>> {
        let mut count_qb: QueryBuilder<Postgres> =
            QueryBuilder::new("SELECT COUNT(*) FROM user_subscriptions");
        apply_filters(&mut count_qb, &query);

        let count_query = count_qb.build_query_scalar();
        let total: i64 = count_query.fetch_one(&*self.pool).await?;

        let mut query_builder: QueryBuilder<Postgres> =
            QueryBuilder::new("SELECT * FROM user_subscriptions");
        apply_list_query(&mut query_builder, &query);
        let query = query_builder.build_query_as::<UserSubscriptionRow>();
        let items = query.fetch_all(&*self.pool).await?;
        Ok(PaginatedResult { items, total })
    }

    async fn create(
        &self,
        user_subscription: NewUserSubscription,
    ) -> RepositoryResult<UserSubscriptionRow> {
        let result = sqlx::query_as!(
            UserSubscriptionRow,
            r#"
            INSERT INTO user_subscriptions (
                customer_id, product_id, order_id, started_at, expires_at, next_charge_at,
                price_at_subscription, period_days, details
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
            user_subscription.customer_id,
            user_subscription.product_id,
            user_subscription.order_id,
            user_subscription.started_at,
            user_subscription.expires_at,
            user_subscription.next_charge_at,
            user_subscription.price_at_subscription,
            user_subscription.period_days,
            user_subscription.details
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(result)
    }

    async fn get_for_customer(
        &self,
        id: i64,
    ) -> RepositoryResult<Vec<UserSubscriptionEnrichedRow>> {
        let result = sqlx::query_as!(
            UserSubscriptionEnrichedRow,
            r#"
            SELECT 
                us.*, 
                p.name AS product_name
            FROM user_subscriptions us
            JOIN products p ON us.product_id = p.id
            WHERE customer_id = $1
            "#,
            id
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{bot::BotRow, customer::CustomerRow, order::OrderRow, product::ProductRow};
    use chrono::Utc;
    use rust_decimal::Decimal;
    use shared_dtos::list_query::{OrderDir, Pagination};
    use sqlx::PgPool;

    async fn create_test_customer(pool: &PgPool, telegram_id: i64, bot_id: i64) -> CustomerRow {
        sqlx::query_as!(
            CustomerRow,
            r#"
            INSERT INTO customers (telegram_id, registered_with_bot, last_seen_with_bot)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
            telegram_id,
            bot_id,
            bot_id
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn create_test_bot(pool: &PgPool) -> BotRow {
        let bot_id: i64 = sqlx::query!(
            r#"
            INSERT INTO bots (owner_id, token, username, type, is_active, is_primary, referral_percentage)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id
            "#,
            None as Option<i64>, // owner_id
            "test_token_bot".to_string(), // Ensure unique token
            "test_bot_username".to_string(), // Ensure unique username
            "main", // type - explicitly use string literal
            true,            // is_active
            true,            // is_primary
            Decimal::from(0), // referral_percentage
        )
        .fetch_one(pool)
        .await
        .unwrap()
        .id; // Get the ID of the newly inserted bot

        // Now, fetch the full BotRow using query_as!
        sqlx::query_as!(
            BotRow,
            r#"SELECT 
                id, owner_id, token, username, type as "type: _", is_active, is_primary, 
                referral_percentage, created_at, updated_at, created_by
            FROM bots WHERE id = $1"#,
            bot_id
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn create_test_product(pool: &PgPool, name: &str) -> ProductRow {
        let product_id: i64 = sqlx::query!(
            r#"
            INSERT INTO products (
                name, base_price, created_by, type, subscription_period_days,
                provider_name
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            "#,
            name,
            Decimal::from(100), // price
            1,                  // created_by
            "subscription",     // type - explicitly use string literal
            30,                 // subscription_period_days
            "test_provider"     // provider_name
        )
        .fetch_one(pool)
        .await
        .unwrap()
        .id; // Get the ID of the newly inserted product

        // Now, fetch the full ProductRow using query_as!
        sqlx::query_as!(
            ProductRow,
            r#"SELECT 
                id, name, base_price, category_id, image_id, type as "type: _",
                subscription_period_days, details, deleted_at, fulfillment_text, 
                fulfillment_image_id, provider_name, external_id, created_at, 
                updated_at, created_by, stock
            FROM products WHERE id = $1"#,
            product_id
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn create_test_order(pool: &PgPool, customer_id: i64) -> OrderRow {
        let order_id: i64 = sqlx::query!(
            r#"
            INSERT INTO orders (customer_id, amount, currency, status, bot_id)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            "#,
            customer_id,
            Decimal::from(100), // amount
            "USD",              // currency
            "created",          // status - explicitly use string literal
            1                   // bot_id
        )
        .fetch_one(pool)
        .await
        .unwrap()
        .id; // Get the ID of the newly inserted order

        // Now, fetch the full OrderRow using query_as!
        sqlx::query_as!(
            OrderRow,
            r#"SELECT 
                id, customer_id, amount, currency, status as "status: _", bot_id,
                created_at, updated_at, paid_at, fulfilled_at, cancelled_at
            FROM orders WHERE id = $1"#,
            order_id
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    #[sqlx::test]
    async fn test_create_and_get_for_user(pool: PgPool) {
        let repo = UserSubscriptionRepository::new(Arc::new(pool.clone()));
        let bot = create_test_bot(&pool).await; // Create a test bot
        let customer = create_test_customer(&pool, 12345, bot.id).await; // Pass bot.id
        let product = create_test_product(&pool, "sub_product").await;
        let order = create_test_order(&pool, customer.id).await;

        let now = Utc::now();
        let new_sub = NewUserSubscription {
            customer_id: customer.id,
            product_id: Some(product.id),
            order_id: order.id,
            started_at: now,
            expires_at: now + chrono::Duration::days(30),
            next_charge_at: None,
            price_at_subscription: Decimal::from(10),
            period_days: 30,
            details: None,
        };

        repo.create(new_sub).await.unwrap();

        let subs = repo.get_for_customer(customer.id).await.unwrap();
        assert_eq!(subs.len(), 1);
        assert_eq!(subs[0].product_id, Some(product.id));
        assert_eq!(subs[0].product_name, Some(product.name));
    }

    #[sqlx::test]
    async fn test_get_list(pool: PgPool) {
        let repo = UserSubscriptionRepository::new(Arc::new(pool.clone()));
        let bot = create_test_bot(&pool).await; // Create a test bot
        let customer = create_test_customer(&pool, 54321, bot.id).await; // Pass bot.id
        let product = create_test_product(&pool, "sub_list_product").await;
        let order = create_test_order(&pool, customer.id).await;
        let now = Utc::now();
        let new_sub = NewUserSubscription {
            customer_id: customer.id,
            product_id: Some(product.id),
            order_id: order.id,
            started_at: now,
            expires_at: now + chrono::Duration::days(30),
            next_charge_at: None,
            price_at_subscription: Decimal::from(10),
            period_days: 30,
            details: None,
        };
        repo.create(new_sub).await.unwrap();

        let list_query = UserSubscriptionListQuery {
            filters: vec![],
            pagination: Pagination {
                page: 1,
                page_size: 10,
            },
            order_by: None,
            order_dir: OrderDir::Desc,
        };
        let result = repo.get_list(list_query).await.unwrap();
        assert_eq!(result.total, 1);
        assert_eq!(result.items.len(), 1);
    }
}
