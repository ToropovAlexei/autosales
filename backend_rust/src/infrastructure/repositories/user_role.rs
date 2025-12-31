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
