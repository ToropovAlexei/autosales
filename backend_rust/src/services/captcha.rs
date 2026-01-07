use std::sync::Arc;

use async_trait::async_trait;
use reqwest::Response;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::errors::api::{ApiError, ApiResult};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Captcha {
    pub answer: String,
    pub variants: Vec<String>,
    pub image_data: String,
}

#[async_trait]
pub trait CaptchaServiceTrait: Send + Sync {
    async fn get_captcha(&self) -> ApiResult<Captcha>;
}

pub struct CaptchaService {
    client: Arc<reqwest::Client>,
    captcha_url: String,
}

impl CaptchaService {
    pub fn new(client: Arc<reqwest::Client>, captcha_url: String) -> Self {
        Self {
            client,
            captcha_url,
        }
    }
}

#[async_trait]
impl CaptchaServiceTrait for CaptchaService {
    async fn get_captcha(&self) -> ApiResult<Captcha> {
        parse_response::<Captcha>(
            self.client
                .get(self.captcha_url.clone())
                .send()
                .await
                .map_err(|e| ApiError::InternalServerError(e.to_string()))?,
        )
        .await
        .ok_or_else(|| ApiError::InternalServerError("Error getting captcha".to_string()))
    }
}

async fn parse_response<T>(response: Response) -> Option<T>
where
    T: DeserializeOwned,
{
    let status = response.status();
    if let Ok(body) = response.text().await
        && status.is_success()
        && let Ok(parsed) = serde_json::from_str::<T>(&body)
    {
        return Some(parsed);
    }
    None
}
