use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::{ToResponse, ToSchema};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema, ToResponse)]
#[ts(export, export_to = "auth.ts", rename = "LoginStep1")]
pub struct LoginStep1Request {
    pub login: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema, ToResponse)]
#[ts(export, export_to = "auth.ts", rename = "LoginStep1Response")]
pub struct LoginStep1Response {
    pub temp_token: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema, ToResponse)]
#[ts(export, export_to = "auth.ts", rename = "LoginStep2")]
pub struct LoginStep2Request {
    pub temp_token: Uuid,
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema, ToResponse)]
#[ts(export, export_to = "auth.ts", rename = "LoginStep2Response")]
pub struct LoginStep2Response {
    pub token: String,
}
