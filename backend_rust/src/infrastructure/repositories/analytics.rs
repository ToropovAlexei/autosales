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

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use sqlx::PgPool;
    use std::str::FromStr;

    async fn create_customer(pool: &PgPool, telegram_id: i64) -> i64 {
        let rec = sqlx::query!(
            r#"
            INSERT INTO customers (
                telegram_id, registered_with_bot, last_seen_with_bot
            )
            VALUES ($1, 1, 1)
            RETURNING id
            "#,
            telegram_id
        )
        .fetch_one(pool)
        .await
        .unwrap();
        rec.id
    }

    async fn create_bot(pool: &PgPool, owner_id: Option<i64>, token: &str, username: &str) -> i64 {
        let rec = sqlx::query!(
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
        .unwrap();
        rec.id
    }

    async fn create_order(pool: &PgPool, customer_id: i64, bot_id: i64) -> i64 {
        let rec = sqlx::query!(
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
        .unwrap();
        rec.id
    }

    async fn create_referral_tx(
        pool: &PgPool,
        customer_id: i64,
        order_id: i64,
        bot_id: i64,
        amount: &str,
    ) {
        let amount = Decimal::from_str(amount).unwrap();
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
            amount,
            bot_id
        )
        .execute(pool)
        .await
        .unwrap();
    }

    #[sqlx::test]
    async fn test_get_referral_stats(pool: PgPool) {
        let repo = AnalyticsRepository::new(Arc::new(pool.clone()));

        let owner_id = create_customer(&pool, 1001).await;
        let buyer_id = create_customer(&pool, 2001).await;
        let other_owner_id = create_customer(&pool, 3001).await;

        let bot_id = create_bot(&pool, Some(owner_id), "ref_token_1", "ref_bot_1").await;
        let other_bot_id =
            create_bot(&pool, Some(other_owner_id), "ref_token_2", "ref_bot_2").await;

        let order1 = create_order(&pool, buyer_id, bot_id).await;
        let order2 = create_order(&pool, buyer_id, bot_id).await;
        let other_order = create_order(&pool, buyer_id, other_bot_id).await;

        create_referral_tx(&pool, owner_id, order1, bot_id, "10.00").await;
        create_referral_tx(&pool, owner_id, order2, bot_id, "20.00").await;
        create_referral_tx(&pool, other_owner_id, other_order, other_bot_id, "50.00").await;

        // Non-referral transaction should be ignored
        sqlx::query!(
            r#"
            INSERT INTO transactions (
                customer_id, order_id, type, amount, store_balance_delta,
                platform_commission, gateway_commission, bot_id
            )
            VALUES ($1, $2, 'purchase', 100.00, 0, 0, 0, $3)
            "#,
            owner_id,
            order1,
            bot_id
        )
        .execute(&pool)
        .await
        .unwrap();

        let stats = repo.get_referral_stats(owner_id).await.unwrap();
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].bot_id, bot_id);
        assert_eq!(stats[0].purchase_count, 2);
        assert_eq!(stats[0].total_earnings, Decimal::from_str("30.00").unwrap());
    }
}
