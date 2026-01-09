use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentSystem {
    PlatformCard,
    PlatformSBP,
    Mock,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaymentGateway {
    pub name: PaymentSystem,
    pub display_name: String,
}
