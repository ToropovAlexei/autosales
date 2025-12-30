use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::{ToResponse, ToSchema};
use validator::Validate;

use crate::models::{admin_user::AdminUserRow, admin_user_with_roles::AdminUserWithRolesRow};

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema, ToResponse)]
#[ts(export, export_to = "admin_user.ts", rename = "AdminUser")]
pub struct AdminUserResponse {
    pub id: i64,
    pub login: String,
    pub telegram_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_by: i64,
}

impl From<AdminUserRow> for AdminUserResponse {
    fn from(r: AdminUserRow) -> Self {
        AdminUserResponse {
            id: r.id,
            login: r.login,
            telegram_id: r.telegram_id,
            deleted_at: r.deleted_at,
            created_by: r.created_by,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, TS, ToSchema, ToResponse)]
#[ts(export, export_to = "admin_user.ts", rename = "NewAdminUser")]
pub struct NewAdminUserRequest {
    #[validate(length(
        min = 3,
        max = 255,
        message = "Login must be at least 3 characters long and at most 255 characters long"
    ))]
    pub login: String,
    #[validate(length(
        min = 8,
        max = 20,
        message = "Password must be at least 8 characters long and at most 20 characters long"
    ))]
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, TS, ToSchema, ToResponse)]
#[ts(export, export_to = "admin_user.ts", rename = "UpdateAdminUser")]
pub struct UpdateAdminUserRequest {
    #[validate(length(
        min = 3,
        max = 255,
        message = "Login must be at least 3 characters long and at most 255 characters long"
    ))]
    pub login: Option<String>,
    #[validate(length(
        min = 8,
        max = 20,
        message = "Password must be at least 8 characters long and at most 20 characters long"
    ))]
    pub password: Option<String>,
    pub telegram_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema, ToResponse)]
#[ts(export, export_to = "admin_user.ts", rename = "RoleSummary")]
pub struct RoleSummaryResponse {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema, ToResponse)]
#[ts(export, export_to = "admin_user.ts", rename = "AdminUserWithRoles")]
pub struct AdminUserWithRolesResponse {
    pub id: i64,
    pub login: String,
    pub telegram_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_by: i64,
    pub roles: Vec<RoleSummaryResponse>,
}

impl From<AdminUserWithRolesRow> for AdminUserWithRolesResponse {
    fn from(r: AdminUserWithRolesRow) -> Self {
        AdminUserWithRolesResponse {
            id: r.id,
            login: r.login,
            telegram_id: r.telegram_id,
            deleted_at: r.deleted_at,
            created_by: r.created_by,
            created_at: r.created_at,
            updated_at: r.updated_at,
            roles: r
                .roles
                .iter()
                .map(|r| RoleSummaryResponse {
                    id: r.id,
                    name: r.name.clone(),
                })
                .collect(),
        }
    }
}
