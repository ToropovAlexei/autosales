use serde::{Deserialize, Serialize};
use teloxide::types::CallbackQuery;

#[derive(Serialize, Deserialize, Debug)]
pub enum CategoryAction {
    View,
    Back,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PaymentAction {
    SelectGateway { gateway: String },
    SelectAmount { gateway: String, amount: i64 },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
pub enum CallbackData {
    Category {
        action: CategoryAction,
        category_id: i64,
    },
    Payment {
        action: PaymentAction,
    },
    Balance,
    MyOrders,
    MySubscriptions,
    Deposit,
    ReferralProgram,
    Support,
    MainMenu,
}

impl CallbackData {
    pub fn pack(&self) -> String {
        serde_json::to_string(self).expect("Failed to serialize CallbackData")
    }

    pub fn unpack(s: &str) -> Option<Self> {
        serde_json::from_str(s).ok()
    }

    pub fn from_query(query: &CallbackQuery) -> Option<Self> {
        query
            .data
            .as_ref()
            .and_then(|d| serde_json::from_str::<Self>(d).ok())
    }
}

impl From<CallbackData> for String {
    fn from(value: CallbackData) -> Self {
        value.pack()
    }
}

impl From<String> for CallbackData {
    fn from(value: String) -> Self {
        CallbackData::unpack(&value).unwrap()
    }
}
