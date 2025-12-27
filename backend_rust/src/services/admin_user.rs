use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    errors::api::ApiResult,
    infrastructure::repositories::admin_user::{AdminUserRepository, AdminUserRepositoryTrait},
    models::admin_user::{AdminUserRow, NewAdminUser, UpdateAdminUser},
};

#[async_trait]
pub trait AdminUserServiceTrait: Send + Sync {
    async fn get_list(&self) -> ApiResult<Vec<AdminUserRow>>;
    async fn create(&self, admin_user: NewAdminUser) -> ApiResult<AdminUserRow>;
    async fn get_by_id(&self, id: i64) -> ApiResult<AdminUserRow>;
    async fn update(&self, id: i64, admin_user: UpdateAdminUser) -> ApiResult<AdminUserRow>;
    async fn delete(&self, id: i64) -> ApiResult<()>;
}

pub struct AdminUserService<R> {
    repo: Arc<R>,
}

impl<R> AdminUserService<R>
where
    R: AdminUserRepositoryTrait + Send + Sync,
{
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl AdminUserServiceTrait for AdminUserService<AdminUserRepository> {
    async fn get_list(&self) -> ApiResult<Vec<AdminUserRow>> {
        let res = self.repo.get_list().await?;
        Ok(res)
    }

    async fn create(&self, admin_user: NewAdminUser) -> ApiResult<AdminUserRow> {
        let created = self.repo.create(admin_user).await?;
        Ok(created)
    }

    async fn get_by_id(&self, id: i64) -> ApiResult<AdminUserRow> {
        let res = self.repo.get_by_id(id).await?;
        Ok(res)
    }

    async fn update(&self, id: i64, admin_user: UpdateAdminUser) -> ApiResult<AdminUserRow> {
        let res = self.repo.update(id, admin_user).await?;
        Ok(res)
    }

    async fn delete(&self, id: i64) -> ApiResult<()> {
        Ok(self.repo.delete(id).await?)
    }
}
