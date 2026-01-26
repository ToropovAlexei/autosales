use axum::{
    body::Body,
    http::{HeaderMap, Response, header},
    response::IntoResponse,
};
use std::sync::Arc;
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use uuid::Uuid;

use axum::{
    Router,
    extract::{Path, State},
    routing::get,
};

use crate::{
    errors::api::{ApiError, ApiResult},
    services::image::{ImageServiceTrait, get_image_path},
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/{id}", get(get_image))
}

#[utoipa::path(
    get,
    path = "/api/images/{id}",
    tag = "Images",
    responses(
        (status = 200, description = "Image file"),
        (status = 404, description = "Image not found", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
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
