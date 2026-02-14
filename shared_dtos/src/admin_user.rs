use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "admin_user.ts", rename = "AdminUser")
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminUserAdminResponse {
    pub id: i64,
    pub login: String,
    pub telegram_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_by: i64,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "validate", derive(validator::Validate))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "admin_user.ts", rename = "NewAdminUser")
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewAdminUserAdminRequest {
    #[cfg_attr(
        feature = "validate",
        validate(length(
            min = 3,
            max = 255,
            message = "Login must be at least 3 characters long and at most 255 characters long"
        ))
    )]
    pub login: String,
    #[cfg_attr(
        feature = "validate",
        validate(length(
            min = 8,
            max = 20,
            message = "Password must be at least 8 characters long and at most 20 characters long"
        ))
    )]
    pub password: String,
    #[cfg_attr(
        feature = "validate",
        validate(length(min = 1, message = "At least one role must be selected"))
    )]
    pub roles: Vec<i64>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "admin_user.ts", rename = "NewAdminUserResponse")
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewAdminUserAdminResponse {
    pub id: i64,
    pub login: String,
    pub telegram_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_by: i64,
    pub two_fa_secret: String,
    pub two_fa_qr_code: String,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "validate", derive(validator::Validate))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "admin_user.ts", rename = "UpdateAdminUser")
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAdminUserAdminRequest {
    #[cfg_attr(
        feature = "validate",
        validate(length(
            min = 3,
            max = 255,
            message = "Login must be at least 3 characters long and at most 255 characters long"
        ))
    )]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub login: Option<String>,
    #[cfg_attr(
        feature = "validate",
        validate(length(
            min = 8,
            max = 20,
            message = "Password must be at least 8 characters long and at most 20 characters long"
        ))
    )]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub password: Option<String>,
    #[cfg_attr(feature = "ts", ts(optional))]
    pub telegram_id: Option<i64>,
    #[cfg_attr(
        feature = "validate",
        validate(length(min = 1, message = "At least one role must be selected"))
    )]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub roles: Option<Vec<i64>>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "admin_user.ts", rename = "RoleSummary")
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleSummaryAdminResponse {
    pub id: i64,
    pub name: String,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "admin_user.ts", rename = "AdminUserWithRoles")
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminUserWithRolesAdminResponse {
    pub id: i64,
    pub login: String,
    pub telegram_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_by: i64,
    pub roles: Vec<RoleSummaryAdminResponse>,
}
