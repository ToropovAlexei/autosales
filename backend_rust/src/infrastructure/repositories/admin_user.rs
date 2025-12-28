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
            "SELECT * FROM admin_users WHERE deleted_at IS NULL"
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
            "SELECT * FROM admin_users WHERE id = $1 AND deleted_at IS NULL",
            id
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(result)
    }

    async fn get_by_login(&self, login: &str) -> RepositoryResult<AdminUserRow> {
        let result = sqlx::query_as!(
            AdminUserRow,
            "SELECT * FROM admin_users WHERE login = $1 AND deleted_at IS NULL",
            login
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(result)
    }

    async fn update(&self, id: i64, admin_user: UpdateAdminUser) -> RepositoryResult<AdminUserRow> {
        let mut query_builder: QueryBuilder<Postgres> =
            QueryBuilder::new("UPDATE admin_users SET ");

        if let Some(hashed_password) = admin_user.hashed_password {
            query_builder.push(", hashed_password = ");
            query_builder.push_bind(hashed_password);
        }

        if let Some(login) = admin_user.login {
            query_builder.push(", login = ");
            query_builder.push_bind(login);
        }

        if let Some(telegram_id) = admin_user.telegram_id {
            query_builder.push(", telegram_id = ");
            query_builder.push_bind(telegram_id);
        }

        if let Some(two_fa_secret) = admin_user.two_fa_secret {
            query_builder.push(", two_fa_secret = ");
            query_builder.push_bind(two_fa_secret);
        }

        query_builder.push(" WHERE id = ");
        query_builder.push_bind(id);
        query_builder.push(" AND deleted_at IS NULL RETURNING *");

        let query = query_builder.build_query_as::<AdminUserRow>();

        query
            .fetch_one(&*self.pool)
            .await
            .map_err(RepositoryError::from)
    }

    async fn delete(&self, id: i64) -> RepositoryResult<()> {
        sqlx::query!(
            "UPDATE admin_users SET deleted_at = NOW() WHERE id = $1",
            id
        )
        .execute(&*self.pool)
        .await?;
        Ok(())
    }
}
