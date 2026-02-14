use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "image.ts", rename = "ImageResponse")
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageAdminResponse {
    pub id: Uuid,
    pub context: String,
    pub created_at: DateTime<Utc>,
    pub created_by: i64,
}
