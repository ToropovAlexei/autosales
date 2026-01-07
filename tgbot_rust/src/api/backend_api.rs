use std::collections::HashMap;

use axum::http::{HeaderMap, HeaderValue};
use bytes::Bytes;
use reqwest::header;
use serde_json::json;
use teloxide::types::MessageId;

use crate::{
    api::{
        api_client::ApiClient,
        api_errors::{ApiClientError, ApiClientResult},
    },
    bot::BotUsername,
    models::{
        BackendResponse, BalanceResponse, BuyResponse, Category, InvoiceResponse, PaymentGateway,
        Product, UserOrder, UserSubscription, common::CaptchaResponse, user::BotUser,
    },
};

pub struct BackendApi {
    api_client: ApiClient,
}

impl BackendApi {
    pub fn new(base_url: &str, api_key: &str, bot_id: Option<i64>) -> ApiClientResult<Self> {
        let mut headers = HeaderMap::new();
        headers.insert("X-API-KEY", HeaderValue::from_str(api_key)?);
        if let Some(bot_id) = bot_id {
            headers.insert("X-BOT-ID", HeaderValue::from_str(&bot_id.to_string())?);
        };
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        let api_client = ApiClient::new(base_url, headers)?;
        Ok(Self { api_client })
    }

    pub async fn register_user(
        &self,
        telegram_id: i64,
        bot_username: &str,
    ) -> ApiClientResult<BotUser> {
        self.api_client
            .post_with_body::<BackendResponse<BotUser>, _>(
                "users/register",
                &json!({"telegram_id": telegram_id, "bot_name": bot_username}),
            )
            .await
            .and_then(|res| {
                res.data.ok_or_else(|| {
                    ApiClientError::Unsuccessful(
                        res.error.unwrap_or_else(|| "Unknown error".to_string()),
                    )
                })
            })
    }

    pub async fn get_user(&self, telegram_id: i64, bot_username: &str) -> ApiClientResult<BotUser> {
        self.api_client
            .get::<BackendResponse<BotUser>>(&format!(
                "users/{telegram_id}?bot_name={bot_username}"
            ))
            .await
            .and_then(|res| {
                res.data.ok_or_else(|| {
                    ApiClientError::Unsuccessful(
                        res.error.unwrap_or_else(|| "Unknown error".to_string()),
                    )
                })
            })
    }

    pub async fn get_user_balance(
        &self,
        telegram_id: i64,
        bot_username: &str,
    ) -> ApiClientResult<f64> {
        self.api_client
            .get::<BackendResponse<BalanceResponse>>(&format!(
                "users/{telegram_id}/balance?bot_name={bot_username}"
            ))
            .await
            .and_then(|res| {
                res.data.ok_or_else(|| {
                    ApiClientError::Unsuccessful(
                        res.error.unwrap_or_else(|| "Unknown error".to_string()),
                    )
                })
            })
            .map(|res| res.balance)
    }

    pub async fn confirm_user_captcha(
        &self,
        telegram_id: i64,
    ) -> ApiClientResult<serde_json::Value> {
        self.api_client
            .put_with_body::<BackendResponse<serde_json::Value>, _>(
                &format!("users/{telegram_id}/captcha-status"),
                &json!({"has_passed_captcha": true}),
            )
            .await
            .and_then(|res| {
                res.data.ok_or_else(|| {
                    ApiClientError::Unsuccessful(
                        res.error.unwrap_or_else(|| "Unknown error".to_string()),
                    )
                })
            })
    }

    pub async fn get_settings(&self) -> ApiClientResult<HashMap<String, String>> {
        let res = self
            .api_client
            .get::<BackendResponse<serde_json::Value>>("settings/public")
            .await?;

        let data = res.data.ok_or_else(|| {
            ApiClientError::Unsuccessful(res.error.unwrap_or_else(|| "Unknown error".to_string()))
        })?;

        let obj = data.as_object().ok_or_else(|| {
            ApiClientError::Unsuccessful(
                "Invalid settings format: expected JSON object".to_string(),
            )
        })?;

        let settings = obj
            .iter()
            .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
            .collect::<HashMap<_, _>>();

        Ok(settings)
    }

    pub async fn is_referral_program_enabled(&self) -> bool {
        self.get_settings()
            .await
            .ok()
            .and_then(|settings| settings.get("referral_program_enabled").cloned())
            .map(|v| v == "true")
            .unwrap_or(false)
    }

    pub async fn get_support_msg(&self) -> Option<String> {
        self.get_settings()
            .await
            .ok()
            .and_then(|settings| settings.get("support_message").cloned())
            .map(|val| val.to_string())
    }

    pub async fn get_new_user_welcome_msg(&self) -> Option<String> {
        self.get_settings()
            .await
            .ok()
            .and_then(|settings| settings.get("new_user_welcome_message").cloned())
            .map(|val| val.to_string())
    }

    pub async fn get_returning_user_welcome_msg(&self) -> Option<String> {
        self.get_settings()
            .await
            .ok()
            .and_then(|settings| settings.get("returning_user_welcome_message").cloned())
            .map(|val| val.to_string())
    }

    pub async fn get_payment_gateways(&self) -> Vec<PaymentGateway> {
        self.api_client
            .get::<BackendResponse<Vec<PaymentGateway>>>("gateways")
            .await
            .map(|res| res.data)
            .unwrap_or_default()
            .unwrap_or_default()
    }

    pub async fn create_deposit_invoice(
        &self,
        gateway_name: &str,
        amount: f64,
        telegram_id: i64,
    ) -> ApiClientResult<InvoiceResponse> {
        self.api_client.post_with_body::<BackendResponse<InvoiceResponse>, _>(
            "deposit/invoice",
            &json!({"telegram_id": telegram_id, "gateway_name": gateway_name, "amount": amount}),
        )
        .await
        .and_then(|res| {
            res.data.ok_or_else(|| {
                ApiClientError::Unsuccessful(res.error.unwrap_or_else(|| "Unknown error".to_string()))
            })
        })
    }

    pub async fn set_invoice_message_id(
        &self,
        order_id: &str,
        message_id: MessageId,
    ) -> ApiClientResult<serde_json::Value> {
        self.api_client
            .patch_with_body::<BackendResponse<serde_json::Value>, _>(
                &format!("invoices/{order_id}/message-id"),
                &json!({"message_id": message_id.0}),
            )
            .await
            .and_then(|res| {
                res.data.ok_or_else(|| {
                    ApiClientError::Unsuccessful(
                        res.error.unwrap_or_else(|| "Unknown error".to_string()),
                    )
                })
            })
    }

    pub async fn get_user_orders(&self, telegram_id: i64) -> ApiClientResult<Vec<UserOrder>> {
        self.api_client
            .get::<BackendResponse<Vec<UserOrder>>>(&format!("users/{telegram_id}/orders"))
            .await
            .and_then(|res| {
                res.data.ok_or_else(|| {
                    ApiClientError::Unsuccessful(
                        res.error.unwrap_or_else(|| "Unknown error".to_string()),
                    )
                })
            })
    }

    pub async fn get_user_subscriptions(
        &self,
        telegram_id: i64,
    ) -> ApiClientResult<Vec<UserSubscription>> {
        self.api_client
            .get::<BackendResponse<Vec<UserSubscription>>>(&format!(
                "users/{telegram_id}/subscriptions"
            ))
            .await
            .and_then(|res| {
                res.data.ok_or_else(|| {
                    ApiClientError::Unsuccessful(
                        res.error.unwrap_or_else(|| "Unknown error".to_string()),
                    )
                })
            })
    }

    pub async fn get_categories(&self) -> ApiClientResult<Vec<Category>> {
        self.api_client
            .get::<BackendResponse<Vec<Category>>>("categories")
            .await
            .and_then(|res| {
                res.data.ok_or_else(|| {
                    ApiClientError::Unsuccessful(
                        res.error.unwrap_or_else(|| "Unknown error".to_string()),
                    )
                })
            })
    }

    pub async fn get_products(&self, category_id: i64) -> ApiClientResult<Vec<Product>> {
        self.api_client
            .get::<BackendResponse<Vec<Product>>>(&format!(
                "bot/products?category_id={category_id}"
            ))
            .await
            .and_then(|res| {
                res.data.ok_or_else(|| {
                    ApiClientError::Unsuccessful(
                        res.error.unwrap_or_else(|| "Unknown error".to_string()),
                    )
                })
            })
    }

    pub async fn get_product(&self, product_id: i64) -> ApiClientResult<Product> {
        self.api_client
            .get::<BackendResponse<Product>>(&format!("bot/products/{product_id}"))
            .await
            .and_then(|res| {
                res.data.ok_or_else(|| {
                    ApiClientError::Unsuccessful(
                        res.error.unwrap_or_else(|| "Unknown error".to_string()),
                    )
                })
            })
    }

    pub async fn get_image_bytes(&self, id: &str) -> ApiClientResult<Bytes> {
        self.api_client.get_bytes(&format!("images/{id}")).await
    }

    pub async fn buy_product(
        &self,
        telegram_id: i64,
        product_id: i64,
        bot_username: BotUsername,
    ) -> ApiClientResult<BuyResponse> {
        self.api_client
            .post_with_body::<BackendResponse<BuyResponse>, _>(
                &format!("buy/product?bot_name={bot_username}"),
                &json!({
                    "telegram_id": telegram_id,
                    "product_id": product_id,
                }),
            )
            .await
            .and_then(|res| {
                res.data.ok_or_else(|| {
                    ApiClientError::Unsuccessful(
                        res.error.unwrap_or_else(|| "Unknown error".to_string()),
                    )
                })
            })
    }

    pub async fn get_bots(&self, bot_type: &str) -> ApiClientResult<Vec<crate::models::Bot>> {
        self.api_client
            .get::<BackendResponse<Vec<crate::models::Bot>>>(&format!("bots?type={bot_type}"))
            .await
            .and_then(|res| {
                res.data.ok_or_else(|| {
                    ApiClientError::Unsuccessful(
                        res.error.unwrap_or_else(|| "Unknown error".to_string()),
                    )
                })
            })
    }

    pub async fn create_referral_bot(
        &self,
        telegram_id: i64,
        bot_token: &str,
    ) -> ApiClientResult<crate::models::Bot> {
        self.api_client
            .post_with_body::<BackendResponse<crate::models::Bot>, _>(
                "referrals",
                &json!({"owner_id": telegram_id, "bot_token": bot_token}),
            )
            .await
            .and_then(|res| {
                res.data.ok_or_else(|| {
                    ApiClientError::Unsuccessful(
                        res.error.unwrap_or_else(|| "Unknown error".to_string()),
                    )
                })
            })
    }

    pub async fn create_main_bot(
        &self,
        token: &str,
        username: &str,
    ) -> ApiClientResult<crate::models::Bot> {
        self.api_client
            .post_with_body::<BackendResponse<crate::models::Bot>, _>(
                "bots/main",
                &json!({"token": token, "username": username}),
            )
            .await
            .and_then(|res| {
                res.data.ok_or_else(|| {
                    ApiClientError::Unsuccessful(
                        res.error.unwrap_or_else(|| "Unknown error".to_string()),
                    )
                })
            })
    }

    pub async fn get_captcha(&self) -> ApiClientResult<CaptchaResponse> {
        self.api_client
            .get::<BackendResponse<CaptchaResponse>>("captcha")
            .await
            .and_then(|res| {
                res.data.ok_or_else(|| {
                    ApiClientError::Unsuccessful(
                        res.error.unwrap_or_else(|| "Unknown error".to_string()),
                    )
                })
            })
    }
}
