use std::sync::Arc;

use serde::Deserialize;
use uuid::Uuid;

use crate::{
    errors::auth::{AuthError, AuthResult},
    infrastructure::repositories::active_token::ActiveTokenRepositoryTrait,
    models::active_token::TokenType,
    services::jwt::JwtService,
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

pub struct AuthService<S> {
    jwt: JwtService,
    tokens: Arc<S>,
}

impl<S> AuthService<S>
where
    S: ActiveTokenRepositoryTrait + Send + Sync,
{
    pub fn new(jwt_secret: String, tokens: Arc<S>) -> Self {
        Self {
            jwt: JwtService::new(jwt_secret),
            tokens,
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
}
