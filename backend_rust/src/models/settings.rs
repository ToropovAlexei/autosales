use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Settings {
    pub bot_messages_support: String,
    pub bot_messages_support_image_id: Option<Uuid>,
    pub bot_messages_new_user_welcome: String,
    pub bot_messages_new_user_welcome_image_id: Option<Uuid>,
    pub bot_messages_returning_user_welcome: String,
    pub bot_messages_returning_user_welcome_image_id: Option<Uuid>,
    pub pricing_global_markup: Decimal,
    pub pricing_platform_commission: Decimal,
    pub pricing_gateway_markup: Decimal,
    pub pricing_gateway_bonus_mock_provider: Decimal,
    pub pricing_gateway_bonus_platform_card: Decimal,
    pub pricing_gateway_bonus_platform_sbp: Decimal,
    pub referral_program_enabled: bool,
    pub referral_percentage: Decimal,
}

#[derive(Debug, Default)]
pub struct UpdateSettings {
    pub bot_messages_support: Option<String>,
    pub bot_messages_support_image_id: Option<Option<Uuid>>,
    pub bot_messages_new_user_welcome: Option<String>,
    pub bot_messages_new_user_welcome_image_id: Option<Option<Uuid>>,
    pub bot_messages_returning_user_welcome: Option<String>,
    pub bot_messages_returning_user_welcome_image_id: Option<Option<Uuid>>,
    pub pricing_global_markup: Option<Decimal>,
    pub pricing_platform_commission: Option<Decimal>,
    pub pricing_gateway_markup: Option<Decimal>,
    pub pricing_gateway_bonus_mock_provider: Option<Decimal>,
    pub pricing_gateway_bonus_platform_card: Option<Decimal>,
    pub pricing_gateway_bonus_platform_sbp: Option<Decimal>,
    pub referral_program_enabled: Option<bool>,
    pub referral_percentage: Option<Decimal>,
}
