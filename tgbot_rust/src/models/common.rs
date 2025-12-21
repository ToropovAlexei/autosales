use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CaptchaResponse {
    pub image_data: String,
    pub answer: String,
    pub variants: Vec<String>,
}
