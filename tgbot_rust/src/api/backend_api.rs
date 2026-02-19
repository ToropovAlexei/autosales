use axum::http::{HeaderMap, HeaderValue};
use bytes::Bytes;
use reqwest::{header, multipart};
use serde_json::json;
use shared_dtos::{
    analytics::BotAnalyticsBotResponse,
    balance_request::{CompleteStoreBalanceRequestBotRequest, RejectStoreBalanceRequestBotRequest},
    bot::{BotBotResponse, NewBotBotRequest, UpdateBotBotRequest},
    captcha::CaptchaBotResponse,
    category::CategoryBotResponse,
    customer::{CustomerBotResponse, NewCustomerBotRequest, UpdateCustomerBotRequest},
    invoice::{
        GatewayBotResponse, NewPaymentInvoiceBotRequest, PaymentInvoiceBotResponse, PaymentSystem,
    },
    list_query::{FilterValue, Operator, RawFilter, RawListQuery, ScalarValue},
    list_response::ListResponse,
    order::{EnrichedOrderBotResponse, PurchaseBotRequest, PurchaseBotResponse},
    product::ProductBotResponse,
    settings::{SettingsBotResponse, UpdateBotManagedSettingsBotRequest},
    user_subscription::UserSubscriptionBotResponse,
};
use uuid::Uuid;

use crate::api::{
    api_client::ApiClient,
    api_errors::{ApiClientError, ApiClientResult},
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

    pub async fn register_user(&self, telegram_id: i64) -> ApiClientResult<CustomerBotResponse> {
        self.api_client
            .post_with_body::<CustomerBotResponse, _>(
                "bot/customers",
                &NewCustomerBotRequest { telegram_id },
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
                &UpdateCustomerBotRequest {
                    has_passed_captcha: Some(true),
                    ..Default::default()
                },
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

    pub async fn update_manager_group_chat_id(
        &self,
        chat_id: i64,
    ) -> ApiClientResult<SettingsBotResponse> {
        self.api_client
            .patch_with_body::<SettingsBotResponse, _>(
                "bot/settings",
                &UpdateBotManagedSettingsBotRequest {
                    manager_group_chat_id: Some(Some(chat_id)),
                },
            )
            .await
    }

    pub async fn complete_store_balance_request(
        &self,
        request_id: i64,
        tg_user_id: i64,
    ) -> ApiClientResult<()> {
        self.api_client
            .post_with_body::<(), _>(
                &format!("bot/store-balance/{request_id}/complete"),
                &CompleteStoreBalanceRequestBotRequest { tg_user_id },
            )
            .await
    }

    pub async fn reject_store_balance_request(
        &self,
        request_id: i64,
        tg_user_id: i64,
    ) -> ApiClientResult<()> {
        self.api_client
            .post_with_body::<(), _>(
                &format!("bot/store-balance/{request_id}/reject"),
                &RejectStoreBalanceRequestBotRequest { tg_user_id },
            )
            .await
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
                &NewPaymentInvoiceBotRequest {
                    gateway: *gateway,
                    amount,
                    telegram_id,
                },
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
                &PurchaseBotRequest {
                    telegram_id,
                    product_id,
                },
            )
            .await
    }

    pub async fn get_bots(
        &self,
        query: RawListQuery,
    ) -> ApiClientResult<ListResponse<BotBotResponse>> {
        let qs =
            serde_qs::to_string(&query).map_err(|e| ApiClientError::Unsuccessful(e.to_string()))?;
        self.api_client
            .get::<ListResponse<BotBotResponse>>(&format!("bot/bots?{qs}"))
            .await
    }

    pub async fn get_customer_bots(
        &self,
        telegram_id: i64,
    ) -> ApiClientResult<ListResponse<BotBotResponse>> {
        let customer_id = self.ensure_user(telegram_id).await?.id;
        self.get_bots(RawListQuery {
            filters: vec![RawFilter {
                value: FilterValue::Scalar(ScalarValue::Int(customer_id)),
                field: "owner_id".to_string(),
                op: Operator::Eq,
            }],
            order_by: Some("id".to_string()),
            ..Default::default()
        })
        .await
    }

    pub async fn get_bot(&self, id: i64) -> ApiClientResult<BotBotResponse> {
        self.api_client.get(&format!("bot/bots/{id}")).await
    }

    pub async fn delete_bot(&self, id: i64) -> ApiClientResult<()> {
        self.api_client
            .delete::<()>(&format!("bot/bots/{id}"))
            .await
    }

    pub async fn update_bot(
        &self,
        id: i64,
        update: UpdateBotBotRequest,
    ) -> ApiClientResult<BotBotResponse> {
        self.api_client
            .patch_with_body::<BotBotResponse, _>(&format!("bot/bots/{id}"), &update)
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
                &NewBotBotRequest {
                    owner_id: telegram_id,
                    token: token.to_string(),
                },
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
        file_name: Option<String>,
        content_type: Option<&str>,
    ) -> ApiClientResult<PaymentInvoiceBotResponse> {
        let part = multipart::Part::bytes(file_bytes.to_vec())
            .file_name(file_name.unwrap_or("receipt.pdf".to_string()))
            .mime_str(content_type.unwrap_or("application/pdf"))?;

        let form = multipart::Form::new().part("file", part);
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

    pub async fn get_referral_stats(
        &self,
        telegram_id: i64,
    ) -> ApiClientResult<ListResponse<BotAnalyticsBotResponse>> {
        self.api_client
            .get(&format!("bot/customers/{telegram_id}/referral-analytics"))
            .await
    }
}
