use chrono::{DateTime, Utc};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "temporary_token_purpose")]
pub enum TemporaryTokenPurpose {
    #[sqlx(rename = "2fa")]
    TwoFa,
    #[sqlx(rename = "password_reset")]
    PasswordReset,
}

#[derive(Debug, FromRow)]
pub struct TemporaryTokenRow {
    pub token: Uuid,
    pub user_id: i64,
    pub purpose: TemporaryTokenPurpose,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
}
