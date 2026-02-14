use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "auth.ts", rename = "LoginStep1")
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "validate", derive(validator::Validate))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginStep1AdminRequest {
    #[cfg_attr(
        feature = "validate",
        validate(length(
            min = 3,
            max = 20,
            message = "Login must be between 3 and 20 characters"
        ))
    )]
    pub login: String,
    #[cfg_attr(
        feature = "validate",
        validate(length(
            min = 6,
            max = 20,
            message = "Password must be between 6 and 20 characters"
        ))
    )]
    pub password: String,
}

#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "auth.ts", rename = "LoginStep1Response")
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginStep1AdminResponse {
    pub temp_token: Uuid,
}

#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "auth.ts", rename = "LoginStep2")
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "validate", derive(validator::Validate))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginStep2AdminRequest {
    pub temp_token: Uuid,
    #[cfg_attr(
        feature = "validate",
        validate(length(min = 6, max = 6, message = "Code must be 6 characters"))
    )]
    pub code: String,
}

#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "auth.ts", rename = "LoginStep2Response")
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginStep2AdminResponse {
    pub token: Uuid,
}
