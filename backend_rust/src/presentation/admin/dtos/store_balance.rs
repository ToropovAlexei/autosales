use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::{ToResponse, ToSchema};

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema, ToResponse)]
#[ts(export, export_to = "store_balance.ts", rename = "StoreBalance")]
pub struct StoreBalanceResponse {
    pub balance: f64,
}
