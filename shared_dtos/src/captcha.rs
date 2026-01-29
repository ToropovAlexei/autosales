use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Serialize, Deserialize)]
pub struct CaptchaBotResponse {
    pub answer: String,
    pub variants: Vec<String>,
    pub image_data: String,
}
