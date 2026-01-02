use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::{ToResponse, ToSchema};
use uuid::Uuid;

use crate::models::image::ImageRow;

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema, ToResponse)]
#[ts(export, export_to = "image.ts")]
pub struct ImageResponse {
    pub id: Uuid,
    pub context: String,
    pub created_at: DateTime<Utc>,
    pub created_by: i64,
}

impl From<ImageRow> for ImageResponse {
    fn from(r: ImageRow) -> Self {
        Self {
            id: r.id,
            context: r.context,
            created_at: r.created_at,
            created_by: r.created_by,
        }
    }
}
