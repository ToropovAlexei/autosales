use axum::extract::{FromRequest, Json, Request};
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use validator::Validate;

use crate::errors::api::ApiError;

#[derive(Debug, Default)]
pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(ApiError::from)?;
        value.validate()?;
        Ok(ValidatedJson(value))
    }
}
