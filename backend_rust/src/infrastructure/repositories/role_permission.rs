use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;

use crate::{
    errors::repository::RepositoryResult,
    models::role_permission::{NewRolePermission, RolePermissionRow},
};

#[async_trait]
pub trait RolePermissionRepositoryTrait {
    async fn get_role_permissions(&self, role_id: i64) -> RepositoryResult<Vec<RolePermissionRow>>;
    async fn create_role_permission(
        &self,
        role_permission: NewRolePermission,
    ) -> RepositoryResult<RolePermissionRow>;
    async fn delete_role_permission(
        &self,
        role_id: i64,
        permission_id: i64,
    ) -> RepositoryResult<()>;
}

#[derive(Clone)]
pub struct RolePermissionRepository {
    pool: Arc<PgPool>,
}

impl RolePermissionRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RolePermissionRepositoryTrait for RolePermissionRepository {
    async fn get_role_permissions(&self, role_id: i64) -> RepositoryResult<Vec<RolePermissionRow>> {
        let result = sqlx::query_as!(
            RolePermissionRow,
            "SELECT * FROM role_permissions WHERE role_id = $1",
            role_id
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(result)
    }

    async fn create_role_permission(
        &self,
        role_permission: NewRolePermission,
    ) -> RepositoryResult<RolePermissionRow> {
        let result = sqlx::query_as!(
            RolePermissionRow,
            r#"INSERT INTO role_permissions (role_id, permission_id, created_by)
                VALUES ($1, $2, $3) 
                RETURNING *"#,
            role_permission.role_id,
            role_permission.permission_id,
            role_permission.created_by
        )
        .fetch_one(&*self.pool)
        .await?;
        Ok(result)
    }

    async fn delete_role_permission(
        &self,
        role_id: i64,
        permission_id: i64,
    ) -> RepositoryResult<()> {
        sqlx::query!(
            "DELETE FROM role_permissions WHERE role_id = $1 AND permission_id = $2",
            role_id,
            permission_id
        )
        .execute(&*self.pool)
        .await?;
        Ok(())
    }
}
