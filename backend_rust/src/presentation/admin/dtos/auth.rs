use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::{ToResponse, ToSchema};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema, ToResponse, Validate)]
#[ts(export, export_to = "auth.ts", rename = "LoginStep1")]
pub struct LoginStep1Request {
    #[validate(length(
        min = 3,
        max = 20,
        message = "Login must be between 3 and 20 characters"
    ))]
    pub login: String,
    #[validate(length(
        min = 6,
        max = 20,
        message = "Password must be between 6 and 20 characters"
    ))]
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema, ToResponse)]
#[ts(export, export_to = "auth.ts", rename = "LoginStep1Response")]
pub struct LoginStep1Response {
    pub temp_token: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema, ToResponse, Validate)]
#[ts(export, export_to = "auth.ts", rename = "LoginStep2")]
pub struct LoginStep2Request {
    pub temp_token: Uuid,
    #[validate(length(min = 6, max = 6, message = "Code must be 6 characters"))]
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema, ToResponse)]
#[ts(export, export_to = "auth.ts", rename = "LoginStep2Response")]
pub struct LoginStep2Response {
    pub token: Uuid,
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn test_login_step1_request_validation() {
        // Valid data
        let req = LoginStep1Request {
            login: "goodlogin".to_string(),
            password: "goodpassword".to_string(),
        };
        assert!(req.validate().is_ok());

        // Invalid login (too short)
        let req = LoginStep1Request {
            login: "a".to_string(),
            password: "goodpassword".to_string(),
        };
        assert!(req.validate().is_err());

        // Invalid login (too long)
        let req = LoginStep1Request {
            login: "a".repeat(21),
            password: "goodpassword".to_string(),
        };
        assert!(req.validate().is_err());

        // Invalid password (too short)
        let req = LoginStep1Request {
            login: "goodlogin".to_string(),
            password: "a".to_string(),
        };
        assert!(req.validate().is_err());

        // Invalid password (too long)
        let req = LoginStep1Request {
            login: "goodlogin".to_string(),
            password: "a".repeat(21),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_login_step2_request_validation() {
        // Valid data
        let req = LoginStep2Request {
            temp_token: Uuid::new_v4(),
            code: "123456".to_string(),
        };
        assert!(req.validate().is_ok());

        // Invalid code (too short)
        let req = LoginStep2Request {
            temp_token: Uuid::new_v4(),
            code: "123".to_string(),
        };
        assert!(req.validate().is_err());

        // Invalid code (too long)
        let req = LoginStep2Request {
            temp_token: Uuid::new_v4(),
            code: "1234567".to_string(),
        };
        assert!(req.validate().is_err());
    }
}
