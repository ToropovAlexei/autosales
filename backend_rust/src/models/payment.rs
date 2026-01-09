use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

#[derive(sqlx::Type, Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS, ToSchema)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "payment.ts")]
pub enum PaymentSystem {
    PlatformCard,
    PlatformSBP,
    Mock,
}
