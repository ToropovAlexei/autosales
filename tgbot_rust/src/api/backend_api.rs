use axum::http::{HeaderMap, HeaderValue};
use bytes::Bytes;
use reqwest::{header, multipart};
use serde_json::json;
use shared_dtos::{
    bot::BotBotResponse,
    captcha::CaptchaBotResponse,
    category::CategoryBotResponse,
    customer::{CustomerBotResponse, UpdateCustomerBotRequest},
    invoice::{GatewayBotResponse, PaymentInvoiceBotResponse, PaymentSystem},
    list_response::ListResponse,
    order::{EnrichedOrderBotResponse, PurchaseBotResponse},
    product::ProductBotResponse,
    settings::SettingsBotResponse,
    user_subscription::UserSubscriptionBotResponse,
};
use uuid::Uuid;

use crate::api::{api_client::ApiClient, api_errors::ApiClientResult};

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

    pub async fn register_user(&self, telegram_id: i64) -> ApiClientResult<CustomerBotResponse> {
        self.api_client
            .post_with_body::<CustomerBotResponse, _>(
                "bot/customers",
                &json!({"telegram_id": telegram_id}),
            )
            .await
    }

    pub async fn get_user(&self, telegram_id: i64) -> ApiClientResult<CustomerBotResponse> {
        self.api_client
            .get::<CustomerBotResponse>(&format!("bot/customers/{telegram_id}"))
            .await
    }

    pub async fn ensure_user(&self, telegram_id: i64) -> ApiClientResult<CustomerBotResponse> {
        let user = self.get_user(telegram_id).await;
        if user.is_err() {
            self.register_user(telegram_id).await
        } else {
            user
        }
    }

    pub async fn confirm_user_captcha(
        &self,
        telegram_id: i64,
    ) -> ApiClientResult<CustomerBotResponse> {
        self.api_client
            .patch_with_body::<CustomerBotResponse, _>(
                &format!("bot/customers/{telegram_id}"),
                &json!({"has_passed_captcha": true}),
            )
            .await
    }

    pub async fn get_settings(&self) -> ApiClientResult<SettingsBotResponse> {
        let res = self
            .api_client
            .get::<SettingsBotResponse>("bot/settings")
            .await?;
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

    pub async fn get_payment_gateways(&self) -> ApiClientResult<ListResponse<GatewayBotResponse>> {
        self.api_client
            .get::<ListResponse<GatewayBotResponse>>("bot/gateways")
            .await
    }

    pub async fn get_customer_invoices(
        &self,
        telegram_id: i64,
    ) -> ApiClientResult<ListResponse<PaymentInvoiceBotResponse>> {
        self.api_client
            .get::<ListResponse<PaymentInvoiceBotResponse>>(&format!(
                "bot/customers/{telegram_id}/invoices"
            ))
            .await
    }

    pub async fn get_invoice(&self, id: i64) -> ApiClientResult<PaymentInvoiceBotResponse> {
        self.api_client
            .get::<PaymentInvoiceBotResponse>(&format!("bot/invoices/{id}"))
            .await
    }

    pub async fn cancel_invoice(&self, id: i64) -> ApiClientResult<PaymentInvoiceBotResponse> {
        self.api_client
            .post::<PaymentInvoiceBotResponse>(&format!("bot/invoices/{id}/cancel"))
            .await
    }

    pub async fn confirm_invoice(&self, id: i64) -> ApiClientResult<PaymentInvoiceBotResponse> {
        self.api_client
            .post::<PaymentInvoiceBotResponse>(&format!("bot/invoices/{id}/confirm"))
            .await
    }

    pub async fn create_deposit_invoice(
        &self,
        gateway: &PaymentSystem,
        amount: f64,
        telegram_id: i64,
    ) -> ApiClientResult<PaymentInvoiceBotResponse> {
        self.api_client
            .post_with_body::<PaymentInvoiceBotResponse, _>(
                "bot/invoices",
                &json!({"telegram_id": telegram_id, "gateway": gateway, "amount": amount}),
            )
            .await
    }

    pub async fn get_user_orders(
        &self,
        telegram_id: i64,
    ) -> ApiClientResult<ListResponse<EnrichedOrderBotResponse>> {
        self.api_client
            .get::<ListResponse<EnrichedOrderBotResponse>>(&format!(
                "bot/customers/{telegram_id}/orders"
            ))
            .await
    }

    pub async fn get_order(&self, id: i64) -> ApiClientResult<EnrichedOrderBotResponse> {
        self.api_client
            .get::<EnrichedOrderBotResponse>(&format!("bot/orders/{id}"))
            .await
    }

    pub async fn get_user_subscriptions(
        &self,
        telegram_id: i64,
    ) -> ApiClientResult<ListResponse<UserSubscriptionBotResponse>> {
        self.api_client
            .get::<ListResponse<UserSubscriptionBotResponse>>(&format!(
                "bot/customers/{telegram_id}/subscriptions"
            ))
            .await
    }

    pub async fn get_categories(&self) -> ApiClientResult<ListResponse<CategoryBotResponse>> {
        self.api_client
            .get::<ListResponse<CategoryBotResponse>>("bot/categories")
            .await
    }

    pub async fn get_products(
        &self,
        category_id: i64,
    ) -> ApiClientResult<ListResponse<ProductBotResponse>> {
        self.api_client
            .get_with_qs::<ListResponse<ProductBotResponse>>(
                "bot/products",
                &[
                    ("filters[0][op]", "eq"),
                    ("filters[0][field]", "category_id"),
                    ("filters[0][value]", &category_id.to_string()),
                ],
            )
            .await
    }

    pub async fn get_product(&self, product_id: i64) -> ApiClientResult<ProductBotResponse> {
        self.api_client
            .get::<ProductBotResponse>(&format!("bot/products/{product_id}"))
            .await
    }

    pub async fn get_image_bytes(&self, id: &Uuid) -> ApiClientResult<Bytes> {
        self.api_client.get_bytes(&format!("images/{id}")).await
    }

    pub async fn buy_product(
        &self,
        telegram_id: i64,
        product_id: i64,
    ) -> ApiClientResult<PurchaseBotResponse> {
        self.api_client
            .post_with_body::<PurchaseBotResponse, _>(
                "bot/orders",
                &json!({
                    "telegram_id": telegram_id,
                    "product_id": product_id,
                }),
            )
            .await
    }

    pub async fn get_bots(&self) -> ApiClientResult<ListResponse<BotBotResponse>> {
        self.api_client
            // TODO Filters
            .get::<ListResponse<BotBotResponse>>("bot/bots")
            .await
    }

    pub async fn get_primary_bots(&self) -> ApiClientResult<ListResponse<BotBotResponse>> {
        self.api_client
            .get::<ListResponse<BotBotResponse>>("bot/bots/primary")
            .await
    }

    pub async fn create_referral_bot(
        &self,
        telegram_id: i64,
        token: &str,
    ) -> ApiClientResult<BotBotResponse> {
        self.api_client
            .post_with_body::<BotBotResponse, _>(
                "bot/bots",
                &json!({"owner_id": telegram_id, "token": token}),
            )
            .await
    }

    pub async fn create_main_bot(&self, token: &str) -> ApiClientResult<BotBotResponse> {
        self.api_client
            .post_with_body::<BotBotResponse, _>("bot/bots", &json!({"token": token}))
            .await
    }

    pub async fn get_captcha(&self) -> ApiClientResult<CaptchaBotResponse> {
        self.api_client
            .get::<CaptchaBotResponse>("bot/captcha")
            .await
    }

    pub async fn update_customer_last_seen(&self, telegram_id: i64) -> ApiClientResult<()> {
        self.api_client
            .post(&format!("bot/customers/{telegram_id}/update-last-seen"))
            .await
    }

    pub async fn submit_payment_receipt_file(
        &self,
        invoice_id: i64,
        file_bytes: Bytes,
    ) -> ApiClientResult<PaymentInvoiceBotResponse> {
        let form = multipart::Form::new().part("file", multipart::Part::bytes(file_bytes.to_vec()));
        self.api_client
            .post_with_multipart::<PaymentInvoiceBotResponse>(
                &format!("bot/invoices/{invoice_id}/send-receipt"),
                form,
            )
            .await
    }

    pub async fn update_customer(
        &self,
        telegram_id: i64,
        update: &UpdateCustomerBotRequest,
    ) -> ApiClientResult<CustomerBotResponse> {
        self.api_client
            .patch_with_body::<CustomerBotResponse, _>(
                &format!("bot/customers/{telegram_id}"),
                update,
            )
            .await
    }
}
