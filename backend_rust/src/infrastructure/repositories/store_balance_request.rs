use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{PgPool, QueryBuilder};

use crate::{
    errors::repository::{RepositoryError, RepositoryResult},
    infrastructure::lib::query::{apply_filters, apply_list_query},
    models::{
        common::PaginatedResult,
        store_balance::{
            NewStoreBalanceRequest, StoreBalanceRequestListQuery, StoreBalanceRequestRow,
            UpdateStoreBalanceRequest,
        },
    },
};

#[async_trait]
pub trait StoreBalanceRequestRepositoryTrait {
    async fn get_list(
        &self,
        query: StoreBalanceRequestListQuery,
    ) -> RepositoryResult<PaginatedResult<StoreBalanceRequestRow>>;
    async fn get_by_id(&self, id: i64) -> RepositoryResult<StoreBalanceRequestRow>;
    async fn create(
        &self,
        new_request: NewStoreBalanceRequest,
    ) -> RepositoryResult<StoreBalanceRequestRow>;
    async fn update(
        &self,
        id: i64,
        update: UpdateStoreBalanceRequest,
    ) -> RepositoryResult<StoreBalanceRequestRow>;
}

#[derive(Clone)]
pub struct StoreBalanceRequestRepository {
    pool: Arc<PgPool>,
}

impl StoreBalanceRequestRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl StoreBalanceRequestRepositoryTrait for StoreBalanceRequestRepository {
    async fn get_list(
        &self,
        query: StoreBalanceRequestListQuery,
    ) -> RepositoryResult<PaginatedResult<StoreBalanceRequestRow>> {
        let mut count_qb = QueryBuilder::new("SELECT COUNT(*) FROM store_balance_requests");
        apply_filters(&mut count_qb, &query);

        let count_query = count_qb.build_query_scalar();
        let total: i64 = count_query.fetch_one(&*self.pool).await?;

        let mut query_builder = QueryBuilder::new(
            r#"
        SELECT * FROM store_balance_requests"#,
        );
        apply_list_query(&mut query_builder, &query);
        let query = query_builder.build_query_as::<StoreBalanceRequestRow>();
        let items = query.fetch_all(&*self.pool).await?;
        Ok(PaginatedResult { items, total })
    }

    async fn get_by_id(&self, id: i64) -> RepositoryResult<StoreBalanceRequestRow> {
        let result = sqlx::query_as!(
            StoreBalanceRequestRow,
            r#"
            SELECT
                id, request_type as "request_type: _", wallet_address, amount_usdt, fx_rate_rub_to_usdt, amount_rub,
                status as "status: _", operator_tg_user_id, operator_comment, operator_action_at,
                telegram_message_id, telegram_chat_id, debit_transaction_id, credit_transaction_id,
                refund_transaction_id, created_at, updated_at
            FROM store_balance_requests WHERE id = $1"#,
            id
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(result)
    }

    async fn create(
        &self,
        new_request: NewStoreBalanceRequest,
    ) -> RepositoryResult<StoreBalanceRequestRow> {
        let result = sqlx::query_as!(
            StoreBalanceRequestRow,
            r#"
            INSERT INTO store_balance_requests (
                request_type, wallet_address, amount_usdt, fx_rate_rub_to_usdt, amount_rub,
                status, debit_transaction_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING
                id, request_type as "request_type: _", wallet_address, amount_usdt, fx_rate_rub_to_usdt, amount_rub,
                status as "status: _", operator_tg_user_id, operator_comment, operator_action_at,
                telegram_message_id, telegram_chat_id, debit_transaction_id, credit_transaction_id,
                refund_transaction_id, created_at, updated_at
            "#,
            new_request.request_type as _,
            new_request.wallet_address,
            new_request.amount_usdt,
            new_request.fx_rate_rub_to_usdt,
            new_request.amount_rub,
            new_request.status as _,
            new_request.debit_transaction_id
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(result)
    }

    async fn update(
        &self,
        id: i64,
        update: UpdateStoreBalanceRequest,
    ) -> RepositoryResult<StoreBalanceRequestRow> {
        let mut query_builder =
            QueryBuilder::new("UPDATE store_balance_requests SET status = COALESCE(");

        query_builder.push_bind(update.status);
        query_builder.push(", status)");

        if let Some(credit_transaction_id) = update.credit_transaction_id {
            query_builder.push(", credit_transaction_id = ");
            query_builder.push_bind(credit_transaction_id);
        }

        if let Some(refund_transaction_id) = update.refund_transaction_id {
            query_builder.push(", refund_transaction_id = ");
            query_builder.push_bind(refund_transaction_id);
        }

        if let Some(debit_transaction_id) = update.debit_transaction_id {
            query_builder.push(", debit_transaction_id = ");
            query_builder.push_bind(debit_transaction_id);
        }

        if let Some(operator_tg_user_id) = update.operator_tg_user_id {
            query_builder.push(", operator_tg_user_id = ");
            query_builder.push_bind(operator_tg_user_id);
        }

        if let Some(operator_comment) = update.operator_comment {
            query_builder.push(", operator_comment = ");
            query_builder.push_bind(operator_comment);
        }

        if let Some(operator_action_at) = update.operator_action_at {
            query_builder.push(", operator_action_at = ");
            query_builder.push_bind(operator_action_at);
        }

        if let Some(telegram_message_id) = update.telegram_message_id {
            query_builder.push(", telegram_message_id = ");
            query_builder.push_bind(telegram_message_id);
        }

        if let Some(telegram_chat_id) = update.telegram_chat_id {
            query_builder.push(", telegram_chat_id = ");
            query_builder.push_bind(telegram_chat_id);
        }

        query_builder.push(" WHERE id = ");
        query_builder.push_bind(id);
        query_builder.push(" RETURNING *");

        let query = query_builder.build_query_as::<StoreBalanceRequestRow>();

        query
            .fetch_one(&*self.pool)
            .await
            .map_err(RepositoryError::from)
    }
}
