use crate::models::common::Filter;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::rust::double_option;
use shared_dtos::list_query::{FilterValue, Operator, ScalarValue};
use ts_rs::TS;
use utoipa::{ToResponse, ToSchema};
use uuid::Uuid;
use validator::Validate;

use crate::models::{
    broadcast::{BroadcastRow, BroadcastStatus},
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
    #[ts(type = "string | null")]
    #[serde(default, with = "double_option")]
    pub content_text: Option<Option<String>>,
    #[ts(optional)]
    #[ts(type = "string | null")]
    #[serde(default, with = "double_option")]
    pub content_image_id: Option<Option<Uuid>>,
    #[schema(value_type = Object)]
    #[ts(type = "any")]
    #[ts(optional)]
    pub filters: Option<JsonRawListQuery>,
    #[ts(optional)]
    #[ts(type = "string | null")]
    #[serde(default, with = "double_option")]
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;
    use serde_json::json;
    use validator::Validate;

    #[test]
    fn test_json_raw_list_query_deserialization_scalar_int() {
        let json_str = r#"{
            "filters": [
                {
                    "field": "balance",
                    "op": "gt",
                    "value": 500
                }
            ]
        }"#;
        let query: JsonRawListQuery = serde_json::from_str(json_str).unwrap();
        assert_eq!(query.filters.len(), 1);
        assert_eq!(query.filters[0].field, "balance");
        assert_eq!(query.filters[0].op, Operator::Gt);
        match &query.filters[0].value {
            JsonFilterValue::Scalar(JsonScalarValue::Int(val)) => assert_eq!(*val, 500),
            _ => panic!("Expected Int scalar value"),
        }
    }

    #[test]
    fn test_json_raw_list_query_deserialization_scalar_float() {
        let json_str = r#"{
            "filters": [
                {
                    "field": "amount",
                    "op": "ge",
                    "value": 123.45
                }
            ]
        }"#;
        let query: JsonRawListQuery = serde_json::from_str(json_str).unwrap();
        match &query.filters[0].value {
            JsonFilterValue::Scalar(JsonScalarValue::Float(val)) => assert_eq!(*val, 123.45),
            _ => panic!("Expected Float scalar value"),
        }
    }

    #[test]
    fn test_json_raw_list_query_deserialization_scalar_text() {
        let json_str = r#"{
            "filters": [
                {
                    "field": "name",
                    "op": "eq",
                    "value": "John Doe"
                }
            ]
        }"#;
        let query: JsonRawListQuery = serde_json::from_str(json_str).unwrap();
        match &query.filters[0].value {
            JsonFilterValue::Scalar(JsonScalarValue::Text(val)) => assert_eq!(*val, "John Doe"),
            _ => panic!("Expected Text scalar value"),
        }
    }

    #[test]
    fn test_json_raw_list_query_deserialization_scalar_uuid() {
        let uuid_str = "a1b2c3d4-e5f6-7890-1234-567890abcdef";
        let json_str = format!(
            r#"{{"filters": [{{"field": "id", "op": "eq", "value": "{}"}}]}}"#,
            uuid_str
        );
        let query: JsonRawListQuery = serde_json::from_str(&json_str).unwrap();
        match &query.filters[0].value {
            JsonFilterValue::Scalar(JsonScalarValue::Uuid(val)) => {
                assert_eq!(val.to_string(), uuid_str)
            }
            _ => panic!("Expected Uuid scalar value"),
        }
    }

    #[test]
    fn test_json_raw_list_query_deserialization_scalar_bool() {
        let json_str = r#"{
            "filters": [
                {
                    "field": "active",
                    "op": "eq",
                    "value": true
                }
            ]
        }"#;
        let query: JsonRawListQuery = serde_json::from_str(json_str).unwrap();
        match &query.filters[0].value {
            JsonFilterValue::Scalar(JsonScalarValue::Bool(val)) => assert!(*val),
            _ => panic!("Expected Bool scalar value"),
        }
    }

    #[test]
    fn test_json_raw_list_query_deserialization_scalar_datetime() {
        let datetime_str = "2026-01-18T10:30:00Z";
        let json_str = format!(
            r#"{{"filters": [{{"field": "created_at", "op": "gt", "value": "{}"}}]}}"#,
            datetime_str
        );
        let query: JsonRawListQuery = serde_json::from_str(&json_str).unwrap();
        match &query.filters[0].value {
            JsonFilterValue::Scalar(JsonScalarValue::DateTime(val)) => {
                let expected_dt: DateTime<Utc> = datetime_str.parse().unwrap();
                assert_eq!(*val, expected_dt);
            }
            _ => panic!("Expected DateTime scalar value"),
        }
    }

    #[test]
    fn test_json_raw_list_query_deserialization_array() {
        let json_str = r#"{
            "filters": [
                {
                    "field": "category_id",
                    "op": "in",
                    "value": [1, 2, 3]
                }
            ]
        }"#;
        let query: JsonRawListQuery = serde_json::from_str(json_str).unwrap();
        match &query.filters[0].value {
            JsonFilterValue::Array(vals) => {
                assert_eq!(vals.len(), 3);
                match &vals[0] {
                    JsonScalarValue::Int(val) => assert_eq!(*val, 1),
                    _ => panic!("Expected Int in array"),
                }
            }
            _ => panic!("Expected Array filter value"),
        }
    }

    #[test]
    fn test_try_from_json_conversion() {
        let json_raw_query = JsonRawListQuery {
            filters: vec![
                JsonRawFilter {
                    field: "balance".to_string(),
                    op: Operator::Gt,
                    value: JsonFilterValue::Scalar(JsonScalarValue::Int(500)),
                },
                JsonRawFilter {
                    field: "is_blocked".to_string(),
                    op: Operator::Eq,
                    value: JsonFilterValue::Scalar(JsonScalarValue::Bool(true)),
                },
                JsonRawFilter {
                    field: "last_seen_at".to_string(),
                    op: Operator::Lt,
                    value: JsonFilterValue::Scalar(JsonScalarValue::DateTime(Utc::now())),
                },
            ],
        };

        let customer_list_query = CustomerListQuery::try_from_json(json_raw_query).unwrap();
        assert_eq!(customer_list_query.filters.len(), 3);

        match &customer_list_query.filters[0].value {
            FilterValue::Scalar(ScalarValue::Int(val)) => assert_eq!(*val, 500),
            _ => panic!("Expected Int scalar value"),
        }
        match &customer_list_query.filters[1].value {
            FilterValue::Scalar(ScalarValue::Bool(val)) => assert!(*val),
            _ => panic!("Expected Bool scalar value"),
        }
        match &customer_list_query.filters[2].value {
            FilterValue::Scalar(ScalarValue::DateTime(_)) => {}
            _ => panic!("Expected DateTime scalar value"),
        }
    }

    #[test]
    fn test_try_from_json_conversion_invalid_field() {
        let json_raw_query = JsonRawListQuery {
            filters: vec![JsonRawFilter {
                field: "invalid_field".to_string(),
                op: Operator::Gt,
                value: JsonFilterValue::Scalar(JsonScalarValue::Int(100)),
            }],
        };

        let result = CustomerListQuery::try_from_json(json_raw_query);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Unknown filter field: invalid_field")
        );
    }

    #[test]
    fn test_json_raw_list_query_serialization_scalar_int() {
        let json_raw_query = JsonRawListQuery {
            filters: vec![JsonRawFilter {
                field: "balance".to_string(),
                op: Operator::Gt,
                value: JsonFilterValue::Scalar(JsonScalarValue::Int(500)),
            }],
        };
        let serialized = serde_json::to_string(&json_raw_query).unwrap();
        let expected = r#"{"filters":[{"field":"balance","op":"gt","value":500}]}"#;
        assert_eq!(serialized, expected);
    }

    #[test]
    fn test_json_raw_list_query_serialization_scalar_datetime() {
        let dt = Utc::now().with_nanosecond(0).unwrap(); // Remove nanos for consistent serialization
        let json_raw_query = JsonRawListQuery {
            filters: vec![JsonRawFilter {
                field: "created_at".to_string(),
                op: Operator::Gt,
                value: JsonFilterValue::Scalar(JsonScalarValue::DateTime(dt)),
            }],
        };
        let serialized = serde_json::to_string(&json_raw_query).unwrap();
        let expected_json = serde_json::json!({
            "filters": [{
                "field": "created_at",
                "op": "gt",
                "value": dt
            }]
        });
        assert_eq!(serialized, expected_json.to_string());
    }

    #[test]
    fn test_new_broadcast_request_validation() {
        // Valid: only content_text
        let req = NewBroadcastRequest {
            content_text: Some("short text".to_string()),
            content_image_id: None,
            filters: None,
            scheduled_for: None,
        };
        assert!(req.validate().is_ok());

        // Valid: only content_image_id
        let req = NewBroadcastRequest {
            content_text: None,
            content_image_id: Some(Uuid::new_v4()),
            filters: None,
            scheduled_for: None,
        };
        assert!(req.validate().is_ok());

        // Valid: both content_text and content_image_id
        let req = NewBroadcastRequest {
            content_text: Some("short text".to_string()),
            content_image_id: Some(Uuid::new_v4()),
            filters: None,
            scheduled_for: None,
        };
        assert!(req.validate().is_ok());

        // Invalid: content_text too long
        let req = NewBroadcastRequest {
            content_text: Some("a".repeat(1025)),
            content_image_id: None,
            filters: None,
            scheduled_for: None,
        };
        assert!(req.validate().is_err());

        // Valid: with filters
        let req = NewBroadcastRequest {
            content_text: Some("short text".to_string()),
            content_image_id: None,
            filters: Some(JsonRawListQuery { filters: vec![] }),
            scheduled_for: None,
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_update_broadcast_request_validation() {
        // Valid: only content_text
        let req = UpdateBroadcastRequest {
            content_text: Some(Some("short text".to_string())),
            content_image_id: None,
            filters: None,
            scheduled_for: None,
        };
        assert!(req.validate().is_ok());

        // Valid: content_text set to None
        let req = UpdateBroadcastRequest {
            content_text: Some(None),
            content_image_id: None,
            filters: None,
            scheduled_for: None,
        };
        assert!(req.validate().is_ok());

        // Valid: content_image_id
        let req = UpdateBroadcastRequest {
            content_text: None,
            content_image_id: Some(Some(Uuid::new_v4())),
            filters: None,
            scheduled_for: None,
        };
        assert!(req.validate().is_ok());

        // Valid: content_image_id set to None
        let req = UpdateBroadcastRequest {
            content_text: None,
            content_image_id: Some(None),
            filters: None,
            scheduled_for: None,
        };
        assert!(req.validate().is_ok());

        // Invalid: content_text too long
        let req = UpdateBroadcastRequest {
            content_text: Some(Some("a".repeat(1025))),
            content_image_id: None,
            filters: None,
            scheduled_for: None,
        };
        assert!(req.validate().is_err());

        // Valid: with filters
        let req = UpdateBroadcastRequest {
            content_text: Some(Some("short text".to_string())),
            content_image_id: None,
            filters: Some(JsonRawListQuery { filters: vec![] }),
            scheduled_for: None,
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_broadcast_response_from_broadcast_row_full() {
        let now = Utc::now();
        let row = BroadcastRow {
            id: 1,
            status: BroadcastStatus::Scheduled,
            content_text: Some("test broadcast content".to_string()),
            content_image_id: Some(Uuid::new_v4()),
            filters: Some(json!({"filter_key": "filter_value"})),
            statistics: Some(json!({"stats_key": "stats_value"})),
            created_by: 101,
            scheduled_for: Some(now + chrono::Duration::hours(1)),
            started_at: Some(now),
            finished_at: Some(now + chrono::Duration::minutes(5)),
            created_at: now,
            updated_at: now,
        };

        let response: BroadcastResponse = row.into();

        assert_eq!(response.id, 1);
        assert_eq!(response.status, BroadcastStatus::Scheduled);
        assert_eq!(
            response.content_text,
            Some("test broadcast content".to_string())
        );
        assert!(response.content_image_id.is_some());
        assert_eq!(
            response.filters,
            Some(json!({"filter_key": "filter_value"}))
        );
        assert_eq!(
            response.statistics,
            Some(json!({"stats_key": "stats_value"}))
        );
        assert_eq!(response.created_by, 101);
        assert!(response.scheduled_for.is_some());
        assert!(response.started_at.is_some());
        assert!(response.finished_at.is_some());
        assert_eq!(response.created_at, now);
        assert_eq!(response.updated_at, now);
    }

    #[test]
    fn test_broadcast_response_from_broadcast_row_minimal() {
        let now = Utc::now();
        let row = BroadcastRow {
            id: 2,
            status: BroadcastStatus::Pending,
            content_text: None,
            content_image_id: None,
            filters: None,
            statistics: None,
            created_by: 102,
            scheduled_for: None,
            started_at: None,
            finished_at: None,
            created_at: now,
            updated_at: now,
        };

        let response: BroadcastResponse = row.into();

        assert_eq!(response.id, 2);
        assert_eq!(response.status, BroadcastStatus::Pending);
        assert_eq!(response.content_text, None);
        assert_eq!(response.content_image_id, None);
        assert_eq!(response.filters, None);
        assert_eq!(response.statistics, None);
        assert_eq!(response.created_by, 102);
        assert_eq!(response.scheduled_for, None);
        assert_eq!(response.started_at, None);
        assert_eq!(response.finished_at, None);
        assert_eq!(response.created_at, now);
        assert_eq!(response.updated_at, now);
    }
}
