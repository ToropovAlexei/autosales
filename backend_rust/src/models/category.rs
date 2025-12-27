use chrono::{DateTime, Utc};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(FromRow, Debug)]
pub struct CategoryRow {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
    pub image_id: Option<Uuid>,
    pub position: i16,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: i64,
}

#[derive(Debug)]
pub struct NewCategory {
    pub name: String,
    pub parent_id: Option<i64>,
    pub image_id: Option<Uuid>,
    pub created_by: i64,
}

#[derive(Debug)]
pub struct UpdateCategory {
    pub name: Option<String>,
    pub parent_id: Option<Option<i64>>,
    pub image_id: Option<Option<Uuid>>,
    pub position: Option<i16>,
}
