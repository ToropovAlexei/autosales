use std::sync::Arc;

use chrono::Duration;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    errors::auth::{AuthError, AuthResult},
    infrastructure::repositories::{
        active_token::ActiveTokenRepositoryTrait, admin_user::AdminUserRepositoryTrait,
        temporary_token::TemporaryTokenRepositoryTrait,
    },
    models::{
        active_token::{ActiveTokenRow, NewToken, TokenType},
        temporary_token::{TemporaryTokenPurpose, TemporaryTokenRow},
    },
    services::{jwt::JwtService, topt_encryptor::TotpEncryptor},
};

pub struct AuthUser {
    pub id: i64,
}

#[derive(Debug, Deserialize)]
pub struct Claims {
    pub sub: i64,
    pub jti: Uuid,
    pub token_type: TokenType,
    pub exp: usize,
}

pub struct AuthService<S, T, R> {
    jwt: JwtService,
    tokens: Arc<S>,
    temp_tokens: Arc<T>,
    admin_user_repo: Arc<R>,
    totp_encryptor: Arc<TotpEncryptor>,
    config: AuthServiceConfig,
}

pub struct AuthServiceConfig {
    pub jwt_secret: String,
    pub totp_encode_secret: String,
    pub totp_algorithm: totp_rs::Algorithm,
    pub totp_digits: usize,
    pub totp_skew: u8,
    pub totp_step: u64,
    pub access_token_ttl: Duration,
    pub refresh_token_ttl: Duration,
    pub two_fa_token_ttl: Duration,
}

impl<S, T, R> AuthService<S, T, R>
where
    S: ActiveTokenRepositoryTrait + Send + Sync,
    T: TemporaryTokenRepositoryTrait + Send + Sync,
    R: AdminUserRepositoryTrait + Send + Sync,
{
    pub fn new(
        tokens: Arc<S>,
        temp_tokens: Arc<T>,
        admin_user_repo: Arc<R>,
        totp_encryptor: Arc<TotpEncryptor>,
        config: AuthServiceConfig,
    ) -> Self {
        Self {
            jwt: JwtService::new(config.jwt_secret.clone()),
            tokens,
            temp_tokens,
            admin_user_repo,
            totp_encryptor,
            config,
        }
    }
    pub async fn authenticate(&self, token: &str) -> AuthResult<AuthUser> {
        let claims = self.jwt.decode(token)?;

        if claims.token_type != TokenType::Access {
            return Err(AuthError::InvalidToken);
        }

        let is_active = self
            .tokens
            .is_token_active(claims.jti, TokenType::Access)
            .await
            .map_err(|_| AuthError::InvalidToken)?;

        if !is_active {
            return Err(AuthError::TokenRevoked);
        }

        Ok(AuthUser { id: claims.sub })
    }

    pub fn verify_password(&self, plain: &str, hash: &str) -> AuthResult<bool> {
        bcrypt::verify(plain, hash).map_err(|_| AuthError::InvalidToken)
    }

    pub fn verify_totp_code(&self, secret: &str, code: &str) -> AuthResult<bool> {
        let totp = totp_rs::TOTP::new(
            self.config.totp_algorithm,
            self.config.totp_digits,
            self.config.totp_skew,
            self.config.totp_step,
            totp_rs::Secret::Encoded(secret.to_string())
                .to_bytes()
                .map_err(|e| AuthError::InternalServerError(e.to_string()))?,
            None,
            "".to_string(),
        )
        .expect("Failed to create TOTP");

        Ok(totp.check_current(code).unwrap_or_default())
    }

    pub async fn login_step1(
        &self,
        login: String,
        password: String,
    ) -> Result<TemporaryTokenRow, AuthError> {
        let user = self
            .admin_user_repo
            .get_by_login(&login)
            .await
            .map_err(|_e| AuthError::InvalidCredentials)?;

        if !self.verify_password(&password, &user.hashed_password)? {
            return Err(AuthError::InvalidCredentials);
        }

        let temp_token = self
            .temp_tokens
            .create(
                user.id,
                TemporaryTokenPurpose::TwoFa,
                self.config.two_fa_token_ttl,
            )
            .await?;

        Ok(temp_token)
    }

    pub async fn login_step2(
        &self,
        temp_token: &Uuid,
        code: &str,
    ) -> Result<ActiveTokenRow, AuthError> {
        let temp_token = self
            .temp_tokens
            .find_unused_by_token_and_purpose(temp_token, TemporaryTokenPurpose::TwoFa)
            .await
            .map_err(|_e| AuthError::InvalidCredentials)?;

        let user = self.admin_user_repo.get_by_id(temp_token.user_id).await?;

        if !self.verify_totp_code(
            &self
                .totp_encryptor
                .decrypt(&user.two_fa_secret)
                .map_err(|e| AuthError::InternalServerError(e.to_string()))?,
            code,
        )? {
            return Err(AuthError::Invalid2FACode);
        }

        self.temp_tokens.mark_as_used(&temp_token.token).await?;

        self.tokens
            .insert_token(NewToken {
                token_type: TokenType::Access,
                user_id: temp_token.user_id,
                ttl: self.config.access_token_ttl,
            })
            .await
            .map_err(AuthError::from)
    }
}
