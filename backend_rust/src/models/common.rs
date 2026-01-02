use serde::{Deserialize, Serialize};
use serde_with::DisplayFromStr;
use serde_with::serde_as;
use uuid::Uuid;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ScalarValue {
    Int(i64),
    Float(f64),
    Text(String),
    Uuid(Uuid),
    Bool(bool),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filter {
    pub field: String,
    pub op: Operator,
    pub value: FilterValue,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListQuery {
    #[serde(default)]
    pub filters: Vec<Filter>,
    #[serde(flatten)]
    pub pagination: Pagination,
    #[serde(default)]
    pub order_by: Option<String>,
    #[serde(default)]
    pub order_dir: OrderDir,
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
