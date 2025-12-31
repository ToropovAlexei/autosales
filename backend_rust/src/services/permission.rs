use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    errors::api::ApiResult,
    infrastructure::repositories::{
        permission::{PermissionRepository, PermissionRepositoryTrait},
        user_permission::{UserPermissionRepository, UserPermissionRepositoryTrait},
    },
    models::{
        permission::PermissionRow,
        user_permission::{UpdateUserPermissions, UserPermissionRow},
    },
};

#[async_trait]
pub trait PermissionServiceTrait: Send + Sync {
    async fn get_list(&self) -> ApiResult<Vec<PermissionRow>>;
    async fn get_for_role(&self, role_id: i64) -> ApiResult<Vec<PermissionRow>>;
    async fn get_for_admin_user(&self, admin_user_id: i64) -> ApiResult<Vec<UserPermissionRow>>;
    async fn update_admin_user_permissions(
        &self,
        permissions: UpdateUserPermissions,
    ) -> ApiResult<()>;
}

pub struct PermissionService<R, T> {
    repo: Arc<R>,
    user_permission_repo: Arc<T>,
}

impl<R, T> PermissionService<R, T>
where
    R: PermissionRepositoryTrait + Send + Sync,
    T: UserPermissionRepositoryTrait + Send + Sync,
{
    pub fn new(repo: Arc<R>, user_permission_repo: Arc<T>) -> Self {
        Self {
            repo,
            user_permission_repo,
        }
    }
}

#[async_trait]
impl PermissionServiceTrait for PermissionService<PermissionRepository, UserPermissionRepository> {
    async fn get_list(&self) -> ApiResult<Vec<PermissionRow>> {
        let res = self.repo.get_list().await?;
        Ok(res)
    }

    async fn get_for_role(&self, role_id: i64) -> ApiResult<Vec<PermissionRow>> {
        let res = self.repo.get_for_role(role_id).await?;
        Ok(res)
    }

    async fn get_for_admin_user(&self, admin_user_id: i64) -> ApiResult<Vec<UserPermissionRow>> {
        let res = self
            .user_permission_repo
            .get_user_permissions(admin_user_id)
            .await?;
        Ok(res)
    }

    async fn update_admin_user_permissions(
        &self,
        permissions: UpdateUserPermissions,
    ) -> ApiResult<()> {
        Ok(self.repo.update_admin_user_permissions(permissions).await?)
    }
}
