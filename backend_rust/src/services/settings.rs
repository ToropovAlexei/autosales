use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    errors::api::{ApiError, ApiResult},
    infrastructure::repositories::settings::{SettingsRepository, SettingsRepositoryTrait},
    models::settings::{Settings, UpdateSettings},
};

#[async_trait]
pub trait SettingsServiceTrait: Send + Sync {
    async fn load_settings(&self) -> ApiResult<Settings>;
    async fn update(&self, settings: UpdateSettings) -> ApiResult<Settings>;
}

pub struct SettingsService<R> {
    repo: Arc<R>,
}

impl<R> SettingsService<R>
where
    R: SettingsRepositoryTrait + Send + Sync,
{
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl SettingsServiceTrait for SettingsService<SettingsRepository> {
    async fn load_settings(&self) -> ApiResult<Settings> {
        self.repo.load_settings().await.map_err(ApiError::from)
    }

    async fn update(&self, settings: UpdateSettings) -> ApiResult<Settings> {
        let updated = self.repo.update(settings).await?;

        Ok(updated)
    }
}
