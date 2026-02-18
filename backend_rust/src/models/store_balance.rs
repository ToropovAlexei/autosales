use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use shared_dtos::balance_request::{StoreBalanceRequestStatus, StoreBalanceRequestType};
use sqlx::prelude::FromRow;

use crate::define_list_query;

#[derive(FromRow, Debug, Clone, Serialize)]
pub struct StoreBalanceRequestRow {
    pub id: i64,
    pub request_type: StoreBalanceRequestType,
    pub wallet_address: String,
    pub amount_usdt: Decimal,
    pub fx_rate_rub_to_usdt: Decimal,
    pub amount_rub: Decimal,
    pub status: StoreBalanceRequestStatus,
    pub operator_tg_user_id: Option<i64>,
    pub operator_comment: Option<String>,
    pub operator_action_at: Option<DateTime<Utc>>,
    pub telegram_message_id: Option<i64>,
    pub telegram_chat_id: Option<i64>,
    pub debit_transaction_id: Option<i64>,
    pub credit_transaction_id: Option<i64>,
    pub refund_transaction_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct NewStoreBalanceRequest {
    pub request_type: StoreBalanceRequestType,
    pub wallet_address: String,
    pub amount_usdt: Decimal,
    pub fx_rate_rub_to_usdt: Decimal,
    pub amount_rub: Decimal,
    pub status: StoreBalanceRequestStatus,
    pub debit_transaction_id: Option<i64>,
}

#[derive(Debug, Default)]
pub struct UpdateStoreBalanceRequest {
    pub status: Option<StoreBalanceRequestStatus>,
    pub operator_tg_user_id: Option<i64>,
    pub operator_comment: Option<String>,
    pub operator_action_at: Option<DateTime<Utc>>,
    pub telegram_message_id: Option<i64>,
    pub telegram_chat_id: Option<i64>,
    pub debit_transaction_id: Option<i64>,
    pub credit_transaction_id: Option<i64>,
    pub refund_transaction_id: Option<i64>,
}

define_list_query! {
    query_name: StoreBalanceRequestListQuery,
    filter_fields: {
        StoreBalanceRequestFilterFields,
        [
            Id => "id",
            RequestType => "request_type",
            Status => "status",
            CreatedAt => "created_at",
            UpdatedAt => "updated_at",
        ]
    },
    order_fields: {
        StoreBalanceRequestOrderFields,
        [
            Id => "id",
            RequestType => "request_type",
            Status => "status",
            CreatedAt => "created_at",
            UpdatedAt => "updated_at",
        ]
    }
}
