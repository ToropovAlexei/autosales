use jsonwebtoken::{DecodingKey, Validation, decode};

use crate::{errors::auth::AuthError, services::auth::Claims};

pub struct JwtService {
    secret: String,
}

impl JwtService {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }

    pub fn decode(&self, token: &str) -> Result<Claims, AuthError> {
        let data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| AuthError::InvalidToken)?;

        Ok(data.claims)
    }
}
