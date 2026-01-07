use axum::http::HeaderMap;
use bytes::Bytes;
use reqwest::{Client, Method, RequestBuilder, Response, Url};
use serde::{Serialize, de::DeserializeOwned};

use super::api_errors::{ApiClientError, ApiClientResult};

pub struct ApiClient {
    base_url: Url,
    client: Client,
}

impl ApiClient {
    pub fn new(base_url: &str, headers: HeaderMap) -> ApiClientResult<Self> {
        let client = Client::builder()
            .default_headers(headers)
            .user_agent(format!("tgbot/{}", env!("CARGO_PKG_VERSION")))
            .build()?;

        let base_url = Url::parse(base_url)?;
        Ok(Self { client, base_url })
    }

    pub async fn get<T>(&self, endpoint: &str) -> ApiClientResult<T>
    where
        T: DeserializeOwned + Send + 'static,
    {
        let url = self.base_url.join(endpoint)?;
        let response = self.client.get(url).send().await?;
        Self::parse_response(response).await
    }

    pub async fn get_bytes(&self, endpoint: &str) -> ApiClientResult<Bytes> {
        let url = self.base_url.join(endpoint)?;
        let response = self.client.get(url).send().await?;
        response.bytes().await.map_err(Into::into)
    }

    pub async fn post_with_body<T, B>(&self, endpoint: &str, body: &B) -> ApiClientResult<T>
    where
        T: DeserializeOwned + Send + 'static,
        B: Serialize + ?Sized,
    {
        let url = self.base_url.join(endpoint)?;
        let response = self.client.post(url).json(body).send().await?;
        Self::parse_response(response).await
    }

    pub async fn post<T>(&self, endpoint: &str) -> ApiClientResult<T>
    where
        T: DeserializeOwned + Send + 'static,
    {
        let url = self.base_url.join(endpoint)?;
        let response = self.client.post(url).send().await?;
        Self::parse_response(response).await
    }

    pub async fn put_with_body<T, B>(&self, endpoint: &str, body: &B) -> ApiClientResult<T>
    where
        T: DeserializeOwned + Send + 'static,
        B: Serialize + ?Sized,
    {
        let url = self.base_url.join(endpoint)?;
        let response = self.client.put(url).json(body).send().await?;
        Self::parse_response(response).await
    }

    pub async fn patch_with_body<T, B>(&self, endpoint: &str, body: &B) -> ApiClientResult<T>
    where
        T: DeserializeOwned + Send + 'static,
        B: Serialize + ?Sized,
    {
        let url = self.base_url.join(endpoint)?;
        let response = self.client.patch(url).json(body).send().await?;
        Self::parse_response(response).await
    }

    async fn parse_response<T>(response: Response) -> ApiClientResult<T>
    where
        T: DeserializeOwned,
    {
        let url = response.url().to_string();
        let status = response.status();
        let body = response.text().await?;

        if status.is_success() {
            match serde_json::from_str::<T>(&body) {
                Ok(parsed) => Ok(parsed),
                Err(err) => {
                    tracing::error!(
                        "Failed to parse response from {url} failed with status code: {status}, body: {body}"
                    );
                    Err(ApiClientError::Json(err))
                }
            }
        } else {
            Err(ApiClientError::Unsuccessful(format!(
                "Request to {url} failed with status code: {status}, body: {body}",
            )))
        }
    }

    pub fn request(&self, method: Method, endpoint: &str) -> ApiClientResult<RequestBuilder> {
        let url = self.base_url.join(endpoint)?;
        Ok(self.client.request(method, url))
    }
}
