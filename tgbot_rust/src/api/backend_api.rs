use axum::http::{HeaderMap, HeaderValue};
use bytes::Bytes;
use reqwest::header;
use serde_json::json;
use uuid::Uuid;

use crate::{
    api::{api_client::ApiClient, api_errors::ApiClientResult},
    models::{
        InvoiceResponse, ListResponse,
        bot::Bot,
        category::Category,
        common::CaptchaResponse,
        customer::Customer,
        order::OrderResponse,
        payment::{PaymentGateway, PaymentInvoiceResponse, PaymentSystem},
        product::Product,
        purchase::PurchaseResponse,
        settings::Settings,
        user_subscription::UserSubscription,
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

    pub async fn ensure_user(&self, telegram_id: i64) -> ApiClientResult<Customer> {
        let user = self.get_user(telegram_id).await;
        if user.is_err() {
            self.register_user(telegram_id).await
        } else {
            user
        }
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

    pub async fn get_new_user_welcome_msg(&self) -> ApiClientResult<String> {
        self.get_settings()
            .await
            .map(|settings| settings.bot_messages_new_user_welcome)
    }

    pub async fn get_returning_user_welcome_msg(&self) -> Option<String> {
        self.get_settings()
            .await
            .ok()
            .map(|settings| settings.bot_messages_returning_user_welcome)
    }

    pub async fn get_payment_gateways(&self) -> ApiClientResult<ListResponse<PaymentGateway>> {
        self.api_client
            .get::<ListResponse<PaymentGateway>>("bot/gateways")
            .await
    }

    pub async fn get_customer_invoices(
        &self,
        telegram_id: i64,
    ) -> ApiClientResult<ListResponse<PaymentInvoiceResponse>> {
        self.api_client
            .get::<ListResponse<PaymentInvoiceResponse>>(&format!(
                "bot/customers/{telegram_id}/invoices"
            ))
            .await
    }

    pub async fn get_invoice(&self, id: i64) -> ApiClientResult<PaymentInvoiceResponse> {
        self.api_client
            .get::<PaymentInvoiceResponse>(&format!("bot/invoices/{id}"))
            .await
    }

    pub async fn cancel_invoice(&self, id: i64) -> ApiClientResult<PaymentInvoiceResponse> {
        self.api_client
            .post::<PaymentInvoiceResponse>(&format!("bot/invoices/{id}/cancel"))
            .await
    }

    pub async fn confirm_invoice(&self, id: i64) -> ApiClientResult<PaymentInvoiceResponse> {
        self.api_client
            .post::<PaymentInvoiceResponse>(&format!("bot/invoices/{id}/confirm"))
            .await
    }

    pub async fn create_deposit_invoice(
        &self,
        gateway: &PaymentSystem,
        amount: f64,
        telegram_id: i64,
    ) -> ApiClientResult<InvoiceResponse> {
        self.api_client
            .post_with_body::<InvoiceResponse, _>(
                "bot/invoices",
                &json!({"telegram_id": telegram_id, "gateway": gateway, "amount": amount}),
            )
            .await
    }

    pub async fn get_user_orders(
        &self,
        telegram_id: i64,
    ) -> ApiClientResult<ListResponse<OrderResponse>> {
        self.api_client
            .get::<ListResponse<OrderResponse>>(&format!("bot/customers/{telegram_id}/orders"))
            .await
    }

    pub async fn get_order(&self, id: i64) -> ApiClientResult<OrderResponse> {
        self.api_client
            .get::<OrderResponse>(&format!("bot/orders/{id}"))
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
            .get_with_qs::<ListResponse<Product>>(
                "bot/products",
                &[
                    ("filters[0][op]", "eq"),
                    ("filters[0][field]", "category_id"),
                    ("filters[0][value]", &category_id.to_string()),
                ],
            )
            .await
    }

    pub async fn get_product(&self, product_id: i64) -> ApiClientResult<Product> {
        self.api_client
            .get::<Product>(&format!("bot/products/{product_id}"))
            .await
    }

    pub async fn get_image_bytes(&self, id: &Uuid) -> ApiClientResult<Bytes> {
        self.api_client.get_bytes(&format!("images/{id}")).await
    }

    pub async fn buy_product(
        &self,
        telegram_id: i64,
        product_id: i64,
    ) -> ApiClientResult<PurchaseResponse> {
        self.api_client
            .post_with_body::<PurchaseResponse, _>(
                "bot/orders",
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

    pub async fn get_primary_bots(&self) -> ApiClientResult<ListResponse<Bot>> {
        self.api_client
            .get::<ListResponse<Bot>>("bot/bots/primary")
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

    pub async fn update_customer_last_seen(&self, telegram_id: i64) -> ApiClientResult<()> {
        self.api_client
            .post(&format!("bot/customers/{telegram_id}/update-last-seen"))
            .await
    }
}
