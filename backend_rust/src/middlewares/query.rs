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
    filters: String,
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
        if !raw_query.filters.is_empty() {
            let raw_filters: Vec<RawFilter> = serde_json::from_str(&raw_query.filters)
                .map_err(|e| ApiError::BadRequest(format!("Invalid filter format: {}", e)))?;

            for raw_filter in raw_filters {
                let field = F::try_from(raw_filter.field)
                    .map_err(|e| ApiError::BadRequest(e.to_string()))?;
                filters.push(Filter {
                    field,
                    op: raw_filter.op,
                    value: raw_filter.value,
                });
            }
        }

        let order_by = match raw_query.order_by {
            Some(field_str) => {
                Some(O::try_from(field_str).map_err(|e| ApiError::BadRequest(e.to_string()))?)
            }
            None => None,
        };

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
