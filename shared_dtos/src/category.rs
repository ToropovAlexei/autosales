use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::rust::double_option;
use uuid::Uuid;

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryBotResponse {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
    pub image_id: Option<Uuid>,
    pub position: i16,
    pub is_active: bool,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "category.ts", rename = "Category")
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryAdminResponse {
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

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "validate", derive(validator::Validate))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "category.ts", rename = "NewCategory")
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewCategoryAdminRequest {
    #[cfg_attr(
        feature = "validate",
        validate(length(
            min = 2,
            max = 255,
            message = "Category name must be at least 2 characters and at most 255 characters long"
        ))
    )]
    pub name: String,
    #[cfg_attr(feature = "ts", ts(optional))]
    pub parent_id: Option<i64>,
    #[cfg_attr(feature = "ts", ts(optional))]
    pub image_id: Option<Uuid>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "validate", derive(validator::Validate))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "category.ts", rename = "UpdateCategory")
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCategoryAdminRequest {
    #[cfg_attr(
        feature = "validate",
        validate(length(
            min = 2,
            max = 255,
            message = "Category name must be at least 2 characters and at most 255 characters long"
        ))
    )]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub name: Option<String>,
    #[cfg_attr(feature = "ts", ts(optional))]
    #[cfg_attr(feature = "ts", ts(type = "number | null"))]
    #[serde(default, with = "double_option")]
    pub parent_id: Option<Option<i64>>,
    #[cfg_attr(feature = "ts", ts(optional))]
    #[cfg_attr(feature = "ts", ts(type = "string | null"))]
    #[serde(default, with = "double_option")]
    pub image_id: Option<Option<Uuid>>,
    #[cfg_attr(feature = "ts", ts(optional))]
    pub position: Option<i16>,
}
