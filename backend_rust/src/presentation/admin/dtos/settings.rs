use rust_decimal::prelude::{Decimal, FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::{ToResponse, ToSchema};
use uuid::Uuid;
use validator::Validate;

use crate::{models::settings::Settings, services::settings::UpdateSettingsCommand};

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema, ToResponse)]
#[ts(export, export_to = "settings.ts", rename = "PricingSettings")]
pub struct PricingSettingsResponse {
    pub pricing_global_markup: f64,
    pub pricing_platform_commission: f64,
    pub pricing_gateway_markup: f64,
    pub pricing_gateway_bonus_mock_provider: f64,
    pub pricing_gateway_bonus_platform_card: f64,
    pub pricing_gateway_bonus_platform_sbp: f64,
    pub referral_program_enabled: bool,
    pub referral_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema, ToResponse)]
#[ts(export, export_to = "settings.ts", rename = "BotSettings")]
pub struct BotSettingsResponse {
    pub bot_messages_support: String,
    pub bot_messages_support_image_id: Option<Uuid>,
    pub bot_messages_new_user_welcome: String,
    pub bot_messages_new_user_welcome_image_id: Option<Uuid>,
    pub bot_messages_returning_user_welcome: String,
    pub bot_messages_returning_user_welcome_image_id: Option<Uuid>,
}

impl From<Settings> for PricingSettingsResponse {
    fn from(r: Settings) -> Self {
        PricingSettingsResponse {
            pricing_gateway_bonus_mock_provider: r
                .pricing_gateway_bonus_mock_provider
                .to_f64()
                .unwrap_or_default(),
            pricing_gateway_bonus_platform_card: r
                .pricing_gateway_bonus_platform_card
                .to_f64()
                .unwrap_or_default(),
            pricing_gateway_bonus_platform_sbp: r
                .pricing_gateway_bonus_platform_sbp
                .to_f64()
                .unwrap_or_default(),
            pricing_gateway_markup: r.pricing_gateway_markup.to_f64().unwrap_or_default(),
            pricing_global_markup: r.pricing_global_markup.to_f64().unwrap_or_default(),
            pricing_platform_commission: r.pricing_platform_commission.to_f64().unwrap_or_default(),
            referral_percentage: r.referral_percentage.to_f64().unwrap_or_default(),
            referral_program_enabled: r.referral_program_enabled,
        }
    }
}

impl From<Settings> for BotSettingsResponse {
    fn from(r: Settings) -> Self {
        BotSettingsResponse {
            bot_messages_support: r.bot_messages_support,
            bot_messages_support_image_id: r.bot_messages_support_image_id,
            bot_messages_new_user_welcome: r.bot_messages_new_user_welcome,
            bot_messages_new_user_welcome_image_id: r.bot_messages_new_user_welcome_image_id,
            bot_messages_returning_user_welcome: r.bot_messages_returning_user_welcome,
            bot_messages_returning_user_welcome_image_id: r
                .bot_messages_returning_user_welcome_image_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema, ToResponse, Validate)]
#[ts(export, export_to = "settings.ts", rename = "UpdatePricingSettings")]
pub struct UpdatePricingSettingsRequest {
    #[validate(range(min = 0.0, max = 10000.0))]
    #[ts(optional)]
    pub pricing_global_markup: Option<f64>,
    #[validate(range(min = 0.0, max = 100.0))]
    #[ts(optional)]
    pub pricing_platform_commission: Option<f64>,
    #[validate(range(min = 0.0, max = 100.0))]
    #[ts(optional)]
    pub pricing_gateway_markup: Option<f64>,
    #[validate(range(min = 0.0, max = 100.0))]
    #[ts(optional)]
    pub pricing_gateway_bonus_mock_provider: Option<f64>,
    #[validate(range(min = 0.0, max = 100.0))]
    #[ts(optional)]
    pub pricing_gateway_bonus_platform_card: Option<f64>,
    #[validate(range(min = 0.0, max = 100.0))]
    #[ts(optional)]
    pub pricing_gateway_bonus_platform_sbp: Option<f64>,
    #[ts(optional)]
    pub referral_program_enabled: Option<bool>,
    #[validate(range(min = 0.0, max = 100.0))]
    #[ts(optional)]
    pub referral_percentage: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema, ToResponse, Validate)]
#[ts(export, export_to = "settings.ts", rename = "UpdateBotSettings")]
pub struct UpdateBotSettingsRequest {
    #[validate(length(max = 999, message = "length must be less than 999"))]
    #[ts(optional)]
    pub bot_messages_support: Option<String>,
    #[ts(optional)]
    pub bot_messages_support_image_id: Option<Option<Uuid>>,
    #[validate(length(max = 999, message = "length must be less than 999"))]
    #[ts(optional)]
    pub bot_messages_new_user_welcome: Option<String>,
    #[ts(optional)]
    pub bot_messages_new_user_welcome_image_id: Option<Option<Uuid>>,
    #[validate(length(max = 999, message = "length must be less than 999"))]
    #[ts(optional)]
    pub bot_messages_returning_user_welcome: Option<String>,
    #[ts(optional)]
    pub bot_messages_returning_user_welcome_image_id: Option<Option<Uuid>>,
}

impl From<UpdatePricingSettingsRequest> for UpdateSettingsCommand {
    fn from(r: UpdatePricingSettingsRequest) -> Self {
        let f64_opt_to_bd =
            |opt: Option<f64>| opt.map(|f| Decimal::from_f64(f).unwrap_or_default());
        UpdateSettingsCommand {
            pricing_gateway_bonus_mock_provider: f64_opt_to_bd(
                r.pricing_gateway_bonus_mock_provider,
            ),
            pricing_gateway_bonus_platform_card: f64_opt_to_bd(
                r.pricing_gateway_bonus_platform_card,
            ),
            pricing_gateway_bonus_platform_sbp: f64_opt_to_bd(r.pricing_gateway_bonus_platform_sbp),
            pricing_gateway_markup: f64_opt_to_bd(r.pricing_gateway_markup),
            pricing_global_markup: f64_opt_to_bd(r.pricing_global_markup),
            pricing_platform_commission: f64_opt_to_bd(r.pricing_platform_commission),
            referral_percentage: f64_opt_to_bd(r.referral_percentage),
            referral_program_enabled: r.referral_program_enabled,
            ..UpdateSettingsCommand::default()
        }
    }
}

impl From<UpdateBotSettingsRequest> for UpdateSettingsCommand {
    fn from(r: UpdateBotSettingsRequest) -> Self {
        UpdateSettingsCommand {
            bot_messages_support: r.bot_messages_support,
            bot_messages_support_image_id: r.bot_messages_support_image_id,
            bot_messages_new_user_welcome: r.bot_messages_new_user_welcome,
            bot_messages_new_user_welcome_image_id: r.bot_messages_new_user_welcome_image_id,
            bot_messages_returning_user_welcome: r.bot_messages_returning_user_welcome,
            bot_messages_returning_user_welcome_image_id: r
                .bot_messages_returning_user_welcome_image_id,
            ..UpdateSettingsCommand::default()
        }
    }
}
