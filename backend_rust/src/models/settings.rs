use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Default)]
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
    pub bot_payment_system_support_operators: Vec<String>,
    pub bot_description: String,
    pub bot_about: String,
    pub manager_group_chat_id: Option<i64>,
    pub usdt_rate_rub: Decimal,
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
    pub bot_payment_system_support_operators: Option<Vec<String>>,
    pub bot_description: Option<String>,
    pub bot_about: Option<String>,
    pub manager_group_chat_id: Option<Option<i64>>,
    pub usdt_rate_rub: Option<Decimal>,
}
