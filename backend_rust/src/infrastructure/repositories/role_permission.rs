use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;

use crate::{
    errors::repository::RepositoryResult,
    models::role_permission::{NewRolePermission, RolePermissionRow, UpdateRolePermissions},
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
    async fn update_role_permissions(
        &self,
        update_role_permissions: UpdateRolePermissions,
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

    async fn update_role_permissions(
        &self,
        update_role_permissions: UpdateRolePermissions,
    ) -> RepositoryResult<()> {
        let mut tx = self.pool.begin().await?;

        if !update_role_permissions.added.is_empty() {
            sqlx::query!(
                r#"
            INSERT INTO role_permissions (role_id, permission_id, created_by)
            SELECT $1, p.id, $2
            FROM unnest($3::BIGINT[]) AS p(id)
            ON CONFLICT (role_id, permission_id) DO NOTHING
            "#,
                update_role_permissions.role_id,
                update_role_permissions.created_by,
                &update_role_permissions.added[..]
            )
            .execute(tx.as_mut())
            .await?;
        }

        if !update_role_permissions.removed.is_empty() {
            sqlx::query!(
                r#"
            DELETE FROM role_permissions
            WHERE role_id = $1
              AND permission_id = ANY($2)
            "#,
                update_role_permissions.role_id,
                &update_role_permissions.removed[..]
            )
            .execute(tx.as_mut())
            .await?;
        }

        tx.commit().await?;

        Ok(())
    }
}
