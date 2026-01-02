use axum::{
    body::Body,
    http::{HeaderMap, Response, StatusCode, header},
    response::IntoResponse,
};
use axum_extra::extract::Multipart;
use bytes::Bytes;
use std::sync::Arc;
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use uuid::Uuid;

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};

use crate::{
    errors::api::{ApiError, ApiResult},
    middlewares::require_permission::{ImagesCreate, ImagesDelete, ImagesRead, RequirePermission},
    models::common::ListQuery,
    presentation::admin::dtos::{image::ImageResponse, list_response::ListResponse},
    services::{
        auth::AuthUser,
        image::{CreateImage, ImageServiceTrait, get_image_path},
    },
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(create_image).get(list_images))
        .route("/{id}", get(get_image).delete(delete_image))
}

async fn get_image(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> ApiResult<Response<Body>> {
    let image = state.image_service.get_by_id(id).await?;
    let path = get_image_path(
        &image.hash,
        std::path::Path::new(&state.config.image_upload_path),
    );

    if !path.exists() {
        return Err(ApiError::NotFound("Image not found".to_string()));
    }

    let file = File::open(&path).await.map_err(|e| {
        tracing::error!("Failed to open image file {:?}: {}", path, e);
        ApiError::InternalServerError("Failed to read image".into())
    })?;

    let mut headers = HeaderMap::new();

    headers.insert(
        header::CONTENT_TYPE,
        image
            .mime_type
            .parse()
            .map_err(|_| ApiError::InternalServerError("Failed to set content type".to_string()))?,
    );

    headers.insert(
        header::ETAG,
        format!("\"{}\"", image.hash)
            .parse()
            .map_err(|_| ApiError::InternalServerError("Failed to set etag".to_string()))?,
    );

    headers.insert(
        header::CACHE_CONTROL,
        "public, max-age=86400".parse().map_err(|_| {
            ApiError::InternalServerError("Failed to set cache control".to_string())
        })?,
    );

    let filename = image
        .original_filename
        .as_ref()
        .and_then(|s| (!s.is_empty()).then(|| s.clone()))
        .unwrap_or_else(|| format!("image-{}.bin", &image.hash[0..8]));

    let ext = match image.mime_type.as_str() {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/webp" => "webp",
        "image/gif" => "gif",
        "image/svg+xml" => "svg",
        _ => "bin",
    };
    let filename = format!("{}.{}", filename.trim_end_matches(".bin"), ext);

    headers.insert(
        header::CONTENT_DISPOSITION,
        format!("inline; filename=\"{}\"", filename)
            .parse()
            .map_err(|_| {
                ApiError::InternalServerError("Failed to set content disposition".to_string())
            })?,
    );

    let stream = ReaderStream::new(file);
    let body = axum::body::Body::from_stream(stream);

    Ok((headers, body).into_response())
}

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

async fn list_images(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    _perm: RequirePermission<ImagesRead>,
    query: ListQuery,
) -> ApiResult<Json<ListResponse<ImageResponse>>> {
    let categories = state.image_service.get_list(query).await?;

    Ok(Json(ListResponse {
        total: categories.total,
        items: categories
            .items
            .into_iter()
            .map(ImageResponse::from)
            .collect(),
    }))
}

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
    let mut content_type: Option<String> = None;

    while let Some(field) = multipart.next_field().await? {
        let name = field.name().ok_or("Field name missing")?.to_string();
        match name.as_str() {
            "context" => {
                let value = field.text().await?;
                context = Some(value);
            }
            "file" => {
                filename = field.file_name().map(|s| s.to_string());
                content_type = field.content_type().map(|s| s.to_string());
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
        content_type: content_type.ok_or("Missing 'content_type' field")?,
        created_by,
    })
}
