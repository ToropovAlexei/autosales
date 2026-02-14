use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::rust::double_option;
use uuid::Uuid;

use crate::list_query::{FilterValue, Operator, ScalarValue};

#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(feature = "sqlx", sqlx(type_name = "TEXT", rename_all = "snake_case"))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export, export_to = "broadcast.ts"))]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BroadcastStatus {
    Pending,
    Scheduled,
    InProgress,
    Completed,
    Failed,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonScalarValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    DateTime(DateTime<Utc>),
    Uuid(Uuid),
    Text(String),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonFilterValue {
    Scalar(JsonScalarValue),
    Array(Vec<JsonScalarValue>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRawFilter {
    pub field: String,
    pub op: Operator,
    pub value: JsonFilterValue,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRawListQuery {
    pub filters: Vec<JsonRawFilter>,
}

impl From<JsonScalarValue> for ScalarValue {
    fn from(val: JsonScalarValue) -> Self {
        match val {
            JsonScalarValue::Int(i) => ScalarValue::Int(i),
            JsonScalarValue::Float(f) => ScalarValue::Float(f),
            JsonScalarValue::Bool(b) => ScalarValue::Bool(b),
            JsonScalarValue::Uuid(u) => ScalarValue::Uuid(u),
            JsonScalarValue::Text(t) => ScalarValue::Text(t),
            JsonScalarValue::DateTime(dt) => ScalarValue::DateTime(dt),
        }
    }
}

impl From<JsonFilterValue> for FilterValue {
    fn from(val: JsonFilterValue) -> Self {
        match val {
            JsonFilterValue::Scalar(s) => FilterValue::Scalar(s.into()),
            JsonFilterValue::Array(a) => {
                FilterValue::Array(a.into_iter().map(Into::into).collect())
            }
        }
    }
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "validate", derive(validator::Validate))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "broadcast.ts", rename = "NewBroadcast")
)]
#[derive(Debug, Deserialize)]
pub struct NewBroadcastRequest {
    #[cfg_attr(
        feature = "validate",
        validate(length(max = 1024, message = "Content text is too long"))
    )]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub content_text: Option<String>,
    #[cfg_attr(feature = "ts", ts(optional))]
    pub content_image_id: Option<Uuid>,
    #[cfg_attr(feature = "openapi", schema(value_type = Object))]
    #[cfg_attr(feature = "ts", ts(type = "any"))]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub filters: Option<JsonRawListQuery>,
    #[cfg_attr(feature = "ts", ts(optional))]
    pub scheduled_for: Option<DateTime<Utc>>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "validate", derive(validator::Validate))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "broadcast.ts", rename = "UpdateBroadcast")
)]
#[derive(Debug, Deserialize)]
pub struct UpdateBroadcastRequest {
    #[cfg_attr(
        feature = "validate",
        validate(length(max = 1024, message = "Content text is too long"))
    )]
    #[cfg_attr(feature = "ts", ts(optional))]
    #[cfg_attr(feature = "ts", ts(type = "string | null"))]
    #[serde(default, with = "double_option")]
    pub content_text: Option<Option<String>>,
    #[cfg_attr(feature = "ts", ts(optional))]
    #[cfg_attr(feature = "ts", ts(type = "string | null"))]
    #[serde(default, with = "double_option")]
    pub content_image_id: Option<Option<Uuid>>,
    #[cfg_attr(feature = "openapi", schema(value_type = Object))]
    #[cfg_attr(feature = "ts", ts(type = "any"))]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub filters: Option<JsonRawListQuery>,
    #[cfg_attr(feature = "ts", ts(optional))]
    #[cfg_attr(feature = "ts", ts(type = "string | null"))]
    #[serde(default, with = "double_option")]
    pub scheduled_for: Option<Option<DateTime<Utc>>>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "broadcast.ts", rename = "Broadcast")
)]
#[derive(Debug, Clone, Serialize)]
pub struct BroadcastResponse {
    pub id: i64,
    pub status: BroadcastStatus,
    pub content_text: Option<String>,
    pub content_image_id: Option<Uuid>,
    pub filters: Option<serde_json::Value>,
    pub statistics: Option<serde_json::Value>,
    pub created_by: i64,
    pub scheduled_for: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
