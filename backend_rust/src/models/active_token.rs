use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(sqlx::Type, Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum TokenType {
    Refresh,
    Access,
}

#[derive(FromRow, Debug)]
pub struct ActiveTokenRow {
    pub jti: Uuid,
    pub user_id: i64,
    pub token_type: TokenType,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct NewToken {
    pub user_id: i64,
    pub token_type: TokenType,
    pub ttl: Duration,
}
