use chrono::{DateTime, Utc};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(FromRow, Debug)]
pub struct ImageRow {
    pub id: Uuid,
    pub original_filename: Option<String>,
    pub hash: String,
    pub mime_type: String,
    pub file_size: i64,
    pub width: Option<i16>,
    pub height: Option<i16>,
    pub context: String,
    pub created_at: DateTime<Utc>,
    pub created_by: i64,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct NewImage {
    pub original_filename: Option<String>,
    pub hash: String,
    pub mime_type: String,
    pub file_size: i64,
    pub width: i16,
    pub height: i16,
    pub context: String,
    pub created_by: i64,
}
