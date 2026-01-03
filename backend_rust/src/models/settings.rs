use bigdecimal::BigDecimal;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Settings {
    pub bot_messages_support: String,
    pub bot_messages_support_image_id: Option<Uuid>,
    pub bot_messages_new_user_welcome: String,
    pub bot_messages_new_user_welcome_image_id: Option<Uuid>,
    pub bot_messages_returning_user_welcome: String,
    pub bot_messages_returning_user_welcome_image_id: Option<Uuid>,
    pub pricing_global_markup: BigDecimal,
    pub pricing_platform_commission: BigDecimal,
    pub pricing_gateway_markup: BigDecimal,
    pub pricing_gateway_bonus_mock_provider: BigDecimal,
    pub pricing_gateway_bonus_platform_card: BigDecimal,
    pub pricing_gateway_bonus_platform_sbp: BigDecimal,
    pub referral_program_enabled: bool,
    pub referral_percentage: BigDecimal,
}

#[derive(Debug, Default)]
pub struct UpdateSettings {
    pub bot_messages_support: Option<String>,
    pub bot_messages_support_image_id: Option<Option<Uuid>>,
    pub bot_messages_new_user_welcome: Option<String>,
    pub bot_messages_new_user_welcome_image_id: Option<Option<Uuid>>,
    pub bot_messages_returning_user_welcome: Option<String>,
    pub bot_messages_returning_user_welcome_image_id: Option<Option<Uuid>>,
    pub pricing_global_markup: Option<BigDecimal>,
    pub pricing_platform_commission: Option<BigDecimal>,
    pub pricing_gateway_markup: Option<BigDecimal>,
    pub pricing_gateway_bonus_mock_provider: Option<BigDecimal>,
    pub pricing_gateway_bonus_platform_card: Option<BigDecimal>,
    pub pricing_gateway_bonus_platform_sbp: Option<BigDecimal>,
    pub referral_program_enabled: Option<bool>,
    pub referral_percentage: Option<BigDecimal>,
}
