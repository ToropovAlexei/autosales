use std::convert::TryFrom;

use axum::extract::{FromRequest, Request};
use serde::Deserialize;

use crate::{
    errors::api::ApiError,
    models::common::{
        AllowedField, Filter, FilterValue, ListQuery, Operator, OrderDir, Pagination,
    },
};

/// A raw version of Filter used for initial deserialization
#[derive(Debug, Deserialize)]
struct RawFilter {
    field: String,
    op: Operator,
    value: FilterValue,
}

/// A raw version of ListQuery for initial deserialization from a query string.
/// Fields that require validation (filters, order_by) are taken as strings.
#[derive(Debug, Deserialize, Default)]
struct RawListQuery {
    #[serde(default)]
    filters: Vec<RawFilter>,
    #[serde(flatten)]
    pagination: Pagination,
    #[serde(default)]
    order_by: Option<String>,
    #[serde(default)]
    order_dir: OrderDir,
}

impl<S, F, O> FromRequest<S> for ListQuery<F, O>
where
    S: Send + Sync,
    F: AllowedField + TryFrom<String, Error = String> + Send,
    O: AllowedField + TryFrom<String, Error = String> + Send,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, _state: &S) -> Result<Self, Self::Rejection> {
        let query = req.uri().query().unwrap_or_default();
        let raw_query: RawListQuery =
            serde_qs::from_str(query).map_err(|e| ApiError::BadRequest(e.to_string()))?;

        let mut filters = Vec::new();
        for raw_filter in raw_query.filters {
            let field =
                F::try_from(raw_filter.field).map_err(|e| ApiError::BadRequest(e.to_string()))?;
            filters.push(Filter {
                field,
                op: raw_filter.op,
                value: raw_filter.value,
            });
        }

        let order_by = match raw_query.order_by {
            Some(field_str) => {
                Some(O::try_from(field_str).map_err(|e| ApiError::BadRequest(e.to_string()))?)
            }
            None => None,
        };

        if raw_query.pagination.page < 1 {
            return Err(ApiError::BadRequest(
                "Page number must be at least 1".to_string(),
            ));
        }

        if raw_query.pagination.page_size > 100 {
            return Err(ApiError::BadRequest(
                "Page size cannot exceed 100".to_string(),
            ));
        }

        Ok(ListQuery {
            filters,
            pagination: raw_query.pagination,
            order_by,
            order_dir: raw_query.order_dir,
            _phantom: std::marker::PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        define_list_query,
        models::common::{FilterValue, ScalarValue},
    };
    use axum::{body::Body, http::Request};
    use serde::{Deserialize, Serialize};

    define_list_query! {
        query_name: TestListQuery,
        filter_fields: {
            TestFilterFields,
            [
                Age => "age",
                Name => "name",
                Type => "type",
            ]
        },
        order_fields: {
            TestOrderFields,
            [
                Name => "name",
                CreatedAt => "created_at",
            ]
        }
    }

    async fn execute_from_request(uri: &str) -> Result<TestListQuery, ApiError> {
        let req = Request::builder().uri(uri).body(Body::empty()).unwrap();
        TestListQuery::from_request(req, &()).await
    }

    #[tokio::test]
    async fn test_full_query() {
        let uri = "http://localhost?filters[0][field]=name&filters[0][op]=like&filters[0][value]=test&filters[1][field]=age&filters[1][op]=ge&filters[1][value]=18&page=2&page_size=20&order_by=created_at&order_dir=asc";
        let query = execute_from_request(uri).await.unwrap();

        assert_eq!(query.filters.len(), 2);
        assert_eq!(query.filters[0].field, TestFilterFields::Name);
        assert_eq!(query.filters[0].op, Operator::Like);
        assert!(
            matches!(query.filters[0].value, FilterValue::Scalar(ScalarValue::Text(ref s)) if s == "test")
        );

        assert_eq!(query.filters[1].field, TestFilterFields::Age);
        assert_eq!(query.filters[1].op, Operator::Ge);
        assert!(matches!(
            query.filters[1].value,
            FilterValue::Scalar(ScalarValue::Int(18))
        ));

        assert_eq!(query.pagination.page, 2);
        assert_eq!(query.pagination.page_size, 20);
        assert_eq!(query.order_by, Some(TestOrderFields::CreatedAt));
        assert_eq!(query.order_dir, OrderDir::Asc);
    }

    #[tokio::test]
    async fn test_empty_query() {
        let query = execute_from_request("http://localhost").await.unwrap();
        assert!(query.filters.is_empty());
        assert_eq!(query.pagination.page, 1);
        assert_eq!(query.pagination.page_size, 10);
        assert!(query.order_by.is_none());
        assert_eq!(query.order_dir, OrderDir::Desc);
    }

    #[tokio::test]
    async fn test_filter_with_array_value() {
        let uri = "http://localhost?filters[0][field]=type&filters[0][op]=in&filters[0][value][]=type1&filters[0][value][]=type2";
        let query = execute_from_request(uri).await.unwrap();

        assert_eq!(query.filters.len(), 1);
        assert_eq!(query.filters[0].field, TestFilterFields::Type);
        assert_eq!(query.filters[0].op, Operator::In);

        let expected_values = [
            ScalarValue::Text("type1".to_string()),
            ScalarValue::Text("type2".to_string()),
        ];
        assert!(
            matches!(query.filters[0].value, FilterValue::Array(ref values) if values.iter().all(|v|
                expected_values.iter().any(|ev| match (v, ev) {
                    (ScalarValue::Text(s1), ScalarValue::Text(s2)) => s1 == s2,
                    _ => false
                })
            ))
        );
    }

    #[tokio::test]
    async fn test_filter_with_array_of_int() {
        let uri = "http://localhost?filters[0][field]=type&filters[0][op]=in&filters[0][value][]=1&filters[0][value][]=2";
        let query = execute_from_request(uri).await.unwrap();

        assert_eq!(query.filters.len(), 1);
        assert_eq!(query.filters[0].field, TestFilterFields::Type);
        assert_eq!(query.filters[0].op, Operator::In);

        let expected_values = [ScalarValue::Int(1), ScalarValue::Int(2)];
        assert!(
            matches!(query.filters[0].value, FilterValue::Array(ref values) if values.iter().all(|v|
                expected_values.iter().any(|ev| match (v, ev) {
                    (ScalarValue::Int(s1), ScalarValue::Int(s2)) => s1 == s2,
                    _ => false
                })
            ))
        );
    }

    #[tokio::test]
    async fn test_invalid_filter_field() {
        let result = execute_from_request(
            "http://localhost?filters[0][field]=invalid&filters[0][op]=eq&filters[0][value]=test",
        )
        .await;
        assert!(matches!(result, Err(ApiError::BadRequest(_))));
    }

    #[tokio::test]
    async fn test_invalid_order_field() {
        let result = execute_from_request("http://localhost?order_by=invalid").await;
        assert!(matches!(result, Err(ApiError::BadRequest(_))));
    }

    #[tokio::test]
    async fn test_pagination_limits() {
        let result = execute_from_request("http://localhost?page=0").await;
        assert!(matches!(result, Err(ApiError::BadRequest(_))));

        let result = execute_from_request("http://localhost?page_size=101").await;
        assert!(matches!(result, Err(ApiError::BadRequest(_))));
    }
}
