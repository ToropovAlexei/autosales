use chrono::{DateTime, Utc};
use serde::Deserializer;
use serde::{Deserialize, Serialize};
use serde_with::DisplayFromStr;
use serde_with::serde_as;
use std::fmt::Debug;
use std::marker::PhantomData;
use uuid::Uuid;

pub trait AllowedField:
    for<'de> Deserialize<'de> + Serialize + Debug + Clone + PartialEq + Eq + AsRef<str> + Send + Sync
{
}

impl<T> AllowedField for T where
    T: for<'de> Deserialize<'de>
        + Serialize
        + Debug
        + Clone
        + PartialEq
        + Eq
        + AsRef<str>
        + Send
        + Sync
{
}

#[derive(Debug, Serialize)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub total: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum OrderDir {
    #[default]
    Desc,
    Asc,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Pagination {
    #[serde_as(as = "DisplayFromStr")]
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(default = "default_page_size")]
    pub page_size: u32,
}

fn default_page() -> u32 {
    1
}
fn default_page_size() -> u32 {
    10
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FilterValue {
    Scalar(ScalarValue),
    Array(Vec<ScalarValue>),
}

#[derive(Debug, Clone, Serialize)]
pub enum ScalarValue {
    Int(i64),
    Float(f64),
    Text(String),
    Uuid(Uuid),
    Bool(bool),
    DateTime(DateTime<Utc>),
}

#[derive(Debug, Clone, Serialize)]
pub struct Filter<F: AllowedField> {
    pub field: F,
    pub op: Operator,
    pub value: FilterValue,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct ListQuery<F: AllowedField, O: AllowedField> {
    pub filters: Vec<Filter<F>>,
    pub pagination: Pagination,
    pub order_by: Option<O>,
    pub order_dir: OrderDir,
    #[serde(skip)]
    pub _phantom: PhantomData<(F, O)>,
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

#[macro_export]
macro_rules! define_list_query {
    (
        query_name: $query_name:ident,
        filter_fields: { $filter_enum:ident, [$($filter_variant:ident => $filter_str:literal),* $(,)?] },
        order_fields: { $order_enum:ident, [$($order_variant:ident => $order_str:literal),* $(,)?] }
    ) => {
        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub enum $filter_enum {
            $( $filter_variant, )*
        }

        impl AsRef<str> for $filter_enum {
            fn as_ref(&self) -> &str {
                match self {
                    $( Self::$filter_variant => $filter_str, )*
                }
            }
        }

        impl std::convert::TryFrom<String> for $filter_enum {
            type Error = String;
            fn try_from(value: String) -> Result<Self, Self::Error> {
                match value.as_str() {
                    $( $filter_str => Ok(Self::$filter_variant), )*
                    _ => Err(format!("Unknown filter field: {}", value)),
                }
            }
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub enum $order_enum {
            $( $order_variant, )*
        }

        impl AsRef<str> for $order_enum {
            fn as_ref(&self) -> &str {
                match self {
                    $( Self::$order_variant => $order_str, )*
                }
            }
        }

        impl std::convert::TryFrom<String> for $order_enum {
            type Error = String;
            fn try_from(value: String) -> Result<Self, Self::Error> {
                match value.as_str() {
                    $( $order_str => Ok(Self::$order_variant), )*
                    _ => Err(format!("Unknown order field: {value}")),
                }
            }
        }

        pub type $query_name = $crate::models::common::ListQuery<$filter_enum, $order_enum>;
    };
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
