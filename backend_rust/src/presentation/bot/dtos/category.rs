use serde::{Deserialize, Serialize};
use utoipa::{ToResponse, ToSchema};
use uuid::Uuid;

use crate::models::category::CategoryRow;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, ToResponse)]
pub struct CategoryResponse {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
    pub image_id: Option<Uuid>,
    pub position: i16,
    pub is_active: bool,
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
        }
    }
}
