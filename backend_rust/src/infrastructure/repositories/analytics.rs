use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;

use crate::{errors::repository::RepositoryResult, models::analytics::BotAnalyticsRow};

#[async_trait]
pub trait AnalyticsRepositoryTrait {
    async fn get_referral_stats(&self, customer_id: i64) -> RepositoryResult<Vec<BotAnalyticsRow>>;
}

#[derive(Clone)]
pub struct AnalyticsRepository {
    pool: Arc<PgPool>,
}

impl AnalyticsRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AnalyticsRepositoryTrait for AnalyticsRepository {
    async fn get_referral_stats(&self, customer_id: i64) -> RepositoryResult<Vec<BotAnalyticsRow>> {
        let result = sqlx::query_as!(
            BotAnalyticsRow,
            r#"
            SELECT
                t.bot_id as "bot_id!",
                COALESCE(SUM(CASE WHEN t.type = 'referral_payout' THEN t.amount ELSE 0 END), 0) AS "total_earnings!",
                COALESCE(COUNT(DISTINCT t.order_id) FILTER (WHERE t.type = 'referral_payout'), 0) AS "purchase_count!"
            FROM transactions t
            JOIN bots b ON b.id = t.bot_id
            WHERE b.owner_id = $1 AND t.bot_id IS NOT NULL
            GROUP BY t.bot_id
            ORDER BY t.bot_id
            "#,
            customer_id
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(result)
    }
}
