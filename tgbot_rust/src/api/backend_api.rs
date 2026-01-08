use axum::http::{HeaderMap, HeaderValue};
use bytes::Bytes;
use reqwest::header;
use serde_json::json;
use teloxide::types::MessageId;

use crate::{
    api::{api_client::ApiClient, api_errors::ApiClientResult},
    models::{
        Bot, BuyResponse, Category, InvoiceResponse, ListResponse, PaymentGateway, Product,
        UserOrder, UserSubscription, common::CaptchaResponse, settings::Settings, user::Customer,
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

    pub async fn register_user(&self, telegram_id: i64) -> ApiClientResult<Customer> {
        self.api_client
            .post_with_body::<Customer, _>("bot/customers", &json!({"telegram_id": telegram_id}))
            .await
    }

    pub async fn get_user(&self, telegram_id: i64) -> ApiClientResult<Customer> {
        self.api_client
            .get::<Customer>(&format!("bot/customers/{telegram_id}"))
            .await
    }

    pub async fn confirm_user_captcha(&self, telegram_id: i64) -> ApiClientResult<Customer> {
        self.api_client
            .patch_with_body::<Customer, _>(
                &format!("bot/customers/{telegram_id}"),
                &json!({"has_passed_captcha": true}),
            )
            .await
    }

    pub async fn get_settings(&self) -> ApiClientResult<Settings> {
        let res = self.api_client.get::<Settings>("bot/settings").await?;
        Ok(res)
    }

    pub async fn is_referral_program_enabled(&self) -> bool {
        self.get_settings()
            .await
            .ok()
            .map(|settings| settings.referral_program_enabled)
            .unwrap_or(false)
    }

    pub async fn get_support_msg(&self) -> Option<String> {
        self.get_settings()
            .await
            .ok()
            .map(|settings| settings.bot_messages_support)
    }

    pub async fn get_new_user_welcome_msg(&self) -> Option<String> {
        self.get_settings()
            .await
            .ok()
            .map(|settings| settings.bot_messages_new_user_welcome)
    }

    pub async fn get_returning_user_welcome_msg(&self) -> Option<String> {
        self.get_settings()
            .await
            .ok()
            .map(|settings| settings.bot_messages_returning_user_welcome)
    }

    // TODO Unused
    pub async fn get_payment_gateways(&self) -> ListResponse<PaymentGateway> {
        self.api_client
            .get::<ListResponse<PaymentGateway>>("gateways")
            .await
            .unwrap_or(ListResponse {
                items: Vec::new(),
                total: 0,
            })
    }

    pub async fn create_deposit_invoice(
        &self,
        gateway_name: &str,
        amount: f64,
        telegram_id: i64,
    ) -> ApiClientResult<InvoiceResponse> {
        self.api_client.post_with_body::<InvoiceResponse, _>(
            "deposit/invoice",
            &json!({"telegram_id": telegram_id, "gateway_name": gateway_name, "amount": amount}),
        )
        .await
    }

    pub async fn set_invoice_message_id(
        &self,
        order_id: &str,
        message_id: MessageId,
    ) -> ApiClientResult<serde_json::Value> {
        self.api_client
            .patch_with_body::<serde_json::Value, _>(
                &format!("invoices/{order_id}/message-id"),
                &json!({"message_id": message_id.0}),
            )
            .await
    }

    pub async fn get_user_orders(
        &self,
        telegram_id: i64,
    ) -> ApiClientResult<ListResponse<UserOrder>> {
        self.api_client
            .get::<ListResponse<UserOrder>>(&format!("bot/customers/{telegram_id}/orders"))
            .await
    }

    pub async fn get_user_subscriptions(
        &self,
        telegram_id: i64,
    ) -> ApiClientResult<ListResponse<UserSubscription>> {
        self.api_client
            .get::<ListResponse<UserSubscription>>(&format!(
                "bot/customers/{telegram_id}/subscriptions"
            ))
            .await
    }

    pub async fn get_categories(&self) -> ApiClientResult<ListResponse<Category>> {
        self.api_client
            .get::<ListResponse<Category>>("bot/categories")
            .await
    }

    pub async fn get_products(&self, category_id: i64) -> ApiClientResult<ListResponse<Product>> {
        self.api_client
            .get::<ListResponse<Product>>(&format!("bot/products?category_id={category_id}"))
            .await
    }

    pub async fn get_product(&self, product_id: i64) -> ApiClientResult<Product> {
        self.api_client
            .get::<Product>(&format!("bot/products/{product_id}"))
            .await
    }

    pub async fn get_image_bytes(&self, id: &str) -> ApiClientResult<Bytes> {
        self.api_client.get_bytes(&format!("images/{id}")).await
    }

    pub async fn buy_product(
        &self,
        telegram_id: i64,
        product_id: i64,
    ) -> ApiClientResult<BuyResponse> {
        self.api_client
            .post_with_body::<BuyResponse, _>(
                "buy/product",
                &json!({
                    "telegram_id": telegram_id,
                    "product_id": product_id,
                }),
            )
            .await
    }

    pub async fn get_bots(&self) -> ApiClientResult<ListResponse<Bot>> {
        self.api_client
            // TODO Filters
            .get::<ListResponse<Bot>>("bot/bots")
            .await
    }

    pub async fn create_referral_bot(&self, telegram_id: i64, token: &str) -> ApiClientResult<Bot> {
        self.api_client
            .post_with_body::<Bot, _>(
                "bot/bots",
                &json!({"owner_id": telegram_id, "token": token}),
            )
            .await
    }

    pub async fn create_main_bot(&self, token: &str) -> ApiClientResult<Bot> {
        self.api_client
            .post_with_body::<Bot, _>("bot/bots", &json!({"token": token}))
            .await
    }

    pub async fn get_captcha(&self) -> ApiClientResult<CaptchaResponse> {
        self.api_client.get::<CaptchaResponse>("bot/captcha").await
    }
}
