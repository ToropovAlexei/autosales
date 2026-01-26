use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MockProviderInvoiceStatus {
    Failed,
    Pending,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MockProviderCreateInvoiceRequest {
    pub amount: f64,
    pub user_id: i64,
    pub order_id: Uuid,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MockProviderCreateInvoiceResponse {
    pub invoice_id: Uuid,
    pub pay_url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct MockProviderInvoiceStatusResponse {
    pub status: MockProviderInvoiceStatus,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct MockProviderInvoiceWebhookPayload {
    pub event: String,
    pub order_id: Uuid,
    pub invoice_id: Uuid,
    pub amount: f64,
    pub status: MockProviderInvoiceStatus,
}
