use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{PgPool, Postgres, QueryBuilder};

use crate::{
    errors::repository::{RepositoryError, RepositoryResult},
    models::admin_user::{AdminUserRow, NewAdminUser, UpdateAdminUser},
};

#[async_trait]
pub trait AdminUserRepositoryTrait {
    async fn get_list(&self) -> RepositoryResult<Vec<AdminUserRow>>;
    async fn create(&self, admin_user: NewAdminUser) -> RepositoryResult<AdminUserRow>;
    async fn get_by_id(&self, id: i64) -> RepositoryResult<AdminUserRow>;
    async fn get_by_login(&self, login: &str) -> RepositoryResult<AdminUserRow>;
    async fn update(&self, id: i64, admin_user: UpdateAdminUser) -> RepositoryResult<AdminUserRow>;
    async fn delete(&self, id: i64) -> RepositoryResult<()>;
}

#[derive(Clone)]
pub struct AdminUserRepository {
    pool: Arc<PgPool>,
}

impl AdminUserRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AdminUserRepositoryTrait for AdminUserRepository {
    async fn get_list(&self) -> RepositoryResult<Vec<AdminUserRow>> {
        let result = sqlx::query_as!(
            AdminUserRow,
            "SELECT * FROM admin_users WHERE deleted_at IS NULL AND is_system = false"
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(result)
    }

    async fn create(&self, admin_user: NewAdminUser) -> RepositoryResult<AdminUserRow> {
        let result = sqlx::query_as!(
            AdminUserRow,
            r#"
            INSERT INTO admin_users (login, hashed_password, two_fa_secret, telegram_id, created_by)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
            admin_user.login,
            admin_user.hashed_password,
            admin_user.two_fa_secret,
            admin_user.telegram_id,
            admin_user.created_by
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(result)
    }

    async fn get_by_id(&self, id: i64) -> RepositoryResult<AdminUserRow> {
        let result = sqlx::query_as!(
            AdminUserRow,
            "SELECT * FROM admin_users WHERE id = $1 AND deleted_at IS NULL AND is_system = false",
            id
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(result)
    }

    async fn get_by_login(&self, login: &str) -> RepositoryResult<AdminUserRow> {
        let result = sqlx::query_as!(
            AdminUserRow,
            "SELECT * FROM admin_users WHERE login = $1 AND deleted_at IS NULL AND is_system = false",
            login
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(result)
    }

    async fn update(&self, id: i64, admin_user: UpdateAdminUser) -> RepositoryResult<AdminUserRow> {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new("UPDATE admin_users SET");
        let mut added = false;

        if let Some(login) = admin_user.login {
            query_builder.push(" login = ");
            query_builder.push_bind(login);
            added = true;
        }

        if let Some(hashed_password) = admin_user.hashed_password {
            if added {
                query_builder.push(",");
            }
            query_builder.push(" hashed_password = ");
            query_builder.push_bind(hashed_password);
            added = true;
        }

        if let Some(two_fa_secret) = admin_user.two_fa_secret {
            if added {
                query_builder.push(",");
            }
            query_builder.push(" two_fa_secret = ");
            query_builder.push_bind(two_fa_secret);
            added = true;
        }

        if let Some(telegram_id) = admin_user.telegram_id {
            if added {
                query_builder.push(",");
            }
            query_builder.push(" telegram_id = ");
            query_builder.push_bind(telegram_id);
        }

        query_builder.push(" WHERE id = ");
        query_builder.push_bind(id);
        query_builder.push(" AND deleted_at IS NULL AND is_system = false RETURNING *");

        let query = query_builder.build_query_as::<AdminUserRow>();

        query
            .fetch_one(&*self.pool)
            .await
            .map_err(RepositoryError::from)
    }

    async fn delete(&self, id: i64) -> RepositoryResult<()> {
        sqlx::query!(
            "UPDATE admin_users SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL AND is_system = false",
            id
        )
        .execute(&*self.pool)
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
            INSERT INTO admin_users (login, hashed_password, two_fa_secret, created_by)
            VALUES ($1, $2, $3, $4)
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
    async fn test_create_get_user(pool: PgPool) {
        let repo = AdminUserRepository::new(Arc::new(pool.clone()));
        let login = "testuser";

        // Create a user
        let created_user = create_test_user(&pool, login).await;
        assert_eq!(created_user.login, login);

        // Get the user by id
        let fetched_user_by_id = repo.get_by_id(created_user.id).await.unwrap();
        assert_eq!(fetched_user_by_id.id, created_user.id);

        // Get the user by login
        let fetched_user_by_login = repo.get_by_login(login).await.unwrap();
        assert_eq!(fetched_user_by_login.login, login);
    }

    #[sqlx::test]
    async fn test_update_user(pool: PgPool) {
        let repo = AdminUserRepository::new(Arc::new(pool.clone()));
        let login = "testuser_update";

        // Create a user
        let user = create_test_user(&pool, login).await;

        // Update the user
        let new_login = "updated_login";
        let update_data = UpdateAdminUser {
            login: Some(new_login.to_string()),
            hashed_password: Some("new_password".to_string()),
            two_fa_secret: None,
            telegram_id: Some(12345),
        };
        let updated_user = repo.update(user.id, update_data).await.unwrap();
        assert_eq!(updated_user.login, new_login);
        assert_eq!(updated_user.telegram_id, Some(12345));

        // Verify the update
        let fetched_user = repo.get_by_id(user.id).await.unwrap();
        assert_eq!(fetched_user.login, new_login);
        assert_eq!(fetched_user.hashed_password, "new_password");
    }

    #[sqlx::test]
    async fn test_delete_user(pool: PgPool) {
        let repo = AdminUserRepository::new(Arc::new(pool.clone()));
        let login = "testuser_delete";

        // Create a user
        let user = create_test_user(&pool, login).await;

        // Delete the user
        repo.delete(user.id).await.unwrap();

        // Try to get the user again
        let result = repo.get_by_id(user.id).await;
        assert!(result.is_err());
    }

    #[sqlx::test]
    async fn test_get_list(pool: PgPool) {
        let repo = AdminUserRepository::new(Arc::new(pool.clone()));

        // Create some users
        create_test_user(&pool, "user1").await;
        create_test_user(&pool, "user2").await;

        // Get the list of users
        let users = repo.get_list().await.unwrap();
        assert!(users.len() >= 2);
    }
}
