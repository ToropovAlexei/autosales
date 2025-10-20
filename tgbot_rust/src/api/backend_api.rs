use reqwest::header;
use serde_json::json;

use crate::{
    api::api_client::ApiClient,
    errors::{AppError, AppResult},
    models::{BackendResponse, user::BotUser},
};

pub struct BackendApi {
    api_client: ApiClient,
}

impl BackendApi {
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

    pub async fn register_user(&self, telegram_id: i64, bot_username: &str) -> AppResult<BotUser> {
        self.api_client
            .post_with_body::<BackendResponse<BotUser>, _>(
                "users/register",
                &json!({"telegram_id": telegram_id, "bot_name": bot_username}),
            )
            .await
            .and_then(|res| {
                res.data.ok_or_else(|| {
                    AppError::BadRequest(res.error.unwrap_or_else(|| "Unknown error".to_string()))
                })
            })
    }

    pub async fn get_user(&self, telegram_id: i64, bot_username: &str) -> AppResult<BotUser> {
        self.api_client
            .get::<BackendResponse<BotUser>>(&format!(
                "users/{telegram_id}?bot_name={bot_username}"
            ))
            .await
            .and_then(|res| {
                res.data.ok_or_else(|| {
                    AppError::BadRequest(res.error.unwrap_or_else(|| "Unknown error".to_string()))
                })
            })
    }

    pub async fn confirm_user_captcha(&self, telegram_id: i64) -> AppResult<serde_json::Value> {
        self.api_client
            .put_with_body::<BackendResponse<serde_json::Value>, _>(
                &format!("users/{telegram_id}/captcha-status"),
                &json!({"has_passed_captcha": true}),
            )
            .await
            .and_then(|res| {
                res.data.ok_or_else(|| {
                    AppError::BadRequest(res.error.unwrap_or_else(|| "Unknown error".to_string()))
                })
            })
    }

    pub async fn get_settings(&self) -> AppResult<serde_json::Value> {
        self.api_client
            .get::<BackendResponse<serde_json::Value>>("settings/public")
            .await
            .and_then(|res| {
                res.data.ok_or_else(|| {
                    AppError::BadRequest(res.error.unwrap_or_else(|| "Unknown error".to_string()))
                })
            })
    }
}
