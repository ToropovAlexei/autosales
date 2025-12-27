use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// DTO for representing a category in API responses.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Category {
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

/// DTO for creating a new category.
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateCategoryDto {
    #[validate(length(min = 1, message = "Category name cannot be empty"))]
    pub name: String,
    pub parent_id: Option<i64>,
    pub image_id: Option<Uuid>,
}

/// DTO for updating an existing category.
#[derive(Debug, Clone, Deserialize, Validate, Default)]
pub struct UpdateCategoryDto {
    #[validate(length(min = 1, message = "Category name cannot be empty"))]
    pub name: Option<String>,
    pub parent_id: Option<Option<i64>>,
    pub image_id: Option<Option<Uuid>>,
    pub position: Option<i16>,
    pub is_active: Option<bool>,
}
