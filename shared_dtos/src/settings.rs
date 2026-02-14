use serde::{Deserialize, Serialize};
use serde_with::rust::double_option;
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

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "settings.ts", rename = "PricingSettings")
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingSettingsAdminResponse {
    pub pricing_global_markup: f64,
    pub pricing_platform_commission: f64,
    pub pricing_gateway_markup: f64,
    pub pricing_gateway_bonus_mock_provider: f64,
    pub pricing_gateway_bonus_platform_card: f64,
    pub pricing_gateway_bonus_platform_sbp: f64,
    pub referral_program_enabled: bool,
    pub referral_percentage: f64,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "settings.ts", rename = "BotSettings")
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotSettingsAdminResponse {
    pub bot_messages_support: String,
    pub bot_messages_support_image_id: Option<Uuid>,
    pub bot_messages_new_user_welcome: String,
    pub bot_messages_new_user_welcome_image_id: Option<Uuid>,
    pub bot_messages_returning_user_welcome: String,
    pub bot_messages_returning_user_welcome_image_id: Option<Uuid>,
    pub bot_payment_system_support_operators: Vec<String>,
    pub bot_description: String,
    pub bot_about: String,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "settings.ts", rename = "UpdatePricingSettings")
)]
#[cfg_attr(feature = "validate", derive(validator::Validate))]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdatePricingSettingsAdminRequest {
    #[cfg_attr(feature = "validate", validate(range(min = 0.0, max = 10000.0)))]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub pricing_global_markup: Option<f64>,
    #[cfg_attr(feature = "validate", validate(range(min = 0.0, max = 100.0)))]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub pricing_platform_commission: Option<f64>,
    #[cfg_attr(feature = "validate", validate(range(min = 0.0, max = 100.0)))]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub pricing_gateway_markup: Option<f64>,
    #[cfg_attr(feature = "validate", validate(range(min = 0.0, max = 100.0)))]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub pricing_gateway_bonus_mock_provider: Option<f64>,
    #[cfg_attr(feature = "validate", validate(range(min = 0.0, max = 100.0)))]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub pricing_gateway_bonus_platform_card: Option<f64>,
    #[cfg_attr(feature = "validate", validate(range(min = 0.0, max = 100.0)))]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub pricing_gateway_bonus_platform_sbp: Option<f64>,
    #[cfg_attr(feature = "ts", ts(optional))]
    pub referral_program_enabled: Option<bool>,
    #[cfg_attr(feature = "validate", validate(range(min = 0.0, max = 100.0)))]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub referral_percentage: Option<f64>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "validate", derive(validator::Validate))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "settings.ts", rename = "UpdateBotSettings")
)]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateBotSettingsAdminRequest {
    #[cfg_attr(
        feature = "validate",
        validate(length(max = 999, message = "length must be less than 999"))
    )]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub bot_messages_support: Option<String>,
    #[cfg_attr(feature = "ts", ts(optional))]
    #[cfg_attr(feature = "ts", ts(type = "string | null"))]
    #[serde(default, with = "double_option")]
    pub bot_messages_support_image_id: Option<Option<Uuid>>,
    #[cfg_attr(
        feature = "validate",
        validate(length(max = 999, message = "length must be less than 999"))
    )]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub bot_messages_new_user_welcome: Option<String>,
    #[cfg_attr(feature = "ts", ts(optional))]
    #[cfg_attr(feature = "ts", ts(type = "string | null"))]
    #[serde(default, with = "double_option")]
    pub bot_messages_new_user_welcome_image_id: Option<Option<Uuid>>,
    #[cfg_attr(
        feature = "validate",
        validate(length(max = 999, message = "length must be less than 999"))
    )]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub bot_messages_returning_user_welcome: Option<String>,
    #[cfg_attr(feature = "ts", ts(optional))]
    #[cfg_attr(feature = "ts", ts(type = "string | null"))]
    #[serde(default, with = "double_option")]
    pub bot_messages_returning_user_welcome_image_id: Option<Option<Uuid>>,
    #[cfg_attr(
        feature = "validate",
        validate(length(max = 3, message = "length must be less than 3"))
    )]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub bot_payment_system_support_operators: Option<Vec<String>>,
    #[cfg_attr(
        feature = "validate",
        validate(length(max = 999, message = "length must be less than 999"))
    )]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub bot_description: Option<String>,
    #[cfg_attr(
        feature = "validate",
        validate(length(max = 999, message = "length must be less than 999"))
    )]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub bot_about: Option<String>,
}
