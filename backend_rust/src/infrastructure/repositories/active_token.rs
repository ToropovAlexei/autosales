use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::repository::RepositoryResult,
    models::active_token::{ActiveTokenRow, NewToken, TokenType},
};

#[async_trait]
pub trait ActiveTokenRepositoryTrait {
    async fn insert_token(&self, token: NewToken) -> RepositoryResult<ActiveTokenRow>;
    async fn get_active_token(
        &self,
        jti: Uuid,
        token_type: TokenType,
    ) -> RepositoryResult<ActiveTokenRow>;
    async fn revoke_token(&self, jti: Uuid) -> RepositoryResult<()>;
    async fn delete_expired(&self) -> RepositoryResult<u64>;
}

#[derive(Clone)]
pub struct ActiveTokenRepository {
    pool: Arc<PgPool>,
}

impl ActiveTokenRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ActiveTokenRepositoryTrait for ActiveTokenRepository {
    async fn insert_token(&self, token: NewToken) -> RepositoryResult<ActiveTokenRow> {
        let expires_at = Utc::now() + token.ttl;
        let rec = sqlx::query_as!(
            ActiveTokenRow,
            r#"
        INSERT INTO active_tokens (user_id, token_type, expires_at)
        VALUES ($1, $2, $3)
        RETURNING jti, user_id, token_type as "token_type: _", expires_at, created_at, revoked_at
        "#,
            token.user_id,
            token.token_type as TokenType,
            expires_at,
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(rec)
    }

    async fn get_active_token(
        &self,
        jti: Uuid,
        token_type: TokenType,
    ) -> RepositoryResult<ActiveTokenRow> {
        let token = sqlx::query_as!(
            ActiveTokenRow,
            r#"
            SELECT jti, user_id, token_type as "token_type: _", expires_at, created_at, revoked_at FROM active_tokens
            WHERE jti = $1
              AND token_type = $2
              AND revoked_at IS NULL
              AND expires_at > NOW()
            "#,
            jti,
            token_type as TokenType,
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(token)
    }

    async fn revoke_token(&self, jti: Uuid) -> RepositoryResult<()> {
        sqlx::query!(
            r#"
        UPDATE active_tokens
        SET revoked_at = NOW()
        WHERE jti = $1
          AND revoked_at IS NULL
        "#,
            jti
        )
        .execute(&*self.pool)
        .await?;

        Ok(())
    }

    async fn delete_expired(&self) -> RepositoryResult<u64> {
        let res = sqlx::query!(
            r#"
        DELETE FROM active_tokens
        WHERE expires_at < NOW()
        "#
        )
        .execute(&*self.pool)
        .await?;

        Ok(res.rows_affected())
    }
}
