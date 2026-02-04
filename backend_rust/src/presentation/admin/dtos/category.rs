use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::rust::double_option;
use ts_rs::TS;
use utoipa::{ToResponse, ToSchema};
use uuid::Uuid;
use validator::Validate;

use crate::models::category::CategoryRow;

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema, ToResponse)]
#[ts(export, export_to = "category.ts", rename = "Category")]
pub struct CategoryResponse {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
    pub image_id: Option<Uuid>,
    pub position: i16,
    pub is_active: bool,
    pub created_by: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<CategoryRow> for CategoryResponse {
    fn from(r: CategoryRow) -> Self {
        CategoryResponse {
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

#[derive(Debug, Clone, Serialize, Deserialize, Validate, TS, ToSchema, ToResponse)]
#[ts(export, export_to = "category.ts", rename = "NewCategory")]
pub struct NewCategoryRequest {
    #[validate(length(
        min = 2,
        max = 255,
        message = "Category name must be at least 2 characters and at most 255 characters long"
    ))]
    pub name: String,
    #[ts(optional)]
    pub parent_id: Option<i64>,
    #[ts(optional)]
    pub image_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, TS, ToSchema, ToResponse)]
#[ts(export, export_to = "category.ts", rename = "UpdateCategory")]
pub struct UpdateCategoryRequest {
    #[validate(length(
        min = 2,
        max = 255,
        message = "Category name must be at least 2 characters and at most 255 characters long"
    ))]
    #[ts(optional)]
    pub name: Option<String>,
    #[ts(optional)]
    #[ts(type = "number | null")]
    #[serde(default, with = "double_option")]
    pub parent_id: Option<Option<i64>>,
    #[ts(optional)]
    #[ts(type = "string | null")]
    #[serde(default, with = "double_option")]
    pub image_id: Option<Option<Uuid>>,
    #[ts(optional)]
    pub position: Option<i16>,
}

#[cfg(test)]
mod tests {
    use super::*;
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

        let category_response: CategoryResponse = category_row.into();

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

        let category_response: CategoryResponse = category_row.into();

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
        let req = NewCategoryRequest {
            name: "Valid Name".to_string(),
            parent_id: Some(1),
            image_id: Some(Uuid::new_v4()),
        };
        assert!(req.validate().is_ok());

        // Name too short
        let req = NewCategoryRequest {
            name: "a".to_string(),
            parent_id: None,
            image_id: None,
        };
        assert!(req.validate().is_err());

        // Name too long
        let req = NewCategoryRequest {
            name: "a".repeat(256),
            parent_id: None,
            image_id: None,
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_update_category_request_validation() {
        // Valid: All optional fields are None
        let req = UpdateCategoryRequest {
            name: None,
            parent_id: None,
            image_id: None,
            position: None,
        };
        assert!(req.validate().is_ok());

        // Valid: All fields provided and correct
        let req = UpdateCategoryRequest {
            name: Some("New Name".to_string()),
            parent_id: Some(Some(2)),
            image_id: Some(Some(Uuid::new_v4())),
            position: Some(5),
        };
        assert!(req.validate().is_ok());

        // Valid: Setting optional fields to None
        let req = UpdateCategoryRequest {
            name: None,
            parent_id: Some(None),
            image_id: Some(None),
            position: Some(5),
        };
        assert!(req.validate().is_ok());

        // Name too short
        let req = UpdateCategoryRequest {
            name: Some("a".to_string()),
            parent_id: None,
            image_id: None,
            position: None,
        };
        assert!(req.validate().is_err());

        // Name too long
        let req = UpdateCategoryRequest {
            name: Some("a".repeat(256)),
            parent_id: None,
            image_id: None,
            position: None,
        };
        assert!(req.validate().is_err());
    }
}
