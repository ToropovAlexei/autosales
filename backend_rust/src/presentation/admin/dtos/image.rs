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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_response_from_image_row() {
        let now = Utc::now();
        let uuid = Uuid::new_v4();
        let image_row = ImageRow {
            id: uuid,
            context: "test_context".to_string(),
            created_at: now,
            created_by: 1,
            file_size: 150,
            hash: "abc".to_string(),
            mime_type: "image/png".to_string(),
            original_filename: None,
            height: None,
            width: None,
        };

        let image_response: ImageResponse = image_row.into();

        assert_eq!(image_response.id, uuid);
        assert_eq!(image_response.context, "test_context");
        assert_eq!(image_response.created_at, now);
        assert_eq!(image_response.created_by, 1);
    }
}
