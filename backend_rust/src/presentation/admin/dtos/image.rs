use shared_dtos::image::ImageAdminResponse;

use crate::models::image::ImageRow;

impl From<ImageRow> for ImageAdminResponse {
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
    use chrono::Utc;
    use uuid::Uuid;

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

        let image_response: ImageAdminResponse = image_row.into();

        assert_eq!(image_response.id, uuid);
        assert_eq!(image_response.context, "test_context");
        assert_eq!(image_response.created_at, now);
        assert_eq!(image_response.created_by, 1);
    }
}
