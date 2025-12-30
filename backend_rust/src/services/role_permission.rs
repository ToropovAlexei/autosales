use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    errors::api::{ApiError, ApiResult},
    infrastructure::repositories::role_permission::{
        RolePermissionRepository, RolePermissionRepositoryTrait,
    },
    models::role_permission::{NewRolePermission, RolePermissionRow},
};

#[async_trait]
pub trait RolePermissionServiceTrait: Send + Sync {
    async fn get_for_role(&self, role_id: i64) -> ApiResult<Vec<RolePermissionRow>>;
    async fn create(&self, permission: NewRolePermission) -> ApiResult<RolePermissionRow>;
    async fn delete(&self, role_id: i64, permission_id: i64) -> ApiResult<()>;
}

pub struct RolePermissionService<R> {
    repo: Arc<R>,
}

impl<R> RolePermissionService<R>
where
    R: RolePermissionRepositoryTrait + Send + Sync,
{
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl RolePermissionServiceTrait for RolePermissionService<RolePermissionRepository> {
    async fn get_for_role(&self, role_id: i64) -> ApiResult<Vec<RolePermissionRow>> {
        self.repo
            .get_role_permissions(role_id)
            .await
            .map_err(ApiError::from)
    }

    async fn create(&self, permission: NewRolePermission) -> ApiResult<RolePermissionRow> {
        let created = self.repo.create_role_permission(permission).await?;

        Ok(created)
    }

    async fn delete(&self, role_id: i64, permission_id: i64) -> ApiResult<()> {
        Ok(self
            .repo
            .delete_role_permission(role_id, permission_id)
            .await?)
    }
}
