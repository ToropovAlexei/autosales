use rust_decimal::prelude::ToPrimitive;
use shared_dtos::balance_request::StoreBalanceRequestAdminResponse;

use crate::models::store_balance::StoreBalanceRequestRow;

impl From<StoreBalanceRequestRow> for StoreBalanceRequestAdminResponse {
    fn from(row: StoreBalanceRequestRow) -> Self {
        Self {
            id: row.id,
            request_type: row.request_type,
            wallet_address: row.wallet_address,
            amount_usdt: row.amount_usdt.to_f64().unwrap_or_default(),
            fx_rate_rub_to_usdt: row.fx_rate_rub_to_usdt.to_f64().unwrap_or_default(),
            amount_rub: row.amount_rub.to_f64().unwrap_or_default(),
            status: row.status,
            operator_tg_user_id: row.operator_tg_user_id,
            operator_comment: row.operator_comment,
            operator_action_at: row.operator_action_at,
            telegram_message_id: row.telegram_message_id,
            telegram_chat_id: row.telegram_chat_id,
            debit_transaction_id: row.debit_transaction_id,
            credit_transaction_id: row.credit_transaction_id,
            refund_transaction_id: row.refund_transaction_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
