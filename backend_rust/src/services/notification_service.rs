use std::sync::Arc;

use async_trait::async_trait;
use shared_dtos::notification::DispatchMessagePayload;

use crate::errors::api::{ApiError, ApiResult};
use deadpool_redis::redis::AsyncTypedCommands;

#[async_trait]
pub trait NotificationServiceTrait: Send + Sync {
    async fn dispatch_message(&self, payload: DispatchMessagePayload) -> ApiResult<()>;
}

pub struct NotificationService {
    redis_pool: Arc<deadpool_redis::Pool>,
}

impl NotificationService {
    pub fn new(redis_pool: Arc<deadpool_redis::Pool>) -> Self {
        Self { redis_pool }
    }
}

#[async_trait]
impl NotificationServiceTrait for NotificationService {
    async fn dispatch_message(&self, payload: DispatchMessagePayload) -> ApiResult<()> {
        let channel = format!("bot-notifications:{}", payload.bot_id);
        let message_json = serde_json::to_string(&payload)
            .map_err(|err| ApiError::InternalServerError(err.to_string()))?;

        let mut conn = self
            .redis_pool
            .get()
            .await
            .map_err(|err| ApiError::InternalServerError(err.to_string()))?;
        conn.publish(&channel, message_json)
            .await
            .map_err(|err| ApiError::InternalServerError(err.to_string()))?;
        Ok(())
    }
}
