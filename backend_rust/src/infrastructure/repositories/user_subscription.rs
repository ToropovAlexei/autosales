use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{PgPool, Postgres, QueryBuilder};

use crate::{
    errors::repository::RepositoryResult,
    infrastructure::lib::query::{apply_filters, apply_list_query},
    models::{
        common::PaginatedResult,
        user_subscription::{NewUserSubscription, UserSubscriptionListQuery, UserSubscriptionRow},
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
    async fn get_for_user(&self, id: i64) -> RepositoryResult<Vec<UserSubscriptionRow>>;
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

    async fn get_for_user(&self, id: i64) -> RepositoryResult<Vec<UserSubscriptionRow>> {
        let result = sqlx::query_as!(
            UserSubscriptionRow,
            "SELECT * FROM user_subscriptions WHERE customer_id = $1",
            id
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(result)
    }
}
