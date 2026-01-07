use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct CaptchaResponse {
    pub answer: String,
    pub variants: Vec<String>,
    pub image_data: String,
}
