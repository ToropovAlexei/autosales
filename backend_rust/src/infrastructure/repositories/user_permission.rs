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
        user_permission: UpdateUserPermission,
    ) -> RepositoryResult<UserPermissionRow> {
        let result = sqlx::query_as!(
            UserPermissionRow,
            r#"UPDATE user_permissions SET effect = COALESCE($1, effect) WHERE user_id = $2 AND permission_id = $3
             RETURNING user_id, permission_id, effect as "effect: _", created_at, updated_at, created_by"#,
            user_permission.effect as Option<PermissionEffect>,
            admin_user_id,
            user_permission.id
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::admin_user::{AdminUserRow, NewAdminUser};
    use crate::models::permission::PermissionRow;
    use sqlx::PgPool;

    async fn create_test_user(pool: &PgPool, login: &str) -> AdminUserRow {
        let new_user = NewAdminUser {
            login: login.to_string(),
            hashed_password: "password".to_string(),
            two_fa_secret: "".to_string(),
            telegram_id: None,
            created_by: 1,
        };
        sqlx::query_as!(
            AdminUserRow,
            r#"
            INSERT INTO admin_users (login, hashed_password, two_fa_secret, created_by, is_system)
            VALUES ($1, $2, $3, $4, false)
            RETURNING *
            "#,
            new_user.login,
            new_user.hashed_password,
            new_user.two_fa_secret,
            new_user.created_by
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn create_test_permission(pool: &PgPool, name: &str, group: &str) -> PermissionRow {
        sqlx::query_as!(
            PermissionRow,
            r#"
            INSERT INTO permissions ("group", name, description)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
            group,
            name,
            "description"
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    #[sqlx::test]
    async fn test_create_and_get_user_permissions(pool: PgPool) {
        let repo = UserPermissionRepository::new(Arc::new(pool.clone()));
        let user = create_test_user(&pool, "test_user_for_up").await;
        let permission = create_test_permission(&pool, "perm:up_create", "group_up").await;

        let new_up = NewUserPermission {
            user_id: user.id,
            permission_id: permission.id,
            effect: PermissionEffect::Allow,
            created_by: user.id,
        };

        repo.create_user_permission(new_up).await.unwrap();

        let ups = repo.get_user_permissions(user.id).await.unwrap();
        assert_eq!(ups.len(), 1);
        assert_eq!(ups[0].permission_id, permission.id);
        assert_eq!(ups[0].effect, PermissionEffect::Allow);
    }

    #[sqlx::test]
    async fn test_update_user_permission(pool: PgPool) {
        let repo = UserPermissionRepository::new(Arc::new(pool.clone()));
        let user = create_test_user(&pool, "test_user_for_up_update").await;
        let permission = create_test_permission(&pool, "perm:up_update", "group_up").await;

        let new_up = NewUserPermission {
            user_id: user.id,
            permission_id: permission.id,
            effect: PermissionEffect::Allow,
            created_by: user.id,
        };
        repo.create_user_permission(new_up).await.unwrap();

        let update = UpdateUserPermission {
            id: permission.id,
            effect: Some(PermissionEffect::Deny),
        };
        repo.update_user_permission(user.id, update).await.unwrap();

        let ups = repo.get_user_permissions(user.id).await.unwrap();
        assert_eq!(ups.len(), 1);
        assert_eq!(ups[0].effect, PermissionEffect::Deny);
    }

    #[sqlx::test]
    async fn test_delete_user_permission(pool: PgPool) {
        let repo = UserPermissionRepository::new(Arc::new(pool.clone()));
        let user = create_test_user(&pool, "test_user_for_up_del").await;
        let permission = create_test_permission(&pool, "perm:up_del", "group_up").await;

        let new_up = NewUserPermission {
            user_id: user.id,
            permission_id: permission.id,
            effect: PermissionEffect::Allow,
            created_by: user.id,
        };
        repo.create_user_permission(new_up).await.unwrap();

        repo.delete_user_permission(user.id, permission.id)
            .await
            .unwrap();

        let ups = repo.get_user_permissions(user.id).await.unwrap();
        assert!(ups.is_empty());
    }
}
