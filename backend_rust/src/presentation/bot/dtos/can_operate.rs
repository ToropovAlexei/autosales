use serde::{Deserialize, Serialize};
use utoipa::{ToResponse, ToSchema};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, ToResponse)]
pub struct CanOperateResponse {
    pub can_operate: bool,
}
