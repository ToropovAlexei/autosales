use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::{
    errors::api::ApiResult,
    infrastructure::repositories::user_subscription::{
        UserSubscriptionRepository, UserSubscriptionRepositoryTrait,
    },
    middlewares::context::RequestContext,
    models::user_subscription::{
        NewUserSubscription, UserSubscriptionEnrichedRow, UserSubscriptionExpiryNotificationRow,
        UserSubscriptionRow,
    },
};

#[derive(Debug)]
pub struct UpdateUserSubscriptionCommand {
    pub id: i64,
    pub is_blocked: Option<bool>,
    pub bot_is_blocked_by_user: Option<bool>,
    pub has_passed_captcha: Option<bool>,
    pub last_seen_with_bot: Option<i64>,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub updated_by: Option<i64>,
    pub ctx: Option<RequestContext>,
}

#[async_trait]
pub trait UserSubscriptionServiceTrait: Send + Sync {
    async fn create(
        &self,
        user_subscription: NewUserSubscription,
    ) -> ApiResult<UserSubscriptionRow>;
    async fn get_for_customer(&self, id: i64) -> ApiResult<Vec<UserSubscriptionEnrichedRow>>;
    async fn get_expiring_for_notification(
        &self,
        within_hours: i64,
    ) -> ApiResult<Vec<UserSubscriptionExpiryNotificationRow>>;
    async fn mark_expiry_notification_sent(&self, subscription_ids: &[i64]) -> ApiResult<u64>;
}

pub struct UserSubscriptionService<R> {
    user_subscription_repo: Arc<R>,
}

impl<R> UserSubscriptionService<R>
where
    R: UserSubscriptionRepositoryTrait + Send + Sync,
{
    pub fn new(user_subscription_repo: Arc<R>) -> Self {
        Self {
            user_subscription_repo,
        }
    }
}

#[async_trait]
impl UserSubscriptionServiceTrait for UserSubscriptionService<UserSubscriptionRepository> {
    async fn create(
        &self,
        user_subscription: NewUserSubscription,
    ) -> ApiResult<UserSubscriptionRow> {
        let created = self
            .user_subscription_repo
            .create(user_subscription)
            .await?;

        Ok(created)
    }

    async fn get_for_customer(&self, id: i64) -> ApiResult<Vec<UserSubscriptionEnrichedRow>> {
        let res = self.user_subscription_repo.get_for_customer(id).await?;
        Ok(res)
    }

    async fn get_expiring_for_notification(
        &self,
        within_hours: i64,
    ) -> ApiResult<Vec<UserSubscriptionExpiryNotificationRow>> {
        self.user_subscription_repo
            .get_expiring_for_notification(within_hours)
            .await
            .map_err(Into::into)
    }

    async fn mark_expiry_notification_sent(&self, subscription_ids: &[i64]) -> ApiResult<u64> {
        self.user_subscription_repo
            .mark_expiry_notification_sent(subscription_ids)
            .await
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::repositories::user_subscription::UserSubscriptionRepository;
    use chrono::Utc;
    use rust_decimal::Decimal;
    use sqlx::PgPool;
    use std::sync::Arc;

    async fn create_customer(pool: &PgPool, telegram_id: i64, bot_id: i64) -> i64 {
        sqlx::query_scalar!(
            r#"
            INSERT INTO customers (telegram_id, registered_with_bot, last_seen_with_bot)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
            telegram_id,
            bot_id,
            bot_id
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn create_bot(pool: &PgPool, token: &str, username: &str) -> i64 {
        sqlx::query_scalar!(
            r#"
            INSERT INTO bots (owner_id, token, username, type, is_active, is_primary, referral_percentage)
            VALUES (NULL, $1, $2, 'main', true, true, 0.0)
            RETURNING id
            "#,
            token,
            username
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn create_product(pool: &PgPool, name: &str) -> i64 {
        sqlx::query_scalar!(
            r#"
            INSERT INTO products (
                name, base_price, created_by, type, subscription_period_days, provider_name
            )
            VALUES ($1, 100.0, 1, 'subscription', 30, 'test_provider')
            RETURNING id
            "#,
            name
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn create_order(pool: &PgPool, customer_id: i64, bot_id: i64) -> i64 {
        sqlx::query_scalar!(
            r#"
            INSERT INTO orders (customer_id, amount, currency, status, bot_id)
            VALUES ($1, 100.0, 'USD', 'created', $2)
            RETURNING id
            "#,
            customer_id,
            bot_id
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    fn build_service(pool: &PgPool) -> UserSubscriptionService<UserSubscriptionRepository> {
        let pool = Arc::new(pool.clone());
        UserSubscriptionService::new(Arc::new(UserSubscriptionRepository::new(pool)))
    }

    #[sqlx::test]
    async fn test_create_and_get_for_customer(pool: PgPool) {
        let service = build_service(&pool);
        let bot_id = create_bot(&pool, "sub_bot_token", "sub_bot").await;
        let customer_id = create_customer(&pool, 9001, bot_id).await;
        let product_id = create_product(&pool, "sub_product").await;
        let order_id = create_order(&pool, customer_id, bot_id).await;

        let now = Utc::now();
        service
            .create(NewUserSubscription {
                customer_id,
                product_id: Some(product_id),
                order_id,
                started_at: now,
                expires_at: now + chrono::Duration::days(30),
                next_charge_at: None,
                price_at_subscription: Decimal::from(10),
                period_days: 30,
                details: None,
            })
            .await
            .unwrap();

        let subs = service.get_for_customer(customer_id).await.unwrap();
        assert_eq!(subs.len(), 1);
        assert_eq!(subs[0].product_id, Some(product_id));
    }
}
