use std::sync::Arc;

use async_trait::async_trait;
use shared_dtos::user_permission::PermissionEffect;
use sqlx::PgPool;

use crate::{
    errors::repository::RepositoryResult,
    models::{permission::PermissionRow, user_permission::UpdateUserPermissions},
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
                    FROM unnest($3::BIGINT[], $4::TEXT[]) AS p(id, effect)
                    ON CONFLICT (user_id, permission_id) 
                    DO UPDATE SET 
                        effect = EXCLUDED.effect
                "#,
                admin_user_permissions.user_id,
                admin_user_permissions.created_by,
                &added_ids[..],
                &added_effects[..] as &[_],
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::admin_user::{AdminUserRow, NewAdminUser};
    use crate::models::role::{NewRole, RoleRow};
    use crate::models::user_permission::UpsertUserPermission;
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
    async fn test_get_list(pool: PgPool) {
        let repo = PermissionRepository::new(Arc::new(pool.clone()));
        create_test_permission(&pool, "perm:1", "group1").await;
        let permissions = repo.get_list().await.unwrap();
        assert!(!permissions.is_empty());
    }

    #[sqlx::test]
    async fn test_get_for_role(pool: PgPool) {
        let repo = PermissionRepository::new(Arc::new(pool.clone()));
        let user = create_test_user(&pool, "test_user_for_perm_role").await;
        let role = create_test_role(&pool, "test_role_for_perm", user.id).await;
        let permission = create_test_permission(&pool, "perm:2", "group1").await;

        sqlx::query!(
            "INSERT INTO role_permissions (role_id, permission_id, created_by) VALUES ($1, $2, $3)",
            role.id,
            permission.id,
            user.id,
        )
        .execute(&pool)
        .await
        .unwrap();

        let permissions = repo.get_for_role(role.id).await.unwrap();
        assert_eq!(permissions.len(), 1);
        assert_eq!(permissions[0].id, permission.id);
    }

    #[sqlx::test]
    async fn test_get_for_admin_user(pool: PgPool) {
        let repo = PermissionRepository::new(Arc::new(pool.clone()));
        let user = create_test_user(&pool, "test_user_for_user_perm").await;
        let permission = create_test_permission(&pool, "perm:3", "group1").await;

        sqlx::query!(
            "INSERT INTO user_permissions (user_id, permission_id, effect, created_by) VALUES ($1, $2, 'allow', $3)",
            user.id,
            permission.id,
            user.id
        )
        .execute(&pool)
        .await
        .unwrap();

        let permissions = repo.get_for_admin_user(user.id).await.unwrap();
        assert_eq!(permissions.len(), 1);
        assert_eq!(permissions[0].id, permission.id);
    }

    #[sqlx::test]
    async fn test_update_admin_user_permissions(pool: PgPool) {
        let repo = PermissionRepository::new(Arc::new(pool.clone()));
        let user = create_test_user(&pool, "test_user_for_update_perm").await;
        let permission1 = create_test_permission(&pool, "perm:4", "group2").await;
        let permission2 = create_test_permission(&pool, "perm:5", "group2").await;

        let update_payload = UpdateUserPermissions {
            user_id: user.id,
            removed: vec![],
            upserted: vec![UpsertUserPermission {
                id: permission1.id,
                effect: PermissionEffect::Allow,
            }],
            created_by: user.id,
        };
        repo.update_admin_user_permissions(update_payload)
            .await
            .unwrap();

        let permissions = repo.get_for_admin_user(user.id).await.unwrap();
        assert_eq!(permissions.len(), 1);

        // Now remove one and add another
        let update_payload_2 = UpdateUserPermissions {
            user_id: user.id,
            removed: vec![permission1.id],
            upserted: vec![UpsertUserPermission {
                id: permission2.id,
                effect: PermissionEffect::Deny,
            }],
            created_by: user.id,
        };
        repo.update_admin_user_permissions(update_payload_2)
            .await
            .unwrap();

        let permissions_after_update = repo.get_for_admin_user(user.id).await.unwrap();
        assert_eq!(permissions_after_update.len(), 1);
        assert_eq!(permissions_after_update[0].id, permission2.id);
    }
}
