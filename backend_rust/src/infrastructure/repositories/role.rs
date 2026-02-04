use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{PgPool, QueryBuilder};

use crate::{
    errors::repository::RepositoryResult,
    models::role::{NewRole, RoleRow, UpdateRole},
};

#[async_trait]
pub trait RoleRepositoryTrait {
    async fn get_roles(&self) -> RepositoryResult<Vec<RoleRow>>;
    async fn create_role(&self, role: NewRole) -> RepositoryResult<RoleRow>;
    async fn update_role(&self, role_id: i64, role: UpdateRole) -> RepositoryResult<RoleRow>;
    async fn delete_role(&self, role_id: i64) -> RepositoryResult<()>;
}

#[derive(Clone)]
pub struct RoleRepository {
    pool: Arc<PgPool>,
}

impl RoleRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RoleRepositoryTrait for RoleRepository {
    async fn get_roles(&self) -> RepositoryResult<Vec<RoleRow>> {
        let result = sqlx::query_as!(RoleRow, r#"SELECT * FROM roles"#,)
            .fetch_all(&*self.pool)
            .await?;
        Ok(result)
    }

    async fn create_role(&self, role: NewRole) -> RepositoryResult<RoleRow> {
        let result = sqlx::query_as!(
            RoleRow,
            r#"INSERT INTO roles (name, description, created_by)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
            role.name,
            role.description,
            role.created_by,
        )
        .fetch_one(&*self.pool)
        .await?;
        Ok(result)
    }

    async fn update_role(&self, role_id: i64, role: UpdateRole) -> RepositoryResult<RoleRow> {
        let mut query_builder = QueryBuilder::new("UPDATE roles SET");
        let mut has_update = false;

        if let Some(name) = role.name {
            query_builder.push(" name = ");
            query_builder.push_bind(name);
            has_update = true;
        }

        if let Some(description) = role.description {
            if has_update {
                query_builder.push(", ");
            }
            query_builder.push(" description = ");
            if let Some(description) = description {
                query_builder.push_bind(description);
            } else {
                query_builder.push("NULL");
            }
        }

        query_builder.push(" WHERE id = ");
        query_builder.push_bind(role_id);
        query_builder.push(" RETURNING *");

        let result = query_builder
            .build_query_as()
            .fetch_one(&*self.pool)
            .await?;

        Ok(result)
    }

    async fn delete_role(&self, role_id: i64) -> RepositoryResult<()> {
        sqlx::query!("DELETE FROM roles WHERE id = $1", role_id)
            .execute(&*self.pool)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::admin_user::{AdminUserRow, NewAdminUser};
    use sqlx::PgPool;

    async fn create_test_user(pool: &PgPool, login: &str) -> AdminUserRow {
        let new_user = NewAdminUser {
            login: login.to_string(),
            hashed_password: "password".to_string(),
            two_fa_secret: "".to_string(),
            telegram_id: None,
            created_by: 1, // System
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

    #[sqlx::test]
    async fn test_create_and_get_roles(pool: PgPool) {
        let repo = RoleRepository::new(Arc::new(pool.clone()));
        let user = create_test_user(&pool, "test_user_for_roles").await;

        let new_role = NewRole {
            name: "Test Role".to_string(),
            description: Some("A role for testing".to_string()),
            created_by: user.id,
        };

        let created_role = repo.create_role(new_role).await.unwrap();
        assert_eq!(created_role.name, "Test Role");
        assert_eq!(created_role.created_by, user.id);

        let roles = repo.get_roles().await.unwrap();
        assert!(!roles.is_empty());
        assert!(roles.iter().any(|r| r.id == created_role.id));
    }

    #[sqlx::test]
    async fn test_update_role(pool: PgPool) {
        let repo = RoleRepository::new(Arc::new(pool.clone()));
        let user = create_test_user(&pool, "test_user_for_role_update").await;

        let new_role = NewRole {
            name: "Role to Update".to_string(),
            description: Some("Initial description".to_string()),
            created_by: user.id,
        };
        let role = repo.create_role(new_role).await.unwrap();

        let update = UpdateRole {
            name: Some("Updated Role Name".to_string()),
            description: Some(Some("Updated description".to_string())),
        };

        let updated_role = repo.update_role(role.id, update).await.unwrap();
        assert_eq!(updated_role.name, "Updated Role Name");
        assert_eq!(
            updated_role.description,
            Some("Updated description".to_string())
        );
    }

    #[sqlx::test]
    async fn test_delete_role(pool: PgPool) {
        let repo = RoleRepository::new(Arc::new(pool.clone()));
        let user = create_test_user(&pool, "test_user_for_role_delete").await;
        let new_role = NewRole {
            name: "Role to Delete".to_string(),
            description: None,
            created_by: user.id,
        };
        let role = repo.create_role(new_role).await.unwrap();

        repo.delete_role(role.id).await.unwrap();

        let roles = repo.get_roles().await.unwrap();
        assert!(!roles.iter().any(|r| r.id == role.id));
    }
}
