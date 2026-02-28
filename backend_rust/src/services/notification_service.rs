use async_trait::async_trait;
use shared_dtos::notification::{DispatchAdminMessage, DispatchMessagePayload};

use crate::errors::api::{ApiError, ApiResult};

#[async_trait]
pub trait NotificationServiceTrait: Send + Sync {
    async fn dispatch_message(&self, payload: DispatchMessagePayload) -> ApiResult<()>;
    async fn dispatch_admin_message(&self, payload: DispatchAdminMessage) -> ApiResult<()>;
}

pub struct NotificationService {
    client: reqwest::Client,
    user_dispatch_url: String,
    admin_dispatch_url: String,
    service_api_key: String,
}

impl NotificationService {
    pub fn new(
        client: reqwest::Client,
        user_dispatch_url: String,
        admin_dispatch_url: String,
        service_api_key: String,
    ) -> Self {
        Self {
            client,
            user_dispatch_url,
            admin_dispatch_url,
            service_api_key,
        }
    }
}

#[async_trait]
impl NotificationServiceTrait for NotificationService {
    async fn dispatch_message(&self, payload: DispatchMessagePayload) -> ApiResult<()> {
        self.client
            .post(&self.user_dispatch_url)
            .header("X-API-KEY", &self.service_api_key)
            .json(&payload)
            .send()
            .await
            .map_err(|err| ApiError::InternalServerError(err.to_string()))?
            .error_for_status()
            .map_err(|err| ApiError::InternalServerError(err.to_string()))?;
        Ok(())
    }

    async fn dispatch_admin_message(&self, payload: DispatchAdminMessage) -> ApiResult<()> {
        self.client
            .post(&self.admin_dispatch_url)
            .header("X-API-KEY", &self.service_api_key)
            .json(&payload)
            .send()
            .await
            .map_err(|err| ApiError::InternalServerError(err.to_string()))?
            .error_for_status()
            .map_err(|err| ApiError::InternalServerError(err.to_string()))?;
        Ok(())
    }
}
