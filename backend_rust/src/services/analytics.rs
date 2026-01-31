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
