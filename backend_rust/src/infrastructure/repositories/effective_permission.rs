use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;

use crate::{errors::repository::RepositoryResult, models::permission::Permission};

#[async_trait]
pub trait EffectivePermissionRepositoryTrait {
    async fn get_for_user(&self, admin_user_id: i64) -> RepositoryResult<Vec<String>>;
    async fn has_permission(
        &self,
        admin_user_id: i64,
        permission: Permission,
    ) -> RepositoryResult<bool>;
}

#[derive(Clone)]
pub struct EffectivePermissionRepository {
    pool: Arc<PgPool>,
}

impl EffectivePermissionRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EffectivePermissionRepositoryTrait for EffectivePermissionRepository {
    async fn get_for_user(&self, admin_user_id: i64) -> RepositoryResult<Vec<String>> {
        let rows: Vec<Option<String>> = sqlx::query_scalar!(
            r#"
        WITH role_perms AS (
            SELECT DISTINCT p.name
            FROM user_roles ur
            JOIN role_permissions rp ON ur.role_id = rp.role_id
            JOIN permissions p ON rp.permission_id = p.id
            WHERE ur.user_id = $1
        ),
        user_perms AS (
            SELECT p.name, up.effect
            FROM user_permissions up
            JOIN permissions p ON up.permission_id = p.id
            WHERE up.user_id = $1
        )
        SELECT name FROM user_perms WHERE effect = 'allow'
        UNION
        SELECT name FROM role_perms
        EXCEPT
        SELECT name FROM user_perms WHERE effect = 'deny'
        "#,
            admin_user_id
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows.into_iter().flatten().collect())
    }

    async fn has_permission(
        &self,
        admin_user_id: i64,
        permission: Permission,
    ) -> RepositoryResult<bool> {
        let has: Option<bool> = sqlx::query_scalar!(
            r#"
    SELECT 
        NOT EXISTS (
            SELECT 1 FROM user_permissions up
            JOIN permissions p ON up.permission_id = p.id
            WHERE up.user_id = $1 AND p.name = $2 AND up.effect = 'deny'
        )
        AND (
            EXISTS (
                SELECT 1 FROM user_permissions up
                JOIN permissions p ON up.permission_id = p.id
                WHERE up.user_id = $1 AND p.name = $2 AND up.effect = 'allow'
            )
            OR EXISTS (
                SELECT 1 FROM user_roles ur
                JOIN role_permissions rp ON ur.role_id = rp.role_id
                JOIN permissions p ON rp.permission_id = p.id
                WHERE ur.user_id = $1 AND p.name = $2
            )
        )
    "#,
            admin_user_id,
            permission.to_string()
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(has.unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{permission::Permission, user_permission::PermissionEffect};
    use sqlx::PgPool;
    use std::collections::HashMap;
    use tokio::sync::OnceCell;

    static PERMISSIONS: OnceCell<HashMap<Permission, i64>> = OnceCell::const_new();

    async fn setup_permissions(pool: &PgPool) -> &HashMap<Permission, i64> {
        PERMISSIONS
            .get_or_init(|| async {
                let mut permissions = HashMap::new();
                for permission in [
                    Permission::ProductsRead,
                    Permission::ProductsUpdate,
                    Permission::CategoriesRead,
                    Permission::ProductsCreate,
                    Permission::OrdersRead,
                    Permission::StockCreate,
                    Permission::StockRead,
                    Permission::InvoicesRead,
                    Permission::BotsRead,
                ] {
                    let id = create_permission(pool, &permission.to_string()).await;
                    permissions.insert(permission, id);
                }
                permissions
            })
            .await
    }

    async fn create_permission(pool: &PgPool, name: &str) -> i64 {
        sqlx::query_scalar!(
            "INSERT INTO permissions (name, \"group\") VALUES ($1, 'test') ON CONFLICT (name) DO UPDATE SET name = $1 RETURNING id",
            name
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn create_user(pool: &PgPool, login: &str) -> i64 {
        sqlx::query_scalar!(
            "INSERT INTO admin_users (login, hashed_password, two_fa_secret, created_by) VALUES ($1, 'password', '', 1) RETURNING id",
            login
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn create_role(pool: &PgPool, name: &str, created_by: i64) -> i64 {
        sqlx::query_scalar!(
            "INSERT INTO roles (name, created_by) VALUES ($1, $2) RETURNING id",
            name,
            created_by
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn assign_role_to_user(pool: &PgPool, user_id: i64, role_id: i64, created_by: i64) {
        sqlx::query!(
            "INSERT INTO user_roles (user_id, role_id, created_by) VALUES ($1, $2, $3)",
            user_id,
            role_id,
            created_by
        )
        .execute(pool)
        .await
        .unwrap();
    }

    async fn add_permission_to_role(pool: &PgPool, role_id: i64, permission_id: i64) {
        sqlx::query!(
            "INSERT INTO role_permissions (role_id, permission_id, created_by) VALUES ($1, $2, 1)",
            role_id,
            permission_id
        )
        .execute(pool)
        .await
        .unwrap();
    }

    async fn add_direct_permission_to_user(
        pool: &PgPool,
        user_id: i64,
        permission_id: i64,
        effect: PermissionEffect,
        created_by: i64,
    ) {
        sqlx::query!(
            r#"INSERT INTO user_permissions (user_id, permission_id, effect, created_by) VALUES ($1, $2, $3, $4)"#,
            user_id,
            permission_id,
            effect as PermissionEffect,
            created_by
        )
        .execute(pool)
        .await
        .unwrap();
    }

    #[sqlx::test]
    async fn test_get_for_user(pool: PgPool) {
        let permissions_map = setup_permissions(&pool).await;
        let repo = EffectivePermissionRepository::new(Arc::new(pool.clone()));

        // Setup
        let user_id = create_user(&pool, "perm_user_3").await;
        let role1_id = create_role(&pool, "role-1-3", user_id).await;
        let role2_id = create_role(&pool, "role-2-3", user_id).await;

        assign_role_to_user(&pool, user_id, role1_id, user_id).await;
        assign_role_to_user(&pool, user_id, role2_id, user_id).await;

        add_permission_to_role(&pool, role1_id, permissions_map[&Permission::ProductsRead]).await;
        add_permission_to_role(
            &pool,
            role1_id,
            permissions_map[&Permission::ProductsUpdate],
        )
        .await;
        add_permission_to_role(
            &pool,
            role2_id,
            permissions_map[&Permission::CategoriesRead],
        )
        .await;

        add_direct_permission_to_user(
            &pool,
            user_id,
            permissions_map[&Permission::ProductsCreate],
            PermissionEffect::Allow,
            user_id,
        )
        .await;
        add_direct_permission_to_user(
            &pool,
            user_id,
            permissions_map[&Permission::ProductsUpdate],
            PermissionEffect::Deny,
            user_id,
        )
        .await;

        // Test
        let permissions = repo.get_for_user(user_id).await.unwrap();

        // Assert
        assert_eq!(permissions.len(), 3);
        assert!(permissions.contains(&"products:read".to_string()));
        assert!(permissions.contains(&"categories:read".to_string()));
        assert!(permissions.contains(&"products:create".to_string()));
        assert!(!permissions.contains(&"products:update".to_string()));
    }

    #[sqlx::test]
    async fn test_has_permission(pool: PgPool) {
        let permissions_map = setup_permissions(&pool).await;
        let repo = EffectivePermissionRepository::new(Arc::new(pool.clone()));

        // Setup
        let user_id = create_user(&pool, "has_perm_user_3").await;
        let role_id = create_role(&pool, "perm-role-3", user_id).await;

        assign_role_to_user(&pool, user_id, role_id, user_id).await;
        add_permission_to_role(&pool, role_id, permissions_map[&Permission::OrdersRead]).await;
        add_permission_to_role(&pool, role_id, permissions_map[&Permission::InvoicesRead]).await;

        add_direct_permission_to_user(
            &pool,
            user_id,
            permissions_map[&Permission::StockCreate],
            PermissionEffect::Allow,
            user_id,
        )
        .await;
        add_direct_permission_to_user(
            &pool,
            user_id,
            permissions_map[&Permission::StockRead],
            PermissionEffect::Deny,
            user_id,
        )
        .await;
        add_direct_permission_to_user(
            &pool,
            user_id,
            permissions_map[&Permission::InvoicesRead],
            PermissionEffect::Deny,
            user_id,
        )
        .await;

        // Test and Assert
        assert!(
            repo.has_permission(user_id, Permission::OrdersRead)
                .await
                .unwrap()
        );
        assert!(
            repo.has_permission(user_id, Permission::StockCreate)
                .await
                .unwrap()
        );
        assert!(
            !repo
                .has_permission(user_id, Permission::StockRead)
                .await
                .unwrap()
        );
        assert!(
            !repo
                .has_permission(user_id, Permission::InvoicesRead)
                .await
                .unwrap()
        );
        assert!(
            !repo
                .has_permission(user_id, Permission::BotsRead)
                .await
                .unwrap()
        );
    }
}
