use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    errors::api::{ApiError, ApiResult},
    infrastructure::repositories::role::{RoleRepository, RoleRepositoryTrait},
    models::role::{NewRole, RoleRow, UpdateRole},
};

#[async_trait]
pub trait RoleServiceTrait: Send + Sync {
    async fn get_list(&self) -> ApiResult<Vec<RoleRow>>;
    async fn create(&self, role: NewRole) -> ApiResult<RoleRow>;
    async fn update(&self, id: i64, role: UpdateRole) -> ApiResult<RoleRow>;
    async fn delete(&self, id: i64) -> ApiResult<()>;
}

pub struct RoleService<R> {
    repo: Arc<R>,
}

impl<R> RoleService<R>
where
    R: RoleRepositoryTrait + Send + Sync,
{
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl RoleServiceTrait for RoleService<RoleRepository> {
    async fn get_list(&self) -> ApiResult<Vec<RoleRow>> {
        self.repo.get_roles().await.map_err(ApiError::from)
    }

    async fn create(&self, role: NewRole) -> ApiResult<RoleRow> {
        let created = self.repo.create_role(role).await?;

        Ok(created)
    }

    async fn update(&self, id: i64, role: UpdateRole) -> ApiResult<RoleRow> {
        let updated = self.repo.update_role(id, role).await?;

        Ok(updated)
    }

    async fn delete(&self, id: i64) -> ApiResult<()> {
        Ok(self.repo.delete_role(id).await?)
    }
}
