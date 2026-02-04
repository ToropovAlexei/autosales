use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Serialize, Deserialize)]
pub struct SettingsBotResponse {
    pub bot_messages_support: String,
    pub bot_messages_support_image_id: Option<Uuid>,
    pub bot_messages_new_user_welcome: String,
    pub bot_messages_new_user_welcome_image_id: Option<Uuid>,
    pub bot_messages_returning_user_welcome: String,
    pub bot_messages_returning_user_welcome_image_id: Option<Uuid>,
    pub pricing_global_markup: f64,
    pub pricing_platform_commission: f64,
    pub pricing_gateway_markup: f64,
    pub pricing_gateway_bonus_mock_provider: f64,
    pub pricing_gateway_bonus_platform_card: f64,
    pub pricing_gateway_bonus_platform_sbp: f64,
    pub referral_program_enabled: bool,
    pub referral_percentage: f64,
    pub bot_payment_system_support_operators: Vec<String>,
    pub bot_about: String,
    pub bot_description: String,
}
