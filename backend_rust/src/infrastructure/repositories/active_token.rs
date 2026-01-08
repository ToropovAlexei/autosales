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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::active_token::NewToken;
    use crate::models::admin_user::{AdminUserRow, NewAdminUser}; // Added
    use chrono::Duration;
    use sqlx::PgPool;

    // Add create_test_user helper function
    async fn create_test_user(pool: &PgPool, login: &str) -> AdminUserRow {
        let new_user = NewAdminUser {
            login: login.to_string(),
            hashed_password: "password".to_string(),
            two_fa_secret: "".to_string(),
            telegram_id: None,
            created_by: 1, // For simplicity, created_by is 1, in real scenario it would be another admin
        };

        let user_id: i64 = sqlx::query!(
            r#"
            INSERT INTO admin_users (login, hashed_password, two_fa_secret, created_by, is_system)
            VALUES ($1, $2, $3, $4, false)
            RETURNING id
            "#,
            new_user.login,
            new_user.hashed_password,
            new_user.two_fa_secret,
            new_user.created_by
        )
        .fetch_one(pool)
        .await
        .unwrap()
        .id;

        sqlx::query_as!(
            AdminUserRow,
            "SELECT * FROM admin_users WHERE id = $1",
            user_id
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    #[sqlx::test]
    async fn test_insert_get_revoke_token(pool: PgPool) {
        let repo = ActiveTokenRepository::new(Arc::new(pool.clone()));
        let user = create_test_user(&pool, "user_for_active_token").await; // Create a user
        let new_token = NewToken {
            user_id: user.id, // Use user.id
            token_type: TokenType::Access,
            ttl: Duration::minutes(15),
        };

        // Insert a token
        let inserted_token = repo.insert_token(new_token).await.unwrap();
        assert_eq!(inserted_token.user_id, user.id);
        assert_eq!(inserted_token.token_type, TokenType::Access);

        // Get the token
        let fetched_token = repo
            .get_active_token(inserted_token.jti, TokenType::Access)
            .await
            .unwrap();
        assert_eq!(fetched_token.jti, inserted_token.jti);

        // Revoke the token
        repo.revoke_token(inserted_token.jti).await.unwrap();

        // Try to get the token again
        let result = repo
            .get_active_token(inserted_token.jti, TokenType::Access)
            .await;
        assert!(result.is_err());
    }

    async fn create_token_with_expiry(
        pool: &PgPool,
        user_id: i64,
        token_type: TokenType,
        ttl: Duration,
    ) -> ActiveTokenRow {
        let new_token = NewToken {
            user_id,
            token_type,
            ttl,
        };

        // Convert TokenType to &str based on its value
        let token_type_str = match new_token.token_type {
            TokenType::Refresh => "refresh",
            TokenType::Access => "access",
        };

        let jti: Uuid = sqlx::query!(
            r#"
            INSERT INTO active_tokens (user_id, token_type, expires_at)
            VALUES ($1, $2, $3)
            RETURNING jti
            "#,
            new_token.user_id,
            token_type_str,
            Utc::now() + new_token.ttl,
        )
        .fetch_one(pool)
        .await
        .unwrap()
        .jti;

        sqlx::query_as!(
            ActiveTokenRow,
            r#"
            SELECT jti, user_id, token_type as "token_type: _", expires_at, created_at, revoked_at
            FROM active_tokens WHERE jti = $1"#,
            jti
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    #[sqlx::test]
    async fn test_delete_expired_tokens(pool: PgPool) {
        let repo = ActiveTokenRepository::new(Arc::new(pool.clone()));

        // Create admin users
        let user1 = create_test_user(&pool, "user1_for_tokens").await;
        let user2 = create_test_user(&pool, "user2_for_tokens").await;

        // Create an expired token for user1
        let expired_token =
            create_token_with_expiry(&pool, user1.id, TokenType::Access, Duration::minutes(-10))
                .await;

        // Create an active token for user2
        let active_token =
            create_token_with_expiry(&pool, user2.id, TokenType::Refresh, Duration::minutes(10))
                .await;

        // Delete expired tokens
        let deleted_count = repo.delete_expired().await.unwrap();
        assert_eq!(deleted_count, 1);

        // Verify expired token is deleted
        let result = repo
            .get_active_token(expired_token.jti, TokenType::Access)
            .await;
        assert!(result.is_err());

        // Verify active token is still present
        let result = repo
            .get_active_token(active_token.jti, TokenType::Refresh)
            .await;
        assert!(result.is_ok());
    }
}
