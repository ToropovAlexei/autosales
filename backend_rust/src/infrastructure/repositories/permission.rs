use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;

use crate::{
    errors::repository::RepositoryResult,
    models::{
        permission::PermissionRow,
        user_permission::{PermissionEffect, UpdateUserPermissions},
    },
};

#[async_trait]
pub trait PermissionRepositoryTrait {
    async fn get_list(&self) -> RepositoryResult<Vec<PermissionRow>>;
    async fn get_for_role(&self, role_id: i64) -> RepositoryResult<Vec<PermissionRow>>;
    async fn get_for_admin_user(&self, admin_user_id: i64) -> RepositoryResult<Vec<PermissionRow>>;
    async fn update_admin_user_permissions(
        &self,
        admin_user_permissions: UpdateUserPermissions,
    ) -> RepositoryResult<()>;
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

    async fn update_admin_user_permissions(
        &self,
        admin_user_permissions: UpdateUserPermissions,
    ) -> RepositoryResult<()> {
        let mut tx = self.pool.begin().await?;

        if !admin_user_permissions.upserted.is_empty() {
            let (added_ids, added_effects): (Vec<i64>, Vec<PermissionEffect>) =
                admin_user_permissions
                    .upserted
                    .iter()
                    .map(|p| (p.id, p.effect))
                    .unzip();

            sqlx::query!(
                r#"
                    INSERT INTO user_permissions (user_id, permission_id, effect, created_by)
                    SELECT $1, p.id, p.effect, $2
                    FROM unnest($3::BIGINT[], $4::permission_effect[]) AS p(id, effect)
                    ON CONFLICT (user_id, permission_id) 
                    DO UPDATE SET 
                        effect = EXCLUDED.effect
                "#,
                admin_user_permissions.user_id,
                admin_user_permissions.created_by,
                &added_ids[..],
                &added_effects[..] as &[PermissionEffect],
            )
            .execute(&mut *tx)
            .await?;
        }

        if !admin_user_permissions.removed.is_empty() {
            sqlx::query!(
                r#"
            DELETE FROM user_permissions
            WHERE user_id = $1 AND permission_id = ANY($2)
            "#,
                admin_user_permissions.user_id,
                &admin_user_permissions.removed[..]
            )
            .execute(tx.as_mut())
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }
}
