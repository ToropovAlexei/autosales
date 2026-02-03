use crate::models::settings::Settings;
use rust_decimal::prelude::ToPrimitive;
use shared_dtos::settings::SettingsBotResponse;

impl From<Settings> for SettingsBotResponse {
    fn from(r: Settings) -> Self {
        SettingsBotResponse {
            bot_messages_support: r.bot_messages_support,
            bot_messages_support_image_id: r.bot_messages_support_image_id,
            bot_messages_new_user_welcome: r.bot_messages_new_user_welcome,
            bot_messages_new_user_welcome_image_id: r.bot_messages_new_user_welcome_image_id,
            bot_messages_returning_user_welcome: r.bot_messages_returning_user_welcome,
            bot_messages_returning_user_welcome_image_id: r
                .bot_messages_returning_user_welcome_image_id,
            pricing_global_markup: r.pricing_global_markup.to_f64().unwrap_or_default(),
            pricing_platform_commission: r.pricing_platform_commission.to_f64().unwrap_or_default(),
            pricing_gateway_markup: r.pricing_gateway_markup.to_f64().unwrap_or_default(),
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
            referral_program_enabled: r.referral_program_enabled,
            referral_percentage: r.referral_percentage.to_f64().unwrap_or_default(),
            bot_payment_system_support_operators: r.bot_payment_system_support_operators,
        }
    }
}
