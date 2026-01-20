use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::api::{ApiError, ApiResult};
use deadpool_redis::redis::AsyncTypedCommands;

#[derive(Debug, Deserialize, Serialize)]
pub struct GenericMessage {
    pub message: String,
    pub image_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InvoiceTroublesNotification {
    pub invoice_id: i64,
    pub amount: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum DispatchMessage {
    GenericMessage(GenericMessage),
    InvoiceTroublesNotification(InvoiceTroublesNotification),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DispatchMessageCommand {
    pub bot_id: i64,
    pub telegram_id: i64,
    pub message: DispatchMessage,
}

#[async_trait]
pub trait NotificationServiceTrait: Send + Sync {
    async fn dispatch_message(&self, payload: DispatchMessageCommand) -> ApiResult<()>;
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
    async fn dispatch_message(&self, payload: DispatchMessageCommand) -> ApiResult<()> {
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
