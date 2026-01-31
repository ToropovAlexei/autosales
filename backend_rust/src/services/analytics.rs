use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    errors::api::{ApiError, ApiResult},
    infrastructure::repositories::analytics::{AnalyticsRepository, AnalyticsRepositoryTrait},
    models::analytics::BotAnalyticsRow,
};

#[async_trait]
pub trait AnalyticsServiceTrait: Send + Sync {
    async fn get_referral_stats(&self, customer_id: i64) -> ApiResult<Vec<BotAnalyticsRow>>;
}

pub struct AnalyticsService<R> {
    analytics_repo: Arc<R>,
}

impl<R> AnalyticsService<R>
where
    R: AnalyticsRepositoryTrait + Send + Sync,
{
    pub fn new(analytics_repo: Arc<R>) -> Self {
        Self { analytics_repo }
    }
}

#[async_trait]
impl AnalyticsServiceTrait for AnalyticsService<AnalyticsRepository> {
    async fn get_referral_stats(&self, customer_id: i64) -> ApiResult<Vec<BotAnalyticsRow>> {
        self.analytics_repo
            .get_referral_stats(customer_id)
            .await
            .map_err(ApiError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::repositories::analytics::AnalyticsRepository;
    use rust_decimal::Decimal;
    use sqlx::PgPool;
    use std::str::FromStr;
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

    async fn create_bot(pool: &PgPool, owner_id: Option<i64>, token: &str, username: &str) -> i64 {
        sqlx::query_scalar!(
            r#"
            INSERT INTO bots (
                owner_id, token, username, type, is_active, is_primary, referral_percentage
            )
            VALUES ($1, $2, $3, 'referral', true, false, 10.0)
            RETURNING id
            "#,
            owner_id,
            token,
            username
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn create_order(pool: &PgPool, customer_id: i64, bot_id: i64) -> i64 {
        sqlx::query_scalar!(
            r#"
            INSERT INTO orders (customer_id, amount, currency, status, bot_id)
            VALUES ($1, 100.00, 'RUB', 'fulfilled', $2)
            RETURNING id
            "#,
            customer_id,
            bot_id
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn create_referral_tx(
        pool: &PgPool,
        customer_id: i64,
        order_id: i64,
        bot_id: i64,
        amount: &str,
    ) {
        sqlx::query!(
            r#"
            INSERT INTO transactions (
                customer_id, order_id, type, amount, store_balance_delta,
                platform_commission, gateway_commission, bot_id
            )
            VALUES ($1, $2, 'referral_payout', $3, 0, 0, 0, $4)
            "#,
            customer_id,
            order_id,
            Decimal::from_str(amount).unwrap(),
            bot_id
        )
        .execute(pool)
        .await
        .unwrap();
    }

    fn build_service(pool: &PgPool) -> AnalyticsService<AnalyticsRepository> {
        let pool = Arc::new(pool.clone());
        AnalyticsService::new(Arc::new(AnalyticsRepository::new(pool)))
    }

    #[sqlx::test]
    async fn test_get_referral_stats(pool: PgPool) {
        let service = build_service(&pool);

        let owner_id = create_customer(&pool, 8001).await;
        let buyer_id = create_customer(&pool, 8002).await;
        let bot_id = create_bot(&pool, Some(owner_id), "svc_ref_token", "svc_ref_bot").await;

        let order = create_order(&pool, buyer_id, bot_id).await;
        create_referral_tx(&pool, owner_id, order, bot_id, "15.00").await;

        let stats = service.get_referral_stats(owner_id).await.unwrap();
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].bot_id, bot_id);
        assert_eq!(stats[0].purchase_count, 1);
        assert_eq!(stats[0].total_earnings, Decimal::from_str("15.00").unwrap());
    }
}
