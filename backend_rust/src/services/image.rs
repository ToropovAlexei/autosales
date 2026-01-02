use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use bytes::Bytes;
use image::{ImageFormat, ImageReader};
use tokio::fs;
use uuid::Uuid;

use crate::{
    errors::api::{ApiError, ApiResult},
    infrastructure::repositories::image::{ImageRepository, ImageRepositoryTrait},
    models::{
        common::PaginatedResult,
        image::{ImageListQuery, ImageRow, NewImage},
    },
};

#[derive(Debug)]
pub struct ImageMetadata {
    pub hash: String,
    pub mime_type: String,
    pub file_size: u64,
    pub width: u32,
    pub height: u32,
    pub original_filename: Option<String>,
}

#[async_trait]
pub trait ImageServiceTrait: Send + Sync {
    async fn get_list(&self, query: &ImageListQuery) -> ApiResult<PaginatedResult<ImageRow>>;
    async fn create(&self, image: CreateImage) -> ApiResult<ImageRow>;
    async fn delete(&self, id: Uuid) -> ApiResult<()>;
    async fn get_by_id(&self, id: Uuid) -> ApiResult<ImageRow>;
}

pub struct ImageService<R> {
    repo: Arc<R>,
    upload_path: String,
}

impl<R> ImageService<R>
where
    R: ImageRepositoryTrait + Send + Sync,
{
    pub fn new(repo: Arc<R>, upload_path: String) -> Self {
        Self { repo, upload_path }
    }
}

#[derive(Debug)]
pub struct CreateImage {
    pub context: String,
    pub file: Bytes,
    pub filename: String,
    pub content_type: String,
    pub created_by: i64,
}

#[async_trait]
impl ImageServiceTrait for ImageService<ImageRepository> {
    async fn get_list(&self, query: &ImageListQuery) -> ApiResult<PaginatedResult<ImageRow>> {
        self.repo.get_list(query).await.map_err(ApiError::from)
    }

    async fn get_by_id(&self, id: Uuid) -> ApiResult<ImageRow> {
        let image = self.repo.get_by_id(id).await?;
        Ok(image)
    }

    async fn create(&self, image: CreateImage) -> ApiResult<ImageRow> {
        let meta = extract_image_metadata(&image.file, Some(&image.filename))
            .map_err(|e| ApiError::BadRequest(e.to_string()))?;
        let created = self
            .repo
            .create(NewImage {
                context: image.context,
                hash: meta.hash.clone(),
                mime_type: meta.mime_type,
                file_size: meta.file_size as i64,
                width: meta.width as i16,
                height: meta.height as i16,
                original_filename: meta.original_filename,
                created_by: image.created_by,
            })
            .await?;

        save_image_to_disk(Path::new(&self.upload_path), &image.file, &meta.hash)
            .await
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

        Ok(created)
    }

    async fn delete(&self, id: Uuid) -> ApiResult<()> {
        Ok(self.repo.delete(id).await?)
    }
}

pub fn extract_image_metadata(
    data: &[u8],
    original_filename: Option<&str>,
) -> Result<ImageMetadata, Box<dyn std::error::Error>> {
    let hash = {
        let mut hasher = blake3::Hasher::new();
        hasher.update(data);
        hasher.finalize().to_string()
    };

    let file_size = data.len() as u64;

    let mime_type = infer::get(data)
        .map(|t| t.mime_type())
        .ok_or("Unknown image format")?
        .to_string();

    if !matches!(
        mime_type.as_str(),
        "image/jpeg" | "image/png" | "image/webp" | "image/gif"
    ) {
        return Err("Unsupported image type".into());
    }

    let (width, height) = extract_raster_dimensions(data, &mime_type)?;

    Ok(ImageMetadata {
        hash,
        mime_type,
        file_size,
        width,
        height,
        original_filename: original_filename.map(|s| s.to_string()),
    })
}

fn extract_raster_dimensions(
    data: &[u8],
    mime_type: &str,
) -> Result<(u32, u32), Box<dyn std::error::Error>> {
    let format = match mime_type {
        "image/jpeg" => ImageFormat::Jpeg,
        "image/png" => ImageFormat::Png,
        "image/webp" => ImageFormat::WebP,
        "image/gif" => ImageFormat::Gif,
        _ => return Err("Unsupported raster format".into()),
    };
    let reader = ImageReader::with_format(std::io::Cursor::new(data), format).decode()?;
    Ok((reader.width(), reader.height()))
}

pub fn get_image_dir(hash: &str, upload_path: &Path) -> PathBuf {
    let dir = &hash[0..2];
    upload_path.join(dir)
}

pub fn get_image_path(hash: &str, upload_path: &Path) -> PathBuf {
    get_image_dir(hash, upload_path).join(hash)
}

pub async fn save_image_to_disk(
    upload_path: &Path,
    data: &[u8],
    hash: &str,
) -> Result<PathBuf, std::io::Error> {
    let dir_path = get_image_dir(hash, upload_path);
    let file_path = get_image_path(hash, upload_path);

    fs::create_dir_all(&dir_path).await?;
    fs::write(&file_path, data).await?;

    Ok(file_path)
}
