use std::sync::Arc;

use async_trait::async_trait;
use reqwest::Response;
use serde::{Serialize, de::DeserializeOwned};
use uuid::Uuid;

use crate::infrastructure::external::payment::mock::dto::{
    MockProviderCreateInvoiceRequest, MockProviderCreateInvoiceResponse, MockProviderInvoiceStatus,
    MockProviderInvoiceWebhookPayload,
};

pub mod dto;

#[async_trait]
pub trait MockPaymentsProviderTrait {
    async fn create_invoide(
        &self,
        req: MockProviderCreateInvoiceRequest,
    ) -> Result<MockProviderCreateInvoiceResponse, String>;
    async fn get_invoice_status(
        &self,
        invoice_id: Uuid,
    ) -> Result<MockProviderInvoiceStatus, String>;
    async fn handle_webhook(&self, req: MockProviderInvoiceWebhookPayload) -> Result<Uuid, String>;
}

pub struct MockPaymentsProvider {
    pub url: String,
    pub client: Arc<reqwest::Client>,
}

impl MockPaymentsProvider {
    pub fn new(client: Arc<reqwest::Client>, url: String) -> MockPaymentsProvider {
        MockPaymentsProvider { client, url }
    }

    async fn get<T>(&self, endpoint: &str) -> Result<T, String>
    where
        T: DeserializeOwned + Send + 'static,
    {
        let response = self
            .client
            .get(format!("{}/{endpoint}", self.url))
            .send()
            .await
            .map_err(|e| format!("Mock payments provider: {e}"))?;
        Self::parse_response(response).await
    }

    async fn post<T, B>(&self, endpoint: &str, payload: &B) -> Result<T, String>
    where
        T: DeserializeOwned + Send + 'static,
        B: Serialize + ?Sized,
    {
        let response = self
            .client
            .post(format!("{}/{endpoint}", self.url))
            .json(payload)
            .send()
            .await
            .map_err(|e| format!("Mock payments provider: {e}"))?;
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
            .map_err(|e| format!("Mock payments provider: {e}"))?;

        if status.is_success() {
            match serde_json::from_str::<T>(&body) {
                Ok(parsed) => Ok(parsed),
                Err(err) => Err(format!(
                    "Mock payments provider: Failed to parse response from {url} failed with status code: {status}, body: {body}, error: {err}"
                )),
            }
        } else {
            Err(format!(
                "Mock payments provider: Request to {url} failed with status code: {status}, body: {body}",
            ))
        }
    }
}

#[async_trait]
impl MockPaymentsProviderTrait for MockPaymentsProvider {
    async fn create_invoide(
        &self,
        req: MockProviderCreateInvoiceRequest,
    ) -> Result<MockProviderCreateInvoiceResponse, String> {
        self.post("create_invoice", &req).await
    }

    async fn get_invoice_status(
        &self,
        invoice_id: Uuid,
    ) -> Result<MockProviderInvoiceStatus, String> {
        self.get(&format!("status/{invoice_id}")).await
    }

    async fn handle_webhook(&self, req: MockProviderInvoiceWebhookPayload) -> Result<Uuid, String> {
        if req.status == MockProviderInvoiceStatus::Completed {
            Ok(req.order_id)
        } else {
            Err("Mock payments provider: received unexpected event type".to_string())
        }
    }
}
