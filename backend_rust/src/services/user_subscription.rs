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
        NewUserSubscription, UserSubscriptionEnrichedRow, UserSubscriptionRow,
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
}
