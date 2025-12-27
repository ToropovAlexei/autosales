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
    ) -> RepositoryResult<Uuid>;
    async fn find_unused_by_token_and_purpose(
        &self,
        token: &Uuid,
        purpose: TemporaryTokenPurpose,
    ) -> RepositoryResult<Option<TemporaryTokenRow>>;
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
    ) -> RepositoryResult<Uuid> {
        let expires_at = Utc::now() + ttl;
        let token = sqlx::query_scalar!(
            r#"
            INSERT INTO temporary_tokens (user_id, purpose, expires_at)
            VALUES ($1, $2, $3)
            RETURNING token
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
    ) -> RepositoryResult<Option<TemporaryTokenRow>> {
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
        .fetch_optional(&*self.pool)
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
