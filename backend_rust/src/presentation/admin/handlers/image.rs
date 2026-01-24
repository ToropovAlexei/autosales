use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, post},
};
use axum_extra::extract::Multipart;
use bytes::Bytes;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    errors::api::{ApiError, ApiResult},
    middlewares::require_permission::{ImagesCreate, ImagesDelete, ImagesRead, RequirePermission},
    models::image::ImageListQuery,
    presentation::admin::dtos::{image::ImageResponse, list_response::ListResponse},
    services::{
        auth::AuthUser,
        image::{CreateImage, ImageServiceTrait},
    },
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(create_image).get(list_images))
        .route("/{id}", delete(delete_image))
}

#[utoipa::path(
    post,
    path = "/api/admin/images",
    tag = "Images",
    responses(
        (status = 200, description = "Image created", body = ImageResponse),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn create_image(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    _perm: RequirePermission<ImagesCreate>,
    mut multipart: Multipart,
) -> ApiResult<Json<ImageResponse>> {
    let image = state
        .image_service
        .create(
            parse_image_form(&mut multipart, user.id)
                .await
                .map_err(|e| ApiError::BadRequest(e.to_string()))?,
        )
        .await?;

    Ok(Json(image.into()))
}

#[utoipa::path(
    get,
    path = "/api/admin/images",
    tag = "Images",
    responses(
        (status = 200, description = "List of images", body = Vec<ImageResponse>),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn list_images(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    _perm: RequirePermission<ImagesRead>,
    query: ImageListQuery,
) -> ApiResult<Json<ListResponse<ImageResponse>>> {
    let categories = state.image_service.get_list(&query).await?;

    Ok(Json(ListResponse {
        total: categories.total,
        items: categories
            .items
            .into_iter()
            .map(ImageResponse::from)
            .collect(),
    }))
}

#[utoipa::path(
    delete,
    path = "/api/admin/images/{id}",
    tag = "Images",
    responses(
        (status = 204, description = "Image deleted"),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn delete_image(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    _user: AuthUser,
    _perm: RequirePermission<ImagesDelete>,
) -> ApiResult<StatusCode> {
    state.image_service.delete(id).await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn parse_image_form(
    multipart: &mut Multipart,
    created_by: i64,
) -> Result<CreateImage, Box<dyn std::error::Error>> {
    let mut context: Option<String> = None;
    let mut file: Option<Bytes> = None;
    let mut filename: Option<String> = None;

    while let Some(field) = multipart.next_field().await? {
        let name = field.name().ok_or("Field name missing")?.to_string();
        match name.as_str() {
            "context" => {
                let value = field.text().await?;
                context = Some(value);
            }
            "file" => {
                filename = field.file_name().map(|s| s.to_string());
                let data = field.bytes().await?;
                file = Some(data);
            }
            _ => {}
        }
    }

    Ok(CreateImage {
        context: context.ok_or("Missing 'context' field")?,
        file: file.ok_or("Missing 'file' field")?,
        filename: filename.ok_or("Missing 'filename' field")?,
        created_by,
    })
}
