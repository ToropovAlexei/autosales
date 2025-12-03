use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct BotUser {
    pub id: i64,
    pub telegram_id: i64,
    pub is_blocked: bool,
    pub has_passed_captcha: bool,
    pub balance: f64,
    pub registered_with_bot: String,
    pub last_seen_with_bot: String,
    pub last_seen_at: String,
    pub created_at: String,
    pub is_new: bool,
}
