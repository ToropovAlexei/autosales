use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(sqlx::Type, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TemporaryTokenPurpose {
    TwoFa,
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
