use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::post};
use serde::Serialize;

use crate::{
    errors::api::{ApiError, ApiResult},
    services::auth::AuthUser,
    state::AppState,
};

#[derive(Serialize)]
pub struct ResetTestDataResponse {
    pub ok: bool,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/reset-data", post(reset_test_data))
}

// TODO Dont forget to remove this endpoint!
async fn reset_test_data(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
) -> ApiResult<Json<ResetTestDataResponse>> {
    let mut tx = state
        .db
        .get_pool()
        .begin()
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    sqlx::query("DELETE FROM user_subscriptions")
        .execute(&mut *tx)
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
    sqlx::query("DELETE FROM stock_movements")
        .execute(&mut *tx)
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
    sqlx::query("DELETE FROM order_items")
        .execute(&mut *tx)
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
    sqlx::query("DELETE FROM payment_invoices")
        .execute(&mut *tx)
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
    sqlx::query("DELETE FROM transactions")
        .execute(&mut *tx)
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
    sqlx::query("DELETE FROM orders")
        .execute(&mut *tx)
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
    sqlx::query("DELETE FROM broadcasts")
        .execute(&mut *tx)
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
    sqlx::query("DELETE FROM products")
        .execute(&mut *tx)
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
    sqlx::query("DELETE FROM categories")
        .execute(&mut *tx)
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
    sqlx::query("DELETE FROM images")
        .execute(&mut *tx)
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
    sqlx::query("DELETE FROM customers")
        .execute(&mut *tx)
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
    sqlx::query("DELETE FROM audit_logs")
        .execute(&mut *tx)
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    tx.commit()
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    Ok(Json(ResetTestDataResponse { ok: true }))
}
