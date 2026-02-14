use axum::http::StatusCode;
use axum_extra::extract::Multipart;
use bytes::Bytes;
use rust_decimal::{Decimal, prelude::FromPrimitive};
use shared_dtos::list_response::ListResponse;
use std::sync::Arc;
use utoipa::ToSchema;

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};

use crate::{
    errors::api::{ApiError, ApiResult, ErrorResponse},
    middlewares::{
        context::RequestContext,
        require_permission::{
            ProductsCreate, ProductsDelete, ProductsRead, ProductsUpdate, RequirePermission,
        },
        validator::ValidatedJson,
    },
    models::product::ProductListQuery,
    presentation::admin::dtos::product::{
        NewProductRequest, ProductResponse, ProductsUploadResponse, UpdateProductRequest,
        UploadedProductCSV,
    },
    services::{
        auth::AuthUser,
        product::{
            CreateProductCommand, DeleteProductCommand, ProductServiceTrait, UpdateProductCommand,
            UploadProductsCommand,
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
        .route("/upload", post(upload_products))
}

#[derive(ToSchema)]
#[allow(dead_code)] // Used by utoipa request_body schema only.
struct UploadProductsMultipartRequest {
    #[schema(value_type = String, format = Binary)]
    file: String,
}

#[utoipa::path(
    post,
    path = "/api/admin/products",
    tag = "Products",
    request_body = NewProductRequest,
    responses(
        (status = 200, description = "Product created", body = ProductResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
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
        .create(CreateProductCommand {
            category_id: payload.category_id,
            created_by: user.id,
            details: None,
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
            ctx: Some(ctx),
        })
        .await?;

    Ok(Json(ProductResponse::from(product)))
}

#[utoipa::path(
    post,
    path = "/api/admin/products/upload",
    tag = "Products",
    request_body(content = UploadProductsMultipartRequest, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Products uploaded", body = ProductsUploadResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    )
)]
async fn upload_products(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    _perm: RequirePermission<ProductsCreate>,
    ctx: RequestContext,
    mut multipart: Multipart,
) -> ApiResult<Json<ProductsUploadResponse>> {
    let (products_to_create, parsing_errors) = parse_upload_products_form(&mut multipart)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;
    let total_rows = (products_to_create.len() + parsing_errors.len()) as i64;

    let (created_products, creation_errors) = state
        .product_service
        .upload_products(UploadProductsCommand {
            products: products_to_create,
            created_by: user.id,
            ctx: Some(ctx),
        })
        .await?;

    let failed = (parsing_errors.len() + creation_errors.len()) as i64;
    let created = created_products.len() as i64;
    let skipped = total_rows - created - failed;
    let errors = parsing_errors.into_iter().chain(creation_errors).collect();

    Ok(Json(ProductsUploadResponse {
        created,
        failed,
        errors,
        skipped,
    }))
}

#[utoipa::path(
    get,
    path = "/api/admin/products",
    tag = "Products",
    responses(
        (status = 200, description = "Products list", body = ListResponse<ProductResponse>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
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
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
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
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
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
                details: None,
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
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
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
        .delete(DeleteProductCommand {
            id,
            deleted_by: user.id,
            ctx: Some(ctx),
        })
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn parse_upload_products_form(
    multipart: &mut Multipart,
) -> Result<(Vec<UploadedProductCSV>, Vec<String>), Box<dyn std::error::Error>> {
    let mut file: Option<Bytes> = None;

    while let Some(field) = multipart.next_field().await? {
        let name = field.name().ok_or("Field name missing")?.to_string();
        if name.as_str() == "file" {
            let data = field.bytes().await?;
            file = Some(data);
        }
    }

    let file = file.ok_or("Missing 'file' field")?;

    let mut errors = Vec::new();

    let records = csv::Reader::from_reader(file.as_ref())
        .deserialize::<UploadedProductCSV>()
        .map(|r| {
            r.map_err(|e| {
                errors.push(e.to_string());
            })
        })
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    Ok((records, errors))
}
