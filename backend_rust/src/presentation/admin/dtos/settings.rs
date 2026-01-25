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

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema, ToResponse, Validate, Default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema, ToResponse, Validate, Default)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    // Helper to create a dummy Settings struct
    fn create_dummy_settings() -> Settings {
        Settings {
            pricing_global_markup: Decimal::from_f64(10.0).unwrap(),
            pricing_platform_commission: Decimal::from_f64(5.0).unwrap(),
            pricing_gateway_markup: Decimal::from_f64(2.0).unwrap(),
            pricing_gateway_bonus_mock_provider: Decimal::from_f64(1.0).unwrap(),
            pricing_gateway_bonus_platform_card: Decimal::from_f64(0.5).unwrap(),
            pricing_gateway_bonus_platform_sbp: Decimal::from_f64(0.2).unwrap(),
            referral_program_enabled: true,
            referral_percentage: Decimal::from_f64(15.0).unwrap(),
            bot_messages_support: "Support text".to_string(),
            bot_messages_support_image_id: Some(Uuid::new_v4()),
            bot_messages_new_user_welcome: "Welcome new user".to_string(),
            bot_messages_new_user_welcome_image_id: Some(Uuid::new_v4()),
            bot_messages_returning_user_welcome: "Welcome back".to_string(),
            bot_messages_returning_user_welcome_image_id: None,
        }
    }

    #[test]
    fn test_pricing_settings_response_from_settings() {
        let settings = create_dummy_settings();
        let response: PricingSettingsResponse = settings.into();

        assert_eq!(response.pricing_global_markup, 10.0);
        assert_eq!(response.pricing_platform_commission, 5.0);
        assert_eq!(response.pricing_gateway_markup, 2.0);
        assert_eq!(response.pricing_gateway_bonus_mock_provider, 1.0);
        assert_eq!(response.pricing_gateway_bonus_platform_card, 0.5);
        assert_eq!(response.pricing_gateway_bonus_platform_sbp, 0.2);
        assert!(response.referral_program_enabled);
        assert_eq!(response.referral_percentage, 15.0);
    }

    #[test]
    fn test_bot_settings_response_from_settings() {
        let settings = create_dummy_settings();
        let response: BotSettingsResponse = settings.into();

        assert_eq!(response.bot_messages_support, "Support text");
        assert!(response.bot_messages_support_image_id.is_some());
        assert_eq!(response.bot_messages_new_user_welcome, "Welcome new user");
        assert!(response.bot_messages_new_user_welcome_image_id.is_some());
        assert_eq!(response.bot_messages_returning_user_welcome, "Welcome back");
        assert!(
            response
                .bot_messages_returning_user_welcome_image_id
                .is_none()
        );
    }

    #[test]
    fn test_update_pricing_settings_request_validation_valid_ranges() {
        let req = UpdatePricingSettingsRequest {
            pricing_global_markup: Some(500.0),
            pricing_platform_commission: Some(50.0),
            pricing_gateway_markup: Some(10.0),
            pricing_gateway_bonus_mock_provider: Some(5.0),
            pricing_gateway_bonus_platform_card: Some(2.5),
            pricing_gateway_bonus_platform_sbp: Some(1.0),
            referral_program_enabled: Some(true),
            referral_percentage: Some(10.0),
        };
        assert!(req.validate().is_ok());

        // Test boundary minimums
        let req = UpdatePricingSettingsRequest {
            pricing_global_markup: Some(0.0),
            pricing_platform_commission: Some(0.0),
            pricing_gateway_markup: Some(0.0),
            pricing_gateway_bonus_mock_provider: Some(0.0),
            pricing_gateway_bonus_platform_card: Some(0.0),
            pricing_gateway_bonus_platform_sbp: Some(0.0),
            referral_program_enabled: Some(false),
            referral_percentage: Some(0.0),
        };
        assert!(req.validate().is_ok());

        // Test boundary maximums
        let req = UpdatePricingSettingsRequest {
            pricing_global_markup: Some(10000.0),
            pricing_platform_commission: Some(100.0),
            pricing_gateway_markup: Some(100.0),
            pricing_gateway_bonus_mock_provider: Some(100.0),
            pricing_gateway_bonus_platform_card: Some(100.0),
            pricing_gateway_bonus_platform_sbp: Some(100.0),
            referral_program_enabled: Some(true),
            referral_percentage: Some(100.0),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_update_pricing_settings_request_validation_invalid_ranges() {
        // global_markup too high
        let req = UpdatePricingSettingsRequest {
            pricing_global_markup: Some(10000.01),
            ..Default::default()
        };
        assert!(req.validate().is_err());

        // platform_commission too low
        let req = UpdatePricingSettingsRequest {
            pricing_platform_commission: Some(-0.01),
            ..Default::default()
        };
        assert!(req.validate().is_err());

        // referral_percentage too high
        let req = UpdatePricingSettingsRequest {
            referral_percentage: Some(100.01),
            ..Default::default()
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_update_bot_settings_request_validation_valid() {
        let req = UpdateBotSettingsRequest {
            bot_messages_support: Some("Short support message".to_string()),
            bot_messages_support_image_id: Some(Some(Uuid::new_v4())),
            bot_messages_new_user_welcome: Some("Short welcome message".to_string()),
            bot_messages_new_user_welcome_image_id: Some(None),
            bot_messages_returning_user_welcome: Some("Short returning message".to_string()),
            bot_messages_returning_user_welcome_image_id: Some(Some(Uuid::new_v4())),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_update_bot_settings_request_validation_too_long() {
        // bot_messages_support too long
        let req = UpdateBotSettingsRequest {
            bot_messages_support: Some("a".repeat(1000)),
            ..Default::default()
        };
        assert!(req.validate().is_err());

        // bot_messages_new_user_welcome too long
        let req = UpdateBotSettingsRequest {
            bot_messages_new_user_welcome: Some("a".repeat(1000)),
            ..Default::default()
        };
        assert!(req.validate().is_err());

        // bot_messages_returning_user_welcome too long
        let req = UpdateBotSettingsRequest {
            bot_messages_returning_user_welcome: Some("a".repeat(1000)),
            ..Default::default()
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_update_settings_command_from_update_pricing_settings_request() {
        let req = UpdatePricingSettingsRequest {
            pricing_global_markup: Some(10.5),
            pricing_platform_commission: Some(2.1),
            referral_program_enabled: Some(true),
            ..Default::default()
        };
        let command: UpdateSettingsCommand = req.into();

        assert_eq!(
            command.pricing_global_markup,
            Some(Decimal::from_f64(10.5).unwrap())
        );
        assert_eq!(
            command.pricing_platform_commission,
            Some(Decimal::from_f64(2.1).unwrap())
        );
        assert_eq!(command.referral_program_enabled, Some(true));
        assert!(command.pricing_gateway_markup.is_none()); // Default is None
    }

    #[test]
    fn test_update_settings_command_from_update_bot_settings_request() {
        let uuid = Uuid::new_v4();
        let req = UpdateBotSettingsRequest {
            bot_messages_support: Some("New support message".to_string()),
            bot_messages_support_image_id: Some(Some(uuid)),
            bot_messages_new_user_welcome: Some("New welcome message".to_string()),
            ..Default::default()
        };
        let command: UpdateSettingsCommand = req.into();

        assert_eq!(
            command.bot_messages_support,
            Some("New support message".to_string())
        );
        assert_eq!(command.bot_messages_support_image_id, Some(Some(uuid)));
        assert_eq!(
            command.bot_messages_new_user_welcome,
            Some("New welcome message".to_string())
        );
        assert!(command.bot_messages_returning_user_welcome.is_none()); // Default is None
    }
}
