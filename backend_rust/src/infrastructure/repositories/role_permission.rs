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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::admin_user::{AdminUserRow, NewAdminUser};
    use crate::models::permission::PermissionRow;
    use crate::models::role::{NewRole, RoleRow};
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

    async fn create_test_role(pool: &PgPool, name: &str, created_by: i64) -> RoleRow {
        let new_role = NewRole {
            name: name.to_string(),
            description: None,
            created_by,
        };
        sqlx::query_as!(
            RoleRow,
            r#"
            INSERT INTO roles (name, description, created_by)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
            new_role.name,
            new_role.description,
            new_role.created_by
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
    async fn test_create_and_get_role_permissions(pool: PgPool) {
        let repo = RolePermissionRepository::new(Arc::new(pool.clone()));
        let user = create_test_user(&pool, "test_user_for_rp").await;
        let role = create_test_role(&pool, "test_role_for_rp", user.id).await;
        let permission = create_test_permission(&pool, "perm:rp_create", "group_rp").await;

        let new_rp = NewRolePermission {
            role_id: role.id,
            permission_id: permission.id,
            created_by: user.id,
        };

        repo.create_role_permission(new_rp).await.unwrap();

        let rps = repo.get_role_permissions(role.id).await.unwrap();
        assert_eq!(rps.len(), 1);
        assert_eq!(rps[0].permission_id, permission.id);
    }

    #[sqlx::test]
    async fn test_delete_role_permission(pool: PgPool) {
        let repo = RolePermissionRepository::new(Arc::new(pool.clone()));
        let user = create_test_user(&pool, "test_user_for_rp_del").await;
        let role = create_test_role(&pool, "test_role_for_rp_del", user.id).await;
        let permission = create_test_permission(&pool, "perm:rp_del", "group_rp").await;

        let new_rp = NewRolePermission {
            role_id: role.id,
            permission_id: permission.id,
            created_by: user.id,
        };
        repo.create_role_permission(new_rp).await.unwrap();

        repo.delete_role_permission(role.id, permission.id)
            .await
            .unwrap();

        let rps = repo.get_role_permissions(role.id).await.unwrap();
        assert!(rps.is_empty());
    }

    #[sqlx::test]
    async fn test_update_role_permissions(pool: PgPool) {
        let repo = RolePermissionRepository::new(Arc::new(pool.clone()));
        let user = create_test_user(&pool, "test_user_for_rp_update").await;
        let role = create_test_role(&pool, "test_role_for_rp_update", user.id).await;
        let p1 = create_test_permission(&pool, "perm:rp_update1", "group_rp").await;
        let p2 = create_test_permission(&pool, "perm:rp_update2", "group_rp").await;
        let p3 = create_test_permission(&pool, "perm:rp_update3", "group_rp").await;

        // Add p1
        repo.create_role_permission(NewRolePermission {
            role_id: role.id,
            permission_id: p1.id,
            created_by: user.id,
        })
        .await
        .unwrap();

        // Update: remove p1, add p2 and p3
        let update = UpdateRolePermissions {
            role_id: role.id,
            added: vec![p2.id, p3.id],
            removed: vec![p1.id],
            created_by: user.id,
        };
        repo.update_role_permissions(update).await.unwrap();

        let rps = repo.get_role_permissions(role.id).await.unwrap();
        assert_eq!(rps.len(), 2);
        assert!(rps.iter().any(|rp| rp.permission_id == p2.id));
        assert!(rps.iter().any(|rp| rp.permission_id == p3.id));
        assert!(!rps.iter().any(|rp| rp.permission_id == p1.id));
    }
}
