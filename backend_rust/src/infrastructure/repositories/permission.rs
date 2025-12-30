use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;

use crate::{errors::repository::RepositoryResult, models::permission::PermissionRow};

#[async_trait]
pub trait PermissionRepositoryTrait {
    async fn get_list(&self) -> RepositoryResult<Vec<PermissionRow>>;
    async fn get_for_role(&self, role_id: i64) -> RepositoryResult<Vec<PermissionRow>>;
    async fn get_for_admin_user(&self, admin_user_id: i64) -> RepositoryResult<Vec<PermissionRow>>;
}

#[derive(Clone)]
pub struct PermissionRepository {
    pool: Arc<PgPool>,
}

impl PermissionRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PermissionRepositoryTrait for PermissionRepository {
    async fn get_list(&self) -> RepositoryResult<Vec<PermissionRow>> {
        let result = sqlx::query_as!(PermissionRow, "SELECT * FROM permissions")
            .fetch_all(&*self.pool)
            .await?;
        Ok(result)
    }

    async fn get_for_role(&self, role_id: i64) -> RepositoryResult<Vec<PermissionRow>> {
        let permissions = sqlx::query_as!(
            PermissionRow,
            r#"
                SELECT p.id, p.name, p.group, p.description, p.created_at
                FROM permissions p
                INNER JOIN role_permissions rp ON p.id = rp.permission_id
                WHERE rp.role_id = $1
            "#,
            role_id
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(permissions)
    }

    async fn get_for_admin_user(&self, admin_user_id: i64) -> RepositoryResult<Vec<PermissionRow>> {
        let permissions = sqlx::query_as!(
            PermissionRow,
            r#"
                SELECT p.id, p.name, p.group, p.description, p.created_at
                FROM permissions p
                INNER JOIN user_permissions up ON p.id = up.permission_id
                WHERE up.user_id = $1
            "#,
            admin_user_id
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(permissions)
    }
}
