use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::models::payment::PaymentSystem;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GatewayResponse {
    pub name: PaymentSystem,
    pub display_name: String,
}
