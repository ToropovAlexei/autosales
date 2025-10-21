pub mod user;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct DispatchMessagePayload {
    pub bot_name: String,
    pub telegram_id: i64,
    pub message: String,
    pub message_to_edit: Option<i32>,
    pub message_to_delete: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackendResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CaptchaResponse {
    pub image: String,
    pub solution: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BalanceResponse {
    pub balance: f64,
}
