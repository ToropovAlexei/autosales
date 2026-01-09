use chrono::Duration;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContmsProxyResponse {
    pub port: u16,
    pub total: u16,
    pub name: String,
    pub r#type: String,
    pub host: String,
    pub ipv: u8,
    pub sonet: String,
    pub auth: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContmsAvailableResponse {
    pub status: String,
    pub action: String,
    pub proxy: Vec<ContmsProxyResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContmsUserResponse {
    pub name: String,
    pub proxy: String,
    pub expires: i64,
    pub pass: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContmsUpResponse {
    pub status: String,
    pub action: String,
    pub user: ContmsUserResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContmsDownResponse {
    pub status: String,
    pub action: String,
    pub user: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContmsStatusProxyResponse {
    pub expires: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContmsStatusResponse {
    pub status: String,
    pub action: String,
    pub proxy: ContmsStatusProxyResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpProxyRequest {
    pub name: String,
    pub expires: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContmsRenewResponse {
    pub status: String,
    pub action: String,
    pub user: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum ContmsRequestAction {
    #[serde(rename = "available")]
    Available,
    #[serde(rename = "up")]
    Up { proxy: UpProxyRequest },
    #[serde(rename = "down")]
    Down { user: String },
    #[serde(rename = "renew")]
    Renew { user: String, expires: Duration },
    #[serde(rename = "status")]
    Status { user: String },
}
