use rust_decimal::prelude::ToPrimitive;
use shared_dtos::bot::BotAdminResponse;

use crate::models::bot::BotRow;

impl From<BotRow> for BotAdminResponse {
    fn from(r: BotRow) -> Self {
        BotAdminResponse {
            id: r.id,
            owner_id: r.owner_id,
            token: r.token,
            username: r.username,
            r#type: r.r#type,
            is_active: r.is_active,
            is_primary: r.is_primary,
            referral_percentage: r.referral_percentage.to_f64().unwrap_or_default(),
            created_at: r.created_at,
            updated_at: r.updated_at,
            created_by: r.created_by,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use rust_decimal::Decimal;
    use shared_dtos::bot::{BotType, NewBotAdminRequest, UpdateBotAdminRequest};
    use validator::Validate;

    #[test]
    fn test_bot_response_from_bot_row() {
        let now = Utc::now();
        let bot_row = BotRow {
            id: 1,
            owner_id: Some(10),
            token: "test_token".to_string(),
            username: "test_bot".to_string(),
            r#type: BotType::Main,
            is_active: true,
            is_primary: false,
            referral_percentage: Decimal::from(50),
            created_at: now,
            updated_at: now,
            created_by: Some(1),
        };

        let bot_response: BotAdminResponse = bot_row.into();

        assert_eq!(bot_response.id, 1);
        assert_eq!(bot_response.owner_id, Some(10));
        assert_eq!(bot_response.token, "test_token");
        assert_eq!(bot_response.username, "test_bot");
        assert_eq!(bot_response.r#type, BotType::Main);
        assert!(bot_response.is_active);
        assert!(!bot_response.is_primary);
        assert_eq!(bot_response.referral_percentage, 50.0);
        assert_eq!(bot_response.created_at, now);
        assert_eq!(bot_response.updated_at, now);
        assert_eq!(bot_response.created_by, Some(1));
    }

    #[test]
    fn test_new_bot_request_validation() {
        // Valid token length
        let req = NewBotAdminRequest {
            token: "a".repeat(44), // Min length
        };
        assert!(req.validate().is_ok());

        let req = NewBotAdminRequest {
            token: "a".repeat(48), // Max length
        };
        assert!(req.validate().is_ok());

        // Too short
        let req = NewBotAdminRequest {
            token: "a".repeat(43),
        };
        assert!(req.validate().is_err());

        // Too long
        let req = NewBotAdminRequest {
            token: "a".repeat(49),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_update_bot_request_validation() {
        // Valid referral_percentage
        let req = UpdateBotAdminRequest {
            is_active: None,
            is_primary: None,
            referral_percentage: Some(50.0),
        };
        assert!(req.validate().is_ok());

        let req = UpdateBotAdminRequest {
            is_active: None,
            is_primary: None,
            referral_percentage: Some(0.0), // Min value
        };
        assert!(req.validate().is_ok());

        let req = UpdateBotAdminRequest {
            is_active: None,
            is_primary: None,
            referral_percentage: Some(100.0), // Max value
        };
        assert!(req.validate().is_ok());

        // Referral percentage too low
        let req = UpdateBotAdminRequest {
            is_active: None,
            is_primary: None,
            referral_percentage: Some(-0.1),
        };
        assert!(req.validate().is_err());

        // Referral percentage too high
        let req = UpdateBotAdminRequest {
            is_active: None,
            is_primary: None,
            referral_percentage: Some(100.1),
        };
        assert!(req.validate().is_err());

        // Other fields work
        let req = UpdateBotAdminRequest {
            is_active: Some(false),
            is_primary: Some(true),
            referral_percentage: None,
        };
        assert!(req.validate().is_ok());
    }
}
