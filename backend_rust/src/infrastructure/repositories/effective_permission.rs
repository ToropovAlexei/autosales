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
