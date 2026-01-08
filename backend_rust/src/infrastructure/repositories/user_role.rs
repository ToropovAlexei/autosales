use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;

use crate::{
    errors::repository::RepositoryResult,
    models::user_role::{AssignUserRoles, NewUserRole, UserRoleRow},
};

#[async_trait]
pub trait UserRoleRepositoryTrait {
    async fn get_user_roles(&self, admin_user_id: i64) -> RepositoryResult<Vec<UserRoleRow>>;
    async fn create_user_role(&self, user_role: NewUserRole) -> RepositoryResult<UserRoleRow>;
    async fn delete_user_role(&self, admin_user_id: i64, role_id: i64) -> RepositoryResult<()>;
    async fn assign_roles_to_admin_user(&self, user_roles: AssignUserRoles)
    -> RepositoryResult<()>;
}

#[derive(Clone)]
pub struct UserRoleRepository {
    pool: Arc<PgPool>,
}

impl UserRoleRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRoleRepositoryTrait for UserRoleRepository {
    async fn get_user_roles(&self, admin_user_id: i64) -> RepositoryResult<Vec<UserRoleRow>> {
        let result = sqlx::query_as!(
            UserRoleRow,
            "SELECT * FROM user_roles WHERE user_id = $1",
            admin_user_id
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(result)
    }

    async fn create_user_role(&self, user_role: NewUserRole) -> RepositoryResult<UserRoleRow> {
        let result = sqlx::query_as!(
            UserRoleRow,
            r#"INSERT INTO user_roles (user_id, role_id, created_by)
                VALUES ($1, $2, $3) 
                RETURNING *"#,
            user_role.user_id,
            user_role.role_id,
            user_role.created_by
        )
        .fetch_one(&*self.pool)
        .await?;
        Ok(result)
    }

    async fn delete_user_role(&self, user_id: i64, role_id: i64) -> RepositoryResult<()> {
        sqlx::query!(
            "DELETE FROM user_roles WHERE user_id = $1 AND role_id = $2",
            user_id,
            role_id
        )
        .execute(&*self.pool)
        .await?;
        Ok(())
    }

    async fn assign_roles_to_admin_user(
        &self,
        user_roles: AssignUserRoles,
    ) -> RepositoryResult<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query!(
            "DELETE FROM user_roles WHERE user_id = $1",
            user_roles.user_id
        )
        .execute(tx.as_mut())
        .await?;

        sqlx::query!(
            r#"
            INSERT INTO user_roles (user_id, role_id, created_by)
            SELECT $1, p.id, $2
            FROM unnest($3::BIGINT[]) AS p(id)
            ON CONFLICT (user_id, role_id) DO NOTHING
            "#,
            user_roles.user_id,
            user_roles.created_by,
            &user_roles.roles[..]
        )
        .execute(tx.as_mut())
        .await?;

        tx.commit().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::admin_user::{AdminUserRow, NewAdminUser};
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

    #[sqlx::test]
    async fn test_create_and_get_user_roles(pool: PgPool) {
        let repo = UserRoleRepository::new(Arc::new(pool.clone()));
        let user = create_test_user(&pool, "test_user_for_ur").await;
        let role = create_test_role(&pool, "test_role_for_ur", user.id).await;

        let new_ur = NewUserRole {
            user_id: user.id,
            role_id: role.id,
            created_by: user.id,
        };

        repo.create_user_role(new_ur).await.unwrap();

        let urs = repo.get_user_roles(user.id).await.unwrap();
        assert_eq!(urs.len(), 1);
        assert_eq!(urs[0].role_id, role.id);
    }

    #[sqlx::test]
    async fn test_delete_user_role(pool: PgPool) {
        let repo = UserRoleRepository::new(Arc::new(pool.clone()));
        let user = create_test_user(&pool, "test_user_for_ur_del").await;
        let role = create_test_role(&pool, "test_role_for_ur_del", user.id).await;

        let new_ur = NewUserRole {
            user_id: user.id,
            role_id: role.id,
            created_by: user.id,
        };
        repo.create_user_role(new_ur).await.unwrap();

        repo.delete_user_role(user.id, role.id).await.unwrap();

        let urs = repo.get_user_roles(user.id).await.unwrap();
        assert!(urs.is_empty());
    }

    #[sqlx::test]
    async fn test_assign_roles_to_admin_user(pool: PgPool) {
        let repo = UserRoleRepository::new(Arc::new(pool.clone()));
        let user = create_test_user(&pool, "test_user_for_assign").await;
        let role1 = create_test_role(&pool, "role1_for_assign", user.id).await;
        let role2 = create_test_role(&pool, "role2_for_assign", user.id).await;
        let role3 = create_test_role(&pool, "role3_for_assign", user.id).await;

        // Assign role1 initially
        repo.create_user_role(NewUserRole {
            user_id: user.id,
            role_id: role1.id,
            created_by: user.id,
        })
        .await
        .unwrap();

        // Assign role2 and role3, which should replace role1
        let assign_payload = AssignUserRoles {
            user_id: user.id,
            roles: vec![role2.id, role3.id],
            created_by: user.id,
        };
        repo.assign_roles_to_admin_user(assign_payload)
            .await
            .unwrap();

        let urs = repo.get_user_roles(user.id).await.unwrap();
        assert_eq!(urs.len(), 2);
        assert!(!urs.iter().any(|ur| ur.role_id == role1.id));
        assert!(urs.iter().any(|ur| ur.role_id == role2.id));
        assert!(urs.iter().any(|ur| ur.role_id == role3.id));
    }
}
