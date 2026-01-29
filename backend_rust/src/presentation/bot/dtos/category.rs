use shared_dtos::category::CategoryBotResponse;

use crate::models::category::CategoryRow;

impl From<CategoryRow> for CategoryBotResponse {
    fn from(r: CategoryRow) -> Self {
        CategoryBotResponse {
            id: r.id,
            name: r.name,
            parent_id: r.parent_id,
            image_id: r.image_id,
            position: r.position,
            is_active: r.is_active,
        }
    }
}
