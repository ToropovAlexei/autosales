use axum::http::HeaderMap;
use reqwest::{Client, Method, RequestBuilder, Response, Url};
use serde::{Serialize, de::DeserializeOwned};

use crate::errors::{AppError, AppResult};

pub struct ApiClient {
    base_url: Url,
    client: Client,
}

impl ApiClient {
    pub fn new(base_url: &str, headers: HeaderMap) -> AppResult<Self> {
        let client = Client::builder()
            .default_headers(headers)
            .user_agent(format!("tgbot/{}", env!("CARGO_PKG_VERSION")))
            .build()?;

        let base_url = Url::parse(base_url)?;
        Ok(Self { client, base_url })
    }

    pub async fn get<T>(&self, endpoint: &str) -> AppResult<T>
    where
        T: DeserializeOwned + Send + 'static,
    {
        let url = self.base_url.join(endpoint)?;
        let response = self.client.get(url).send().await?;
        Self::parse_response(response).await
    }

    pub async fn post_with_body<T, B>(&self, endpoint: &str, body: &B) -> AppResult<T>
    where
        T: DeserializeOwned + Send + 'static,
        B: Serialize + ?Sized,
    {
        let url = self.base_url.join(endpoint)?;
        let response = self.client.post(url).json(body).send().await?;
        Self::parse_response(response).await
    }

    pub async fn post<T>(&self, endpoint: &str) -> AppResult<T>
    where
        T: DeserializeOwned + Send + 'static,
    {
        let url = self.base_url.join(endpoint)?;
        let response = self.client.post(url).send().await?;
        Self::parse_response(response).await
    }

    pub async fn put_with_body<T, B>(&self, endpoint: &str, body: &B) -> AppResult<T>
    where
        T: DeserializeOwned + Send + 'static,
        B: Serialize + ?Sized,
    {
        let url = self.base_url.join(endpoint)?;
        let response = self.client.put(url).json(body).send().await?;
        Self::parse_response(response).await
    }

    async fn parse_response<T>(response: Response) -> AppResult<T>
    where
        T: DeserializeOwned,
    {
        let status = response.status();

        if status.is_success() {
            let parsed = response.json::<T>().await?;
            Ok(parsed)
        } else {
            Err(AppError::from(reqwest::Error::from(
                response.error_for_status().unwrap_err(),
            )))
        }
    }

    pub fn request(&self, method: Method, endpoint: &str) -> AppResult<RequestBuilder> {
        let url = self.base_url.join(endpoint)?;
        Ok(self.client.request(method, url))
    }
}
