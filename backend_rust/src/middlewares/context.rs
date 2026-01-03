use std::{convert::Infallible, net::IpAddr};

use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts},
};
use axum_client_ip::ClientIp;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct RequestContext {
    pub ip_address: Option<IpAddr>,
    pub user_agent: Option<String>,
    pub request_id: Uuid,
}

impl<S> FromRequestParts<S> for RequestContext
where
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let ip = ClientIp::from_request_parts(parts, state).await.ok();

        let user_agent = parts
            .headers
            .get(header::USER_AGENT)
            .and_then(|v| v.to_str().ok());

        let request_id = Uuid::new_v4();

        Ok(RequestContext {
            ip_address: ip.map(|ip| ip.0),
            user_agent: user_agent.map(String::from),
            request_id,
        })
    }
}
