use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContmsProxy {
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
    pub proxy: Vec<ContmsProxy>,
}
