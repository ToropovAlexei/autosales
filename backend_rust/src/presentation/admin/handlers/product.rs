use axum::http::StatusCode;
use rust_decimal::{Decimal, prelude::FromPrimitive};
use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};

use crate::{
    errors::api::{ApiError, ApiResult},
    middlewares::{
        context::RequestContext,
        require_permission::{
            ProductsCreate, ProductsDelete, ProductsRead, ProductsUpdate, RequirePermission,
        },
        validator::ValidatedJson,
    },
    models::product::ProductListQuery,
    presentation::admin::dtos::{
        list_response::ListResponse,
        product::{NewProductRequest, ProductResponse, UpdateProductRequest},
    },
    services::{
        auth::AuthUser,
        product::{
            CreateProductCommand, DeleteProductCommand, ProductServiceTrait, UpdateProductCommand,
        },
    },
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(create_product).get(list_products))
        .route(
            "/{id}",
            get(get_product)
                .patch(update_product)
                .delete(delete_product),
        )
}

#[utoipa::path(
    post,
    path = "/api/admin/products",
    tag = "Products",
    request_body = NewProductRequest,
    responses(
        (status = 201, description = "Product created", body = ProductResponse),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn create_product(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    _perm: RequirePermission<ProductsCreate>,
    ctx: RequestContext,
    ValidatedJson(payload): ValidatedJson<NewProductRequest>,
) -> ApiResult<Json<ProductResponse>> {
    let product = state
        .product_service
        .create(
            CreateProductCommand {
                category_id: payload.category_id,
                created_by: user.id,
                details: payload.details,
                external_id: None,
                fulfillment_image_id: payload.fulfillment_image_id,
                fulfillment_text: payload.fulfillment_text,
                image_id: payload.image_id,
                name: payload.name,
                base_price: Decimal::from_f64(payload.base_price)
                    .ok_or_else(|| ApiError::BadRequest("invalid price".into()))?,
                provider_name: "internal".to_string(),
                subscription_period_days: payload.subscription_period_days,
                r#type: payload.r#type,
                initial_stock: payload.initial_stock,
            },
            ctx,
        )
        .await?;

    Ok(Json(ProductResponse::from(product)))
}

#[utoipa::path(
    get,
    path = "/api/admin/products",
    tag = "Products",
    responses(
        (status = 200, description = "Products list", body = ListResponse<ProductResponse>),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn list_products(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    _perm: RequirePermission<ProductsRead>,
    query: ProductListQuery,
) -> ApiResult<Json<ListResponse<ProductResponse>>> {
    let products = state.product_service.get_list(query).await?;

    Ok(Json(ListResponse {
        total: products.total,
        items: products
            .items
            .into_iter()
            .map(ProductResponse::from)
            .collect(),
    }))
}

#[utoipa::path(
    get,
    path = "/api/admin/products/{id}",
    tag = "Products",
    responses(
        (status = 200, description = "Product details", body = ProductResponse),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn get_product(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    _user: AuthUser,
    _perm: RequirePermission<ProductsRead>,
) -> ApiResult<Json<ProductResponse>> {
    let product = state.product_service.get_by_id(id).await?;

    Ok(Json(ProductResponse::from(product)))
}

#[utoipa::path(
    patch,
    path = "/api/admin/products/{id}",
    tag = "Products",
    request_body = UpdateProductRequest,
    responses(
        (status = 200, description = "Product updated", body = ProductResponse),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn update_product(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    user: AuthUser,
    _perm: RequirePermission<ProductsUpdate>,
    ctx: RequestContext,
    ValidatedJson(payload): ValidatedJson<UpdateProductRequest>,
) -> ApiResult<Json<ProductResponse>> {
    let product = state
        .product_service
        .update(
            UpdateProductCommand {
                id,
                category_id: payload.category_id,
                details: payload.details,
                external_id: payload.external_id,
                fulfillment_image_id: payload.fulfillment_image_id,
                fulfillment_text: payload.fulfillment_text,
                image_id: payload.image_id,
                name: payload.name,
                base_price: payload
                    .base_price
                    .map(|base_price| {
                        Decimal::from_f64(base_price)
                            .ok_or_else(|| ApiError::BadRequest("invalid price".into()))
                    })
                    .transpose()?,
                subscription_period_days: payload.subscription_period_days,
                r#type: payload.r#type,
                stock: payload.stock,
                updated_by: user.id,
            },
            ctx,
        )
        .await?;

    Ok(Json(ProductResponse::from(product)))
}

#[utoipa::path(
    delete,
    path = "/api/admin/products/{id}",
    tag = "Products",
    responses(
        (status = 204, description = "Product deleted"),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn delete_product(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    user: AuthUser,
    ctx: RequestContext,
    _perm: RequirePermission<ProductsDelete>,
) -> ApiResult<StatusCode> {
    state
        .product_service
        .delete(
            DeleteProductCommand {
                id,
                deleted_by: user.id,
            },
            ctx,
        )
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
