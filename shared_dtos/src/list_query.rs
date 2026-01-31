use chrono::DateTime;
use chrono::Utc;
use serde::Deserializer;
use serde::{Deserialize, Serialize};
use serde_with::DisplayFromStr;
use serde_with::serde_as;
use uuid::Uuid;

/// A raw version of Filter used for initial deserialization
#[derive(Debug, Deserialize, Serialize)]
pub struct RawFilter {
    pub field: String,
    pub op: Operator,
    pub value: FilterValue,
}

/// A raw version of ListQuery for initial deserialization from a query string.
/// Fields that require validation (filters, order_by) are taken as strings.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct RawListQuery {
    #[serde(default)]
    pub filters: Vec<RawFilter>,
    #[serde(flatten)]
    pub pagination: Pagination,
    #[serde(default)]
    pub order_by: Option<String>,
    #[serde(default)]
    pub order_dir: OrderDir,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum OrderDir {
    #[default]
    Desc,
    Asc,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    #[serde_as(as = "DisplayFromStr")]
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(default = "default_page_size")]
    pub page_size: u32,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: 1,
            page_size: 10,
        }
    }
}

fn default_page() -> u32 {
    1
}
fn default_page_size() -> u32 {
    10
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Operator {
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    Like,
    Contains,
    In,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FilterValue {
    Scalar(ScalarValue),
    Array(Vec<ScalarValue>),
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum ScalarValue {
    Int(i64),
    Float(f64),
    Text(String),
    Uuid(Uuid),
    Bool(bool),
    DateTime(DateTime<Utc>),
}

impl<'de> Deserialize<'de> for ScalarValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        if s == "true" || s == "false" {
            return Ok(ScalarValue::Bool(s == "true"));
        }

        if let Ok(dt) = s.parse::<DateTime<Utc>>() {
            return Ok(ScalarValue::DateTime(dt));
        }

        if let Ok(uuid) = Uuid::parse_str(&s) {
            return Ok(ScalarValue::Uuid(uuid));
        }

        if let Ok(i) = s.parse::<i64>() {
            return Ok(ScalarValue::Int(i));
        }

        if let Ok(f) = s.parse::<f64>() {
            return Ok(ScalarValue::Float(f));
        }

        Ok(ScalarValue::Text(s))
    }
}
