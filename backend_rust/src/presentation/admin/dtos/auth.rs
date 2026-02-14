#[cfg(test)]
mod tests {
    use shared_dtos::auth::{LoginStep1AdminRequest, LoginStep2AdminRequest};
    use uuid::Uuid;
    use validator::Validate;

    #[test]
    fn test_login_step1_request_validation() {
        // Valid data
        let req = LoginStep1AdminRequest {
            login: "goodlogin".to_string(),
            password: "goodpassword".to_string(),
        };
        assert!(req.validate().is_ok());

        // Invalid login (too short)
        let req = LoginStep1AdminRequest {
            login: "a".to_string(),
            password: "goodpassword".to_string(),
        };
        assert!(req.validate().is_err());

        // Invalid login (too long)
        let req = LoginStep1AdminRequest {
            login: "a".repeat(21),
            password: "goodpassword".to_string(),
        };
        assert!(req.validate().is_err());

        // Invalid password (too short)
        let req = LoginStep1AdminRequest {
            login: "goodlogin".to_string(),
            password: "a".to_string(),
        };
        assert!(req.validate().is_err());

        // Invalid password (too long)
        let req = LoginStep1AdminRequest {
            login: "goodlogin".to_string(),
            password: "a".repeat(21),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_login_step2_request_validation() {
        // Valid data
        let req = LoginStep2AdminRequest {
            temp_token: Uuid::new_v4(),
            code: "123456".to_string(),
        };
        assert!(req.validate().is_ok());

        // Invalid code (too short)
        let req = LoginStep2AdminRequest {
            temp_token: Uuid::new_v4(),
            code: "123".to_string(),
        };
        assert!(req.validate().is_err());

        // Invalid code (too long)
        let req = LoginStep2AdminRequest {
            temp_token: Uuid::new_v4(),
            code: "1234567".to_string(),
        };
        assert!(req.validate().is_err());
    }
}
