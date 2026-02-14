use shared_dtos::category::CategoryAdminResponse;

use crate::models::category::CategoryRow;

impl From<CategoryRow> for CategoryAdminResponse {
    fn from(r: CategoryRow) -> Self {
        CategoryAdminResponse {
            id: r.id,
            name: r.name,
            parent_id: r.parent_id,
            image_id: r.image_id,
            position: r.position,
            is_active: r.is_active,
            created_by: r.created_by,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use shared_dtos::category::{NewCategoryAdminRequest, UpdateCategoryAdminRequest};
    use uuid::Uuid;
    use validator::Validate;

    #[test]
    fn test_category_response_from_category_row() {
        let now = Utc::now();
        let uuid = Uuid::new_v4();
        let category_row = CategoryRow {
            id: 1,
            name: "Test Category".to_string(),
            parent_id: Some(10),
            image_id: Some(uuid),
            position: 1,
            is_active: true,
            created_by: 100,
            created_at: now,
            updated_at: now,
        };

        let category_response: CategoryAdminResponse = category_row.into();

        assert_eq!(category_response.id, 1);
        assert_eq!(category_response.name, "Test Category");
        assert_eq!(category_response.parent_id, Some(10));
        assert_eq!(category_response.image_id, Some(uuid));
        assert_eq!(category_response.position, 1);
        assert!(category_response.is_active);
        assert_eq!(category_response.created_by, 100);
        assert_eq!(category_response.created_at, now);
        assert_eq!(category_response.updated_at, now);
    }

    #[test]
    fn test_category_response_from_category_row_minimal() {
        let now = Utc::now();
        let category_row = CategoryRow {
            id: 1,
            name: "Test Category".to_string(),
            parent_id: None,
            image_id: None,
            position: 1,
            is_active: true,
            created_by: 100,
            created_at: now,
            updated_at: now,
        };

        let category_response: CategoryAdminResponse = category_row.into();

        assert_eq!(category_response.id, 1);
        assert_eq!(category_response.name, "Test Category");
        assert_eq!(category_response.parent_id, None);
        assert_eq!(category_response.image_id, None);
        assert_eq!(category_response.position, 1);
        assert!(category_response.is_active);
        assert_eq!(category_response.created_by, 100);
        assert_eq!(category_response.created_at, now);
        assert_eq!(category_response.updated_at, now);
    }

    #[test]
    fn test_new_category_request_validation() {
        // Valid data
        let req = NewCategoryAdminRequest {
            name: "Valid Name".to_string(),
            parent_id: Some(1),
            image_id: Some(Uuid::new_v4()),
        };
        assert!(req.validate().is_ok());

        // Name too short
        let req = NewCategoryAdminRequest {
            name: "a".to_string(),
            parent_id: None,
            image_id: None,
        };
        assert!(req.validate().is_err());

        // Name too long
        let req = NewCategoryAdminRequest {
            name: "a".repeat(256),
            parent_id: None,
            image_id: None,
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_update_category_request_validation() {
        // Valid: All optional fields are None
        let req = UpdateCategoryAdminRequest {
            name: None,
            parent_id: None,
            image_id: None,
            position: None,
        };
        assert!(req.validate().is_ok());

        // Valid: All fields provided and correct
        let req = UpdateCategoryAdminRequest {
            name: Some("New Name".to_string()),
            parent_id: Some(Some(2)),
            image_id: Some(Some(Uuid::new_v4())),
            position: Some(5),
        };
        assert!(req.validate().is_ok());

        // Valid: Setting optional fields to None
        let req = UpdateCategoryAdminRequest {
            name: None,
            parent_id: Some(None),
            image_id: Some(None),
            position: Some(5),
        };
        assert!(req.validate().is_ok());

        // Name too short
        let req = UpdateCategoryAdminRequest {
            name: Some("a".to_string()),
            parent_id: None,
            image_id: None,
            position: None,
        };
        assert!(req.validate().is_err());

        // Name too long
        let req = UpdateCategoryAdminRequest {
            name: Some("a".repeat(256)),
            parent_id: None,
            image_id: None,
            position: None,
        };
        assert!(req.validate().is_err());
    }
}
