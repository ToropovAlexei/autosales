use std::sync::Arc;

use async_trait::async_trait;
use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::repository::RepositoryResult,
    models::temporary_token::{TemporaryTokenPurpose, TemporaryTokenRow},
};

#[async_trait]
pub trait TemporaryTokenRepositoryTrait {
    async fn create(
        &self,
        user_id: i64,
        purpose: TemporaryTokenPurpose,
        ttl: Duration,
    ) -> RepositoryResult<TemporaryTokenRow>;
    async fn find_unused_by_token_and_purpose(
        &self,
        token: &Uuid,
        purpose: TemporaryTokenPurpose,
    ) -> RepositoryResult<TemporaryTokenRow>;
    async fn mark_as_used(&self, token: &Uuid) -> RepositoryResult<bool>;
    async fn delete_expired(&self) -> RepositoryResult<u64>;
}

#[derive(Clone)]
pub struct TemporaryTokenRepository {
    pool: Arc<PgPool>,
}

impl TemporaryTokenRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TemporaryTokenRepositoryTrait for TemporaryTokenRepository {
    async fn create(
        &self,
        user_id: i64,
        purpose: TemporaryTokenPurpose,
        ttl: Duration,
    ) -> RepositoryResult<TemporaryTokenRow> {
        let expires_at = Utc::now() + ttl;
        let token = sqlx::query_as!(
            TemporaryTokenRow,
            r#"
            INSERT INTO temporary_tokens (user_id, purpose, expires_at)
            VALUES ($1, $2, $3)
            RETURNING token, user_id, purpose as "purpose: _", expires_at, created_at, used_at
            "#,
            user_id,
            purpose as TemporaryTokenPurpose,
            expires_at
        )
        .fetch_one(&*self.pool)
        .await?;
        Ok(token)
    }

    async fn find_unused_by_token_and_purpose(
        &self,
        token: &Uuid,
        purpose: TemporaryTokenPurpose,
    ) -> RepositoryResult<TemporaryTokenRow> {
        let res = sqlx::query_as!(
            TemporaryTokenRow,
            r#"
            SELECT token, user_id, purpose as "purpose: _", expires_at, created_at, used_at
            FROM temporary_tokens
            WHERE token = $1
              AND purpose = $2
              AND used_at IS NULL
              AND expires_at > NOW() AT TIME ZONE 'UTC'
            "#,
            token,
            purpose as TemporaryTokenPurpose
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(res)
    }

    async fn mark_as_used(&self, token: &Uuid) -> RepositoryResult<bool> {
        let rows = sqlx::query!(
            r#"
            UPDATE temporary_tokens
            SET used_at = NOW() AT TIME ZONE 'UTC'
            WHERE token = $1
              AND used_at IS NULL
              AND expires_at > NOW() AT TIME ZONE 'UTC'
            "#,
            token
        )
        .execute(&*self.pool)
        .await?
        .rows_affected();

        Ok(rows > 0)
    }

    async fn delete_expired(&self) -> RepositoryResult<u64> {
        let rows = sqlx::query!(
            r#"DELETE FROM temporary_tokens WHERE expires_at <= NOW() AT TIME ZONE 'UTC'"#
        )
        .execute(&*self.pool)
        .await?
        .rows_affected();
        Ok(rows)
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

    #[sqlx::test]
    async fn test_create_and_find(pool: PgPool) {
        let repo = TemporaryTokenRepository::new(Arc::new(pool.clone()));
        let user = create_test_user(&pool, "test_user_for_temp_token").await;

        let purpose = TemporaryTokenPurpose::TwoFa;
        let ttl = Duration::minutes(15);

        let created_token = repo.create(user.id, purpose.clone(), ttl).await.unwrap();
        assert_eq!(created_token.user_id, user.id);
        assert_eq!(created_token.purpose, purpose);

        let found_token = repo
            .find_unused_by_token_and_purpose(&created_token.token, purpose)
            .await
            .unwrap();

        assert_eq!(created_token.token, found_token.token);
    }

    #[sqlx::test]
    async fn test_mark_as_used(pool: PgPool) {
        let repo = TemporaryTokenRepository::new(Arc::new(pool.clone()));
        let user = create_test_user(&pool, "test_user_for_mark_used").await;
        let purpose = TemporaryTokenPurpose::PasswordReset;
        let ttl = Duration::minutes(15);
        let token = repo.create(user.id, purpose.clone(), ttl).await.unwrap();

        let marked = repo.mark_as_used(&token.token).await.unwrap();
        assert!(marked);

        let should_not_find = repo
            .find_unused_by_token_and_purpose(&token.token, purpose)
            .await;
        assert!(should_not_find.is_err());

        // Try to mark again
        let marked_again = repo.mark_as_used(&token.token).await.unwrap();
        assert!(!marked_again)
    }

    #[sqlx::test]
    async fn test_delete_expired(pool: PgPool) {
        let repo = TemporaryTokenRepository::new(Arc::new(pool.clone()));
        let user = create_test_user(&pool, "test_user_for_delete_expired").await;

        // Create an expired token
        let token_ttl = Duration::seconds(-1);
        let expired_token = repo
            .create(user.id, TemporaryTokenPurpose::TwoFa, token_ttl)
            .await
            .unwrap();

        let deleted_count = repo.delete_expired().await.unwrap();
        assert_eq!(deleted_count, 1);

        let should_not_find = repo
            .find_unused_by_token_and_purpose(&expired_token.token, TemporaryTokenPurpose::TwoFa)
            .await;
        assert!(should_not_find.is_err());
    }
}
