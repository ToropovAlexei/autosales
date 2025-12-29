use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;

use crate::{
    errors::repository::RepositoryResult,
    models::user_permission::{
        NewUserPermission, PermissionEffect, UpdateUserPermission, UserPermissionRow,
    },
};

#[async_trait]
pub trait UserPermissionRepositoryTrait {
    async fn get_user_permissions(
        &self,
        admin_user_id: i64,
    ) -> RepositoryResult<Vec<UserPermissionRow>>;
    async fn create_user_permission(
        &self,
        user_permission: NewUserPermission,
    ) -> RepositoryResult<UserPermissionRow>;
    async fn update_user_permission(
        &self,
        admin_user_id: i64,
        permission_id: i64,
        user_permission: UpdateUserPermission,
    ) -> RepositoryResult<UserPermissionRow>;
    async fn delete_user_permission(
        &self,
        admin_user_id: i64,
        permission_id: i64,
    ) -> RepositoryResult<()>;
}

#[derive(Clone)]
pub struct UserPermissionRepository {
    pool: Arc<PgPool>,
}

impl UserPermissionRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserPermissionRepositoryTrait for UserPermissionRepository {
    async fn get_user_permissions(
        &self,
        admin_user_id: i64,
    ) -> RepositoryResult<Vec<UserPermissionRow>> {
        let result = sqlx::query_as!(
            UserPermissionRow,
            r#"SELECT 
                user_id,
                permission_id,
                effect as "effect: _",
                created_at,
                updated_at,
                created_by
               FROM user_permissions
                WHERE user_id = $1
                "#,
            admin_user_id
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(result)
    }

    async fn create_user_permission(
        &self,
        user_permission: NewUserPermission,
    ) -> RepositoryResult<UserPermissionRow> {
        let result = sqlx::query_as!(
            UserPermissionRow,
            r#"INSERT INTO user_permissions (user_id, permission_id, effect, created_by)
                VALUES ($1, $2, $3, $4) 
                RETURNING user_id, permission_id, effect as "effect: _", created_at, updated_at, created_by"#,
            user_permission.user_id,
            user_permission.permission_id,
            user_permission.effect as PermissionEffect,
            user_permission.created_by
        )
        .fetch_one(&*self.pool)
        .await?;
        Ok(result)
    }

    async fn update_user_permission(
        &self,
        admin_user_id: i64,
        permission_id: i64,
        user_permission: UpdateUserPermission,
    ) -> RepositoryResult<UserPermissionRow> {
        let result = sqlx::query_as!(
            UserPermissionRow,
            r#"UPDATE user_permissions SET effect = COALESCE($1, effect) WHERE user_id = $2 AND permission_id = $3
             RETURNING user_id, permission_id, effect as "effect: _", created_at, updated_at, created_by"#,
            user_permission.effect as Option<PermissionEffect>,
            admin_user_id,
            permission_id
        )
        .fetch_one(&*self.pool)
        .await?;
        Ok(result)
    }

    async fn delete_user_permission(
        &self,
        user_id: i64,
        permission_id: i64,
    ) -> RepositoryResult<()> {
        sqlx::query!(
            "DELETE FROM user_permissions WHERE user_id = $1 AND permission_id = $2",
            user_id,
            permission_id
        )
        .execute(&*self.pool)
        .await?;
        Ok(())
    }
}
