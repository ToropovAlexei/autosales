use axum::extract::{FromRequest, Request};

use crate::{errors::api::ApiError, models::common::ListQuery};

impl<S> FromRequest<S> for ListQuery
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, _state: &S) -> Result<Self, Self::Rejection> {
        let Some(query) = req.uri().query() else {
            return Ok(ListQuery::default());
        };
        match serde_qs::from_str(query) {
            Ok(query) => Ok(query),
            Err(err) => Err(ApiError::BadRequest(err.to_string())),
        }
    }
}
