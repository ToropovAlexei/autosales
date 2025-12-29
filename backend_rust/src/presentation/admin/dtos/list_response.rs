use serde::Serialize;
use utoipa::{ToResponse, ToSchema};

#[derive(Debug, ToSchema, ToResponse, Serialize)]
pub struct ListResponse<T>
where
    T: ToSchema,
{
    pub items: Vec<T>,
    pub total: i64,
}
