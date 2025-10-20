use reqwest::header;

use crate::{api::api_client::ApiClient, errors::AppResult, models::CaptchaResponse};

pub struct CaptchaApi {
    api_client: ApiClient,
}

impl CaptchaApi {
    pub fn new(base_url: &str, api_key: &str) -> AppResult<Self> {
        let mut headers = header::HeaderMap::new();
        headers.insert("X-API-KEY", header::HeaderValue::from_str(api_key)?);
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );
        let api_client = ApiClient::new(base_url, headers)?;
        Ok(Self { api_client })
    }

    pub async fn get_captcha(&self) -> AppResult<CaptchaResponse> {
        self.api_client.get::<CaptchaResponse>("captcha").await
    }
}
