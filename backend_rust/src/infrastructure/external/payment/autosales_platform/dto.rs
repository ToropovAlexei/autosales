use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AutosalesPlatformResponseStatus {
    Success,
    Error,
}

#[derive(Debug)]
pub enum AutosalesPlatformError {
    NoSuitableRequisites,
    IncreaseAmountBy10,
    Unknown(String),
}

impl<'de> Deserialize<'de> for AutosalesPlatformError {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "Нет подходящих реквизитов" => Ok(Self::NoSuitableRequisites),
            "<b>ПОМЕНЯЙТЕ СУММУ НА 10₽</b>" => Ok(Self::IncreaseAmountBy10),
            other => Ok(Self::Unknown(other.to_string())),
        }
    }
}

impl From<AutosalesPlatformError> for String {
    fn from(value: AutosalesPlatformError) -> Self {
        match value {
            AutosalesPlatformError::NoSuitableRequisites => "Нет подходящих реквизитов".to_string(),
            AutosalesPlatformError::IncreaseAmountBy10 => {
                "<b>ПОМЕНЯЙТЕ СУММУ НА 10₽</b>".to_string()
            }
            AutosalesPlatformError::Unknown(s) => s,
        }
    }
}

impl From<String> for AutosalesPlatformError {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Нет подходящих реквизитов" => Self::NoSuitableRequisites,
            "<b>ПОМЕНЯЙТЕ СУММУ НА 10₽</b>" => Self::IncreaseAmountBy10,
            other => Self::Unknown(other.to_string()),
        }
    }
}

impl std::fmt::Display for AutosalesPlatformError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AutosalesPlatformError::NoSuitableRequisites => {
                write!(f, "Нет подходящих реквизитов")
            }
            AutosalesPlatformError::IncreaseAmountBy10 => {
                write!(f, "<b>ПОМЕНЯЙТЕ СУММУ НА 10₽</b>")
            }
            AutosalesPlatformError::Unknown(s) => write!(f, "{s}"),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "response")]
pub enum AutosalesPlatformResponse<T> {
    #[serde(rename = "success")]
    Success {
        message: String,
        query: serde_json::Value,
        data: T,
    },
    #[serde(rename = "error")]
    Error {
        message: AutosalesPlatformError,
        query: serde_json::Value,
        data: Option<serde_json::Value>,
    },
}

#[derive(Debug, Deserialize)]
pub struct AutosalesPlatformAuthStep1Data {
    pub temp: String,
}

#[derive(Debug, Deserialize)]
pub struct AutosalesPlatformAuthStep2Data {
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct AutosalesPlatformAuthStep1Request {
    pub version: i64,
    pub login: String,
    pub password: String,
}

pub type AutosalesPlatformAuthStep1Response =
    AutosalesPlatformResponse<AutosalesPlatformAuthStep1Data>;

#[derive(Debug, Serialize)]
pub struct AutosalesPlatformAuthStep2Request {
    pub version: i64,
    pub temp: String,
    pub key: String,
}

pub type AutosalesPlatformAuthStep2Response =
    AutosalesPlatformResponse<AutosalesPlatformAuthStep2Data>;

#[derive(Debug, Deserialize, Serialize)]
pub struct AutosalesPlatformOrderInitializedDataRequisiteBank {
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AutosalesPlatformOrderInitializedDataRequisitePeople {
    pub surname: String,
    pub name: String,
    pub patronymic: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AutosalesPlatformOrderInitializedDataRequisiteMathematics {
    pub currency: String,
    pub country: String,
    pub amount_pay: f64,
    pub amount_transfer: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AutosalesPlatformOrderInitializedDataRequisite {
    pub object_token: String,
    pub value: String,
    pub data_bank: AutosalesPlatformOrderInitializedDataRequisiteBank,
    pub data_people: AutosalesPlatformOrderInitializedDataRequisitePeople,
    pub data_mathematics: AutosalesPlatformOrderInitializedDataRequisiteMathematics,
}

#[derive(Debug, Deserialize)]
pub struct AutosalesPlatformOrderInitializedData {
    pub data_requisite: AutosalesPlatformOrderInitializedDataRequisite,
}

pub type AutosalesPlatformOrderInitializedResponse =
    AutosalesPlatformResponse<AutosalesPlatformOrderInitializedData>;

#[derive(Debug, Deserialize)]
pub struct AutosalesPlatformOrderStatusRequisiteData {
    pub country: String,
    pub bank_img: String,
    pub bank_name: String,
    pub currency: String,
    pub emoji: String,
    pub value: String,
    pub r#type: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AutosalesPlatformOrderStatusType {
    MerchProcess,
    MerchInitialized,
    TraderSuccess,
    MerchSuccess,
    SystemTimerEndMerchProcessSuccess,
    SystemTimerEndMerchCheckDownSuccess,
    AdminAppealSuccess,
    TraderCheckQuery,
    TraderAppeal,
    SystemTimerEndMerchInitializedCancel,
    OrderCancel,
    MerchCancel,
    SystemTimerEndTraderCheckQueryCancel,
    AdminAppealCancel,
    MerchCheckDown,
}

#[derive(Debug, Deserialize)]
pub struct AutosalesPlatformOrderStatus {
    pub token: String,
    pub status: AutosalesPlatformOrderStatusType,
    pub appeal_fake_status: Option<String>,
    pub appeal_url_file: Option<String>,
    pub requisite_data: AutosalesPlatformOrderStatusRequisiteData,
    pub token_link: Option<String>,
    pub amount_order_requested: String,
    pub amount_order_charged: String,
    pub datetime_created_cosmetics: String,
    pub datetime_created_datetime: String,
}

#[derive(Debug, Deserialize)]
pub struct AutosalesPlatformOrderStatusData {
    pub status: AutosalesPlatformOrderStatus,
}

pub type AutosalesPlatformOrderStatusResponse =
    AutosalesPlatformResponse<AutosalesPlatformOrderStatusData>;

#[derive(Debug, Deserialize, Serialize)]
pub enum AutosalesPlatformPaymentMethod {
    #[serde(rename = "1")]
    Card,
    #[serde(rename = "2")]
    SBP,
}

#[derive(Debug, Serialize)]
pub struct AutosalesPlatformInitializeOrderRequest {
    pub amount: i64,
    pub id_pay_method: AutosalesPlatformPaymentMethod,
}

#[derive(Debug, Serialize)]
pub struct AutosalesPlatformSendReceiptRequest {
    pub object_token: String,
    pub url_file: String,
}

#[derive(Debug, Serialize)]
pub struct AutosalesPlatformObjectTokenPayload {
    pub object_token: String,
}

#[derive(Debug, Serialize)]
pub struct AutosalesPlatformRequest<T: Serialize> {
    pub version: String,
    pub user_token: String,
    #[serde(flatten)]
    pub payload: T,
}
