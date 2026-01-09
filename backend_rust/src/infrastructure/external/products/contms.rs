pub mod dto;

use std::sync::Arc;

use async_trait::async_trait;
use chrono::Duration;
use reqwest::Response;
use serde::de::DeserializeOwned;

use crate::infrastructure::external::products::contms::dto::{
    ContmsAvailableResponse, ContmsProxyResponse, ContmsRenewResponse, ContmsRequestAction,
    ContmsStatusProxyResponse, ContmsStatusResponse, ContmsUpResponse, ContmsUserResponse,
    UpProxyRequest,
};

#[async_trait]
pub trait ContmsProductsProviderTrait {
    async fn get_products(&self) -> Result<Vec<ContmsProxyResponse>, String>;
    async fn subscribe_to_proxy(
        &self,
        product_name: &str,
        duration: Duration,
    ) -> Result<ContmsUserResponse, String>;
    async fn renew_subscription(
        &self,
        subscription_id: &str,
        duration: Duration,
    ) -> Result<ContmsStatusProxyResponse, String>;
    async fn unsubscribe_from_proxy(&self, subscription_id: &str) -> Result<(), String>;
}

pub struct ContmsProductsProvider {
    pub client: Arc<reqwest::Client>,
    pub url: String,
}

impl ContmsProductsProvider {
    pub fn new(client: Arc<reqwest::Client>, url: String) -> ContmsProductsProvider {
        ContmsProductsProvider { client, url }
    }

    async fn request<T>(&self, payload: &ContmsRequestAction) -> Result<T, String>
    where
        T: DeserializeOwned + Send + 'static,
    {
        let response = self
            .client
            .post(&self.url)
            .json(payload)
            .send()
            .await
            .map_err(|e| format!("Contms: {e}"))?;
        Self::parse_response(response).await
    }

    async fn parse_response<T>(response: Response) -> Result<T, String>
    where
        T: DeserializeOwned,
    {
        let url = response.url().to_string();
        let status = response.status();
        let body = response.text().await.map_err(|e| format!("Contms: {e}"))?;

        if status.is_success() {
            match serde_json::from_str::<T>(&body) {
                Ok(parsed) => Ok(parsed),
                Err(err) => Err(format!(
                    "Contms: Failed to parse response from {url} failed with status code: {status}, body: {body}, error: {err}"
                )),
            }
        } else {
            Err(format!(
                "Contms: Request to {url} failed with status code: {status}, body: {body}",
            ))
        }
    }
}

#[async_trait]
impl ContmsProductsProviderTrait for ContmsProductsProvider {
    async fn get_products(&self) -> Result<Vec<ContmsProxyResponse>, String> {
        self.request::<ContmsAvailableResponse>(&ContmsRequestAction::Available)
            .await
            .map(|resp| resp.proxy)
    }

    async fn subscribe_to_proxy(
        &self,
        product_name: &str,
        duration: Duration,
    ) -> Result<ContmsUserResponse, String> {
        self.request::<ContmsUpResponse>(&ContmsRequestAction::Up {
            proxy: UpProxyRequest {
                expires: duration,
                name: product_name.to_string(),
            },
        })
        .await
        .map(|resp| resp.user)
    }

    async fn renew_subscription(
        &self,
        subscription_id: &str,
        duration: Duration,
    ) -> Result<ContmsStatusProxyResponse, String> {
        self.request::<ContmsRenewResponse>(&ContmsRequestAction::Renew {
            user: subscription_id.to_string(),
            expires: duration,
        })
        .await?;
        self.request::<ContmsStatusResponse>(&ContmsRequestAction::Status {
            user: subscription_id.to_string(),
        })
        .await
        .map(|r| r.proxy)
    }

    async fn unsubscribe_from_proxy(&self, subscription_id: &str) -> Result<(), String> {
        self.request::<ContmsUpResponse>(&ContmsRequestAction::Down {
            user: subscription_id.to_string(),
        })
        .await?;
        Ok(())
    }
}
