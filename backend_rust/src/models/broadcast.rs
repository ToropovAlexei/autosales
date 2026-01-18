use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use ts_rs::TS;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::define_list_query;

#[derive(sqlx::Type, Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS, ToSchema)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "broadcast.ts")]
pub enum BroadcastStatus {
    Pending,
    Scheduled,
    InProgress,
    Completed,
    Failed,
}

#[derive(FromRow, Debug, Clone, Serialize)]
pub struct BroadcastRow {
    pub id: i64,
    pub status: BroadcastStatus,
    pub content_text: Option<String>,
    pub content_image_id: Option<Uuid>,
    pub filters: Option<serde_json::Value>,
    pub statistics: Option<serde_json::Value>,
    pub created_by: i64,
    pub scheduled_for: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct NewBroadcast {
    pub status: BroadcastStatus,
    pub content_text: Option<String>,
    pub content_image_id: Option<Uuid>,
    pub filters: Option<serde_json::Value>,
    pub created_by: i64,
    pub scheduled_for: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct UpdateBroadcast {
    pub status: Option<BroadcastStatus>,
    pub content_text: Option<Option<String>>,
    pub content_image_id: Option<Option<Uuid>>,
    pub filters: Option<Option<serde_json::Value>>,
    pub scheduled_for: Option<Option<DateTime<Utc>>>,
    pub statistics: Option<Option<serde_json::Value>>,
    pub started_at: Option<Option<DateTime<Utc>>>,
    pub finished_at: Option<Option<DateTime<Utc>>>,
}

define_list_query! {
    query_name: BroadcastListQuery,
    filter_fields: {
        BroadcastFilterFields,
        [
            Id => "id",
            Status => "status",
            CreatedBy => "created_by",
            CreatedAt => "created_at",
            UpdatedAt => "updated_at",
        ]
    },
    order_fields: {
        BroadcastOrderFields,
        [
            Id => "id",
            Status => "status",
            CreatedBy => "created_by",
            CreatedAt => "created_at",
            UpdatedAt => "updated_at",
        ]
    }
}
