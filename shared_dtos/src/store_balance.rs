use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "store_balance.ts", rename = "StoreBalance")
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreBalanceAdminResponse {
    pub balance: f64,
}
