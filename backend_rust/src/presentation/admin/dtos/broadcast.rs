use crate::models::common::Filter;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::{ToResponse, ToSchema};
use uuid::Uuid;
use validator::Validate;

use crate::models::{
    broadcast::{BroadcastRow, BroadcastStatus},
    common::{FilterValue, Operator, ScalarValue},
    customer::{CustomerFilterFields, CustomerListQuery},
};

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

impl CustomerListQuery {
    pub fn try_from_json(json_val: JsonRawListQuery) -> Result<Self, String> {
        let mut filters = Vec::new();
        for raw_filter in json_val.filters {
            let field =
                CustomerFilterFields::try_from(raw_filter.field).map_err(|e| e.to_string())?;

            filters.push(Filter {
                field,
                op: raw_filter.op,
                value: raw_filter.value.into(),
            });
        }
        Ok(CustomerListQuery {
            filters,
            pagination: Default::default(),
            order_by: None,
            order_dir: Default::default(),
            _phantom: std::marker::PhantomData,
        })
    }
}

#[derive(Debug, Deserialize, Validate, TS, ToSchema, ToResponse)]
#[ts(export, export_to = "broadcast.ts", rename = "NewBroadcast")]
pub struct NewBroadcastRequest {
    #[validate(length(max = 1024, message = "Content text is too long"))]
    #[ts(optional)]
    pub content_text: Option<String>,
    #[ts(optional)]
    pub content_image_id: Option<Uuid>,
    #[schema(value_type = Object)]
    #[ts(type = "any")]
    #[ts(optional)]
    pub filters: Option<JsonRawListQuery>,
    #[ts(optional)]
    pub scheduled_for: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Validate, TS, ToSchema, ToResponse)]
#[ts(export, export_to = "broadcast.ts", rename = "UpdateBroadcast")]
pub struct UpdateBroadcastRequest {
    #[validate(length(max = 1024, message = "Content text is too long"))]
    #[ts(optional)]
    pub content_text: Option<Option<String>>,
    #[ts(optional)]
    pub content_image_id: Option<Option<Uuid>>,
    #[schema(value_type = Object)]
    #[ts(type = "any")]
    #[ts(optional)]
    pub filters: Option<JsonRawListQuery>,
    #[ts(optional)]
    pub scheduled_for: Option<Option<DateTime<Utc>>>,
}

#[derive(Debug, Clone, Serialize, TS, ToSchema)]
#[ts(export, export_to = "broadcast.ts", rename = "Broadcast")]
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

impl From<BroadcastRow> for BroadcastResponse {
    fn from(r: BroadcastRow) -> Self {
        BroadcastResponse {
            id: r.id,
            status: r.status,
            content_text: r.content_text,
            content_image_id: r.content_image_id,
            filters: r.filters,
            statistics: r.statistics,
            created_by: r.created_by,
            scheduled_for: r.scheduled_for,
            started_at: r.started_at,
            finished_at: r.finished_at,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}
