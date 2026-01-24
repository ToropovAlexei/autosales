use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use reqwest::Response;
use serde::{Serialize, de::DeserializeOwned};
use tokio::sync::RwLock;
use totp_rs::Algorithm;

use crate::infrastructure::external::payment::autosales_platform::dto::{
    AutosalesPlatformAuthStep1Request, AutosalesPlatformAuthStep1Response,
    AutosalesPlatformAuthStep2Request, AutosalesPlatformAuthStep2Response,
    AutosalesPlatformInitializeOrderRequest, AutosalesPlatformObjectTokenPayload,
    AutosalesPlatformOrderInitializedDataRequisite, AutosalesPlatformOrderInitializedResponse,
    AutosalesPlatformOrderStatus, AutosalesPlatformOrderStatusResponse, AutosalesPlatformRequest,
    AutosalesPlatformSendReceiptRequest,
};

pub mod dto;

#[async_trait]
pub trait AutosalesPlatformPaymentsProviderTrait {
    async fn init_order(
        &self,
        req: AutosalesPlatformInitializeOrderRequest,
    ) -> Result<AutosalesPlatformOrderInitializedDataRequisite, String>;
    async fn cancel_order(&self, object_token: String) -> Result<(), String>;
    async fn process_order(&self, object_token: String) -> Result<(), String>;
    async fn send_receipt(&self, req: AutosalesPlatformSendReceiptRequest) -> Result<(), String>;
    async fn get_order_status(
        &self,
        object_token: String,
    ) -> Result<AutosalesPlatformOrderStatus, String>;
}

struct AutosalesToken {
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

pub struct AutosalesPlatformPaymentsProvider {
    url: String,
    client: Arc<reqwest::Client>,
    login: String,
    password: String,
    two_fa: String,
    token: Arc<RwLock<Option<AutosalesToken>>>,
}

impl AutosalesPlatformPaymentsProvider {
    pub fn new(
        client: Arc<reqwest::Client>,
        url: String,
        login: String,
        password: String,
        two_fa: String,
    ) -> AutosalesPlatformPaymentsProvider {
        AutosalesPlatformPaymentsProvider {
            client,
            url,
            token: Arc::new(RwLock::new(None)),
            login,
            password,
            two_fa,
        }
    }

    async fn request<T, B>(&self, endpoint: &str, payload: B, version: i64) -> Result<T, String>
    where
        T: DeserializeOwned + Send + 'static,
        B: Serialize + Sized,
    {
        let user_token = self.get_token().await?;
        self.post::<T, AutosalesPlatformRequest<B>>(
            endpoint,
            &AutosalesPlatformRequest {
                user_token,
                version: version.to_string(),
                payload,
            },
        )
        .await
    }

    async fn post<T, B>(&self, endpoint: &str, payload: &B) -> Result<T, String>
    where
        T: DeserializeOwned + Send + 'static,
        B: Serialize + ?Sized,
    {
        let response = self
            .client
            .post(format!("{}/{endpoint}", self.url))
            .form(payload)
            .send()
            .await
            .map_err(|e| format!("[Autosales platform payments provider] {e}"))?;
        Self::parse_response(response).await
    }

    async fn parse_response<T>(response: Response) -> Result<T, String>
    where
        T: DeserializeOwned,
    {
        let url = response.url().to_string();
        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| format!("[Autosales platform payments provider] {e}"))?;

        if status.is_success() {
            match serde_json::from_str::<T>(&body) {
                Ok(parsed) => Ok(parsed),
                Err(err) => Err(format!(
                    "[Autosales platform payments provider] Failed to parse response from {url} failed with status code: {status}, body: {body}, error: {err}"
                )),
            }
        } else {
            Err(format!(
                "[Autosales platform payments provider] Request to {url} failed with status code: {status}, body: {body}",
            ))
        }
    }

    async fn auth_step_1(&self) -> Result<AutosalesPlatformAuthStep1Response, String> {
        self.post::<AutosalesPlatformAuthStep1Response, AutosalesPlatformAuthStep1Request>(
            "api/method/client/auth/step1",
            &AutosalesPlatformAuthStep1Request {
                version: 1,
                login: self.login.clone(),
                password: self.password.clone(),
            },
        )
        .await
    }

    fn generate_2fa_code(&self) -> Result<String, String> {
        let totp = totp_rs::TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            totp_rs::Secret::Encoded(self.two_fa.clone())
                .to_bytes()
                .map_err(|e| e.to_string())?,
            None,
            "".to_string(),
        )
        .map_err(|e| e.to_string())?;
        totp.generate_current().map_err(|e| e.to_string())
    }

    async fn auth_step_2(
        &self,
        temp_token: &str,
    ) -> Result<AutosalesPlatformAuthStep2Response, String> {
        self.post::<AutosalesPlatformAuthStep2Response, AutosalesPlatformAuthStep2Request>(
            "api/method/client/auth/step2",
            &AutosalesPlatformAuthStep2Request {
                version: 1,
                temp: temp_token.to_string(),
                key: self.generate_2fa_code()?,
            },
        )
        .await
    }

    async fn authenticate(&self) -> Result<String, String> {
        let temp_token = self.auth_step_1().await?;
        let token = self
            .auth_step_2(&temp_token.data.temp)
            .await
            .map(|r| r.data.token)?;
        let mut guard = self.token.write().await;
        *guard = Some(AutosalesToken {
            token: token.clone(),
            expires_at: Utc::now() + Duration::days(2), // Token lives 3 days since last request
        });
        Ok(token)
    }

    async fn get_token(&self) -> Result<String, String> {
        // Inner scope to prevent deadlock
        {
            let guard = self.token.read().await;
            if let Some(token) = guard.as_ref()
                && token.expires_at > Utc::now()
            {
                return Ok(token.token.clone());
            }
        }

        self.authenticate().await
    }
}

#[async_trait]
impl AutosalesPlatformPaymentsProviderTrait for AutosalesPlatformPaymentsProvider {
    async fn init_order(
        &self,
        req: AutosalesPlatformInitializeOrderRequest,
    ) -> Result<AutosalesPlatformOrderInitializedDataRequisite, String> {
        self.request::<AutosalesPlatformOrderInitializedResponse, AutosalesPlatformInitializeOrderRequest>(
            "api/method/merch/payin/order_initialized/standart",
            req,
            3,
        )
        .await.map(|r| r.data.data_requisite)
    }

    async fn cancel_order(&self, object_token: String) -> Result<(), String> {
        self.request::<(), AutosalesPlatformObjectTokenPayload>(
            "api/method/merch/payin/order_cancel",
            AutosalesPlatformObjectTokenPayload { object_token },
            1,
        )
        .await
    }

    async fn process_order(&self, object_token: String) -> Result<(), String> {
        self.request::<(), AutosalesPlatformObjectTokenPayload>(
            "api/method/merch/payin/order_process",
            AutosalesPlatformObjectTokenPayload { object_token },
            1,
        )
        .await
    }

    async fn send_receipt(&self, req: AutosalesPlatformSendReceiptRequest) -> Result<(), String> {
        self.request("api/method/merch/payin/order_check_down", req, 2)
            .await
    }

    async fn get_order_status(
        &self,
        object_token: String,
    ) -> Result<AutosalesPlatformOrderStatus, String> {
        self.request::<AutosalesPlatformOrderStatusResponse, AutosalesPlatformObjectTokenPayload>(
            "api/method/merch/payin/order_get_status",
            AutosalesPlatformObjectTokenPayload { object_token },
            1,
        )
        .await
        .map(|r| r.data.status)
    }
}
