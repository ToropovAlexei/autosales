use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{PgPool, Postgres, QueryBuilder};

use crate::{
    errors::repository::RepositoryResult,
    infrastructure::lib::query::{apply_filters, apply_list_query},
    models::{
        common::PaginatedResult,
        user_subscription::{
            NewUserSubscription, UserSubscriptionEnrichedRow,
            UserSubscriptionExpiryNotificationRow, UserSubscriptionListQuery, UserSubscriptionRow,
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
    async fn get_expiring_for_notification(
        &self,
        within_hours: i64,
    ) -> RepositoryResult<Vec<UserSubscriptionExpiryNotificationRow>>;
    async fn mark_expiry_notification_sent(
        &self,
        subscription_ids: &[i64],
    ) -> RepositoryResult<u64>;
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
            RETURNING
                id, customer_id, product_id, order_id, started_at, expires_at, cancelled_at,
                next_charge_at, renewal_order_id, price_at_subscription, period_days, details,
                expiry_notification_sent_at, created_at, updated_at
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
                us.id,
                us.customer_id,
                us.product_id,
                p.name AS product_name,
                us.order_id,
                us.started_at,
                us.expires_at,
                us.cancelled_at,
                us.next_charge_at,
                us.renewal_order_id,
                us.price_at_subscription,
                us.period_days,
                us.details,
                us.expiry_notification_sent_at,
                us.created_at,
                us.updated_at
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

    async fn get_expiring_for_notification(
        &self,
        within_hours: i64,
    ) -> RepositoryResult<Vec<UserSubscriptionExpiryNotificationRow>> {
        let result = sqlx::query_as!(
            UserSubscriptionExpiryNotificationRow,
            r#"
            SELECT
                us.id,
                us.expires_at,
                p.name AS product_name,
                c.telegram_id,
                c.last_seen_with_bot
            FROM user_subscriptions us
            JOIN customers c ON c.id = us.customer_id
            LEFT JOIN products p ON p.id = us.product_id
            WHERE us.cancelled_at IS NULL
              AND us.expiry_notification_sent_at IS NULL
              AND us.expires_at > NOW()
              AND us.expires_at <= NOW() + make_interval(hours => $1::int)
              AND c.last_seen_with_bot IS NOT NULL
            "#,
            within_hours as i32
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(result)
    }

    async fn mark_expiry_notification_sent(
        &self,
        subscription_ids: &[i64],
    ) -> RepositoryResult<u64> {
        if subscription_ids.is_empty() {
            return Ok(0);
        }

        let result = sqlx::query!(
            r#"
            UPDATE user_subscriptions
            SET expiry_notification_sent_at = NOW()
            WHERE id = ANY($1)
              AND expiry_notification_sent_at IS NULL
            "#,
            subscription_ids
        )
        .execute(&*self.pool)
        .await?;

        Ok(result.rows_affected())
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

    #[sqlx::test]
    async fn test_get_expiring_for_notification_filters(pool: PgPool) {
        let repo = UserSubscriptionRepository::new(Arc::new(pool.clone()));
        let bot = create_test_bot(&pool).await;
        let customer = create_test_customer(&pool, 777001, bot.id).await;
        let product = create_test_product(&pool, "expiring_filter_product").await;
        let order = create_test_order(&pool, customer.id).await;
        let now = Utc::now();

        let should_notify = repo
            .create(NewUserSubscription {
                customer_id: customer.id,
                product_id: Some(product.id),
                order_id: order.id,
                started_at: now - chrono::Duration::days(20),
                expires_at: now + chrono::Duration::hours(12),
                next_charge_at: None,
                price_at_subscription: Decimal::from(10),
                period_days: 30,
                details: None,
            })
            .await
            .unwrap();

        let already_notified = repo
            .create(NewUserSubscription {
                customer_id: customer.id,
                product_id: Some(product.id),
                order_id: order.id,
                started_at: now - chrono::Duration::days(20),
                expires_at: now + chrono::Duration::hours(12),
                next_charge_at: None,
                price_at_subscription: Decimal::from(10),
                period_days: 30,
                details: None,
            })
            .await
            .unwrap();
        sqlx::query(
            "UPDATE user_subscriptions SET expiry_notification_sent_at = NOW() WHERE id = $1",
        )
        .bind(already_notified.id)
        .execute(&pool)
        .await
        .unwrap();

        let out_of_window = repo
            .create(NewUserSubscription {
                customer_id: customer.id,
                product_id: Some(product.id),
                order_id: order.id,
                started_at: now - chrono::Duration::days(20),
                expires_at: now + chrono::Duration::hours(72),
                next_charge_at: None,
                price_at_subscription: Decimal::from(10),
                period_days: 30,
                details: None,
            })
            .await
            .unwrap();

        let cancelled = repo
            .create(NewUserSubscription {
                customer_id: customer.id,
                product_id: Some(product.id),
                order_id: order.id,
                started_at: now - chrono::Duration::days(20),
                expires_at: now + chrono::Duration::hours(6),
                next_charge_at: None,
                price_at_subscription: Decimal::from(10),
                period_days: 30,
                details: None,
            })
            .await
            .unwrap();
        sqlx::query("UPDATE user_subscriptions SET cancelled_at = NOW() WHERE id = $1")
            .bind(cancelled.id)
            .execute(&pool)
            .await
            .unwrap();

        let found = repo.get_expiring_for_notification(24).await.unwrap();
        let found_ids = found.into_iter().map(|s| s.id).collect::<Vec<_>>();

        assert_eq!(found_ids, vec![should_notify.id]);
        assert!(!found_ids.contains(&already_notified.id));
        assert!(!found_ids.contains(&out_of_window.id));
        assert!(!found_ids.contains(&cancelled.id));
    }

    #[sqlx::test]
    async fn test_mark_expiry_notification_sent_is_idempotent(pool: PgPool) {
        let repo = UserSubscriptionRepository::new(Arc::new(pool.clone()));
        let bot = create_test_bot(&pool).await;
        let customer = create_test_customer(&pool, 777002, bot.id).await;
        let product = create_test_product(&pool, "expiring_mark_product").await;
        let order = create_test_order(&pool, customer.id).await;
        let now = Utc::now();

        let sub1 = repo
            .create(NewUserSubscription {
                customer_id: customer.id,
                product_id: Some(product.id),
                order_id: order.id,
                started_at: now - chrono::Duration::days(20),
                expires_at: now + chrono::Duration::hours(8),
                next_charge_at: None,
                price_at_subscription: Decimal::from(10),
                period_days: 30,
                details: None,
            })
            .await
            .unwrap();
        let sub2 = repo
            .create(NewUserSubscription {
                customer_id: customer.id,
                product_id: Some(product.id),
                order_id: order.id,
                started_at: now - chrono::Duration::days(20),
                expires_at: now + chrono::Duration::hours(9),
                next_charge_at: None,
                price_at_subscription: Decimal::from(10),
                period_days: 30,
                details: None,
            })
            .await
            .unwrap();

        let updated = repo
            .mark_expiry_notification_sent(&[sub1.id, sub2.id])
            .await
            .unwrap();
        assert_eq!(updated, 2);

        let updated_again = repo
            .mark_expiry_notification_sent(&[sub1.id, sub2.id])
            .await
            .unwrap();
        assert_eq!(updated_again, 0);

        let found = repo.get_expiring_for_notification(24).await.unwrap();
        let found_ids = found.into_iter().map(|s| s.id).collect::<Vec<_>>();
        assert!(!found_ids.contains(&sub1.id));
        assert!(!found_ids.contains(&sub2.id));
    }
}
