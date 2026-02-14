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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::repositories::{
        permission::PermissionRepository, user_permission::UserPermissionRepository,
    };
    use crate::models::user_permission::UpsertUserPermission;
    use shared_dtos::user_permission::PermissionEffect;
    use sqlx::PgPool;
    use std::sync::Arc;

    async fn create_admin_user(pool: &PgPool, login: &str) -> i64 {
        sqlx::query_scalar!(
            r#"
            INSERT INTO admin_users (login, hashed_password, two_fa_secret, created_by)
            VALUES ($1, 'password', '', 1)
            RETURNING id
            "#,
            login
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn create_role(pool: &PgPool, name: &str) -> i64 {
        sqlx::query_scalar!(
            r#"
            INSERT INTO roles (name, description, created_by)
            VALUES ($1, NULL, 1)
            RETURNING id
            "#,
            name
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn create_permission(pool: &PgPool, name: &str, group: &str) -> i64 {
        sqlx::query_scalar!(
            r#"
            INSERT INTO permissions ("group", name, description)
            VALUES ($1, $2, 'desc')
            RETURNING id
            "#,
            group,
            name
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    fn build_service(
        pool: &PgPool,
    ) -> PermissionService<PermissionRepository, UserPermissionRepository> {
        let pool = Arc::new(pool.clone());
        PermissionService::new(
            Arc::new(PermissionRepository::new(pool.clone())),
            Arc::new(UserPermissionRepository::new(pool)),
        )
    }

    #[sqlx::test]
    async fn test_get_list(pool: PgPool) {
        let service = build_service(&pool);
        let list = service.get_list().await.unwrap();
        assert!(!list.is_empty());
    }

    #[sqlx::test]
    async fn test_get_for_role_and_admin_user(pool: PgPool) {
        let service = build_service(&pool);
        let admin_id = create_admin_user(&pool, "perm_admin").await;
        let role_id = create_role(&pool, "perm_role").await;
        let permission_id = create_permission(&pool, "perm:test", "test").await;

        sqlx::query!(
            "INSERT INTO role_permissions (role_id, permission_id, created_by) VALUES ($1, $2, $3)",
            role_id,
            permission_id,
            admin_id
        )
        .execute(&pool)
        .await
        .unwrap();

        let role_permissions = service.get_for_role(role_id).await.unwrap();
        assert!(role_permissions.iter().any(|p| p.id == permission_id));

        service
            .update_admin_user_permissions(UpdateUserPermissions {
                user_id: admin_id,
                removed: vec![],
                upserted: vec![UpsertUserPermission {
                    id: permission_id,
                    effect: PermissionEffect::Allow,
                }],
                created_by: admin_id,
            })
            .await
            .unwrap();

        let user_permissions = service.get_for_admin_user(admin_id).await.unwrap();
        assert!(
            user_permissions
                .iter()
                .any(|p| p.permission_id == permission_id)
        );
    }
}
