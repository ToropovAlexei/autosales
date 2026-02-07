use crate::define_list_query;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(FromRow, Debug, Serialize, Deserialize)]
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

define_list_query! {
    query_name: ImageListQuery,
    filter_fields: {
        ImageFilterFields,
        [
            Context => "context",
        ]
    },
    order_fields: {
        ImageOrderFields,
        [
            CreatedAt => "created_at",
        ]
    }
}
