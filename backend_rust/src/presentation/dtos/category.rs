use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::models::category::CategoryRow;

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct NewCategoryRequest {
    #[validate(length(
        min = 2,
        max = 255,
        message = "Category name must be at least 2 characters and at most 255 characters long"
    ))]
    pub name: String,
    pub parent_id: Option<i64>,
    pub image_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateCategoryRequest {
    #[validate(length(
        min = 2,
        max = 255,
        message = "Category name must be at least 2 characters and at most 255 characters long"
    ))]
    pub name: Option<String>,
    pub parent_id: Option<Option<i64>>,
    pub image_id: Option<Option<Uuid>>,
    pub position: Option<i16>,
}
