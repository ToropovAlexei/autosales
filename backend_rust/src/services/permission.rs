use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    errors::api::ApiResult,
    infrastructure::repositories::permission::{PermissionRepository, PermissionRepositoryTrait},
    models::permission::PermissionRow,
};

#[async_trait]
pub trait PermissionServiceTrait: Send + Sync {
    async fn get_list(&self) -> ApiResult<Vec<PermissionRow>>;
}

pub struct PermissionService<R> {
    repo: Arc<R>,
}

impl<R> PermissionService<R>
where
    R: PermissionRepositoryTrait + Send + Sync,
{
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl PermissionServiceTrait for PermissionService<PermissionRepository> {
    async fn get_list(&self) -> ApiResult<Vec<PermissionRow>> {
        let res = self.repo.get_list().await?;
        Ok(res)
    }
}
