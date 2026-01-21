use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bot {
    pub id: i64,
    pub token: String,
    pub username: String,
    pub is_active: bool,
    pub is_primary: bool,
    pub referral_percentage: f64,
    pub owner_id: Option<i64>,
}
