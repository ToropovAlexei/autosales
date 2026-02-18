use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use rust_decimal::{Decimal, prelude::ToPrimitive};
use rust_decimal_macros::dec;
use shared_dtos::{
    audit_log::{AuditAction, AuditStatus},
    balance_request::{StoreBalanceRequestStatus, StoreBalanceRequestType},
    notification::DispatchAdminMessage,
    transaction::TransactionType,
};

use crate::{
    errors::api::{ApiError, ApiResult},
    infrastructure::repositories::{
        audit_log::AuditLogRepository,
        settings::{SettingsRepository, SettingsRepositoryTrait},
        store_balance_request::{
            StoreBalanceRequestRepository, StoreBalanceRequestRepositoryTrait,
        },
        transaction::TransactionRepository,
    },
    middlewares::context::RequestContext,
    models::{
        audit_log::NewAuditLog,
        common::PaginatedResult,
        store_balance::{
            NewStoreBalanceRequest, StoreBalanceRequestListQuery, StoreBalanceRequestRow,
            UpdateStoreBalanceRequest,
        },
        transaction::NewTransaction,
    },
    services::{
        audit_log::{AuditLogService, AuditLogServiceTrait},
        notification_service::{NotificationService, NotificationServiceTrait},
        transaction::{TransactionService, TransactionServiceTrait},
    },
};

pub struct CreateStoreBalanceRequestCommand {
    pub request_type: StoreBalanceRequestType,
    pub wallet_address: String,
    pub amount_rub: Decimal,
    pub admin_user_id: i64,
    pub ctx: RequestContext,
}

pub struct CompleteStoreBalanceRequestCommand {
    pub id: i64,
    pub tg_user_id: i64,
}

pub struct RejectStoreBalanceRequestCommand {
    pub id: i64,
    pub tg_user_id: i64,
}

#[async_trait]
pub trait StoreBalanceRequestServiceTrait: Send + Sync {
    async fn create(
        &self,
        cmd: CreateStoreBalanceRequestCommand,
    ) -> ApiResult<StoreBalanceRequestRow>;
    async fn get_list(
        &self,
        query: StoreBalanceRequestListQuery,
    ) -> ApiResult<PaginatedResult<StoreBalanceRequestRow>>;
    async fn complete(
        &self,
        cmd: CompleteStoreBalanceRequestCommand,
    ) -> ApiResult<StoreBalanceRequestRow>;
    async fn reject(
        &self,
        cmd: RejectStoreBalanceRequestCommand,
    ) -> ApiResult<StoreBalanceRequestRow>;
}

pub struct StoreBalanceRequestService<R, S, A, T, N> {
    repo: Arc<R>,
    settings_repo: Arc<S>,
    audit_log_service: Arc<A>,
    transaction_service: Arc<T>,
    notification_service: Arc<N>,
}

impl<R, S, A, T, N> StoreBalanceRequestService<R, S, A, T, N>
where
    R: StoreBalanceRequestRepositoryTrait + Send + Sync,
    S: SettingsRepositoryTrait + Send + Sync,
    A: AuditLogServiceTrait + Send + Sync,
    T: TransactionServiceTrait + Send + Sync,
    N: NotificationServiceTrait + Send + Sync,
{
    pub fn new(
        repo: Arc<R>,
        settings_repo: Arc<S>,
        audit_log_service: Arc<A>,
        transaction_service: Arc<T>,
        notification_service: Arc<N>,
    ) -> Self {
        Self {
            repo,
            settings_repo,
            audit_log_service,
            transaction_service,
            notification_service,
        }
    }

    fn validate_wallet_address(wallet_address: &str) -> ApiResult<()> {
        let valid_len = wallet_address.len() == 34;
        let valid_prefix = wallet_address.starts_with('T');
        let valid_base58 = wallet_address
            .chars()
            .all(|c| "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz".contains(c));

        if valid_len && valid_prefix && valid_base58 {
            return Ok(());
        }

        Err(ApiError::BadRequest(
            "Invalid USDT TRC20 wallet address format".to_string(),
        ))
    }
}

#[async_trait]
impl StoreBalanceRequestServiceTrait
    for StoreBalanceRequestService<
        StoreBalanceRequestRepository,
        SettingsRepository,
        AuditLogService<AuditLogRepository>,
        TransactionService<TransactionRepository>,
        NotificationService,
    >
{
    async fn create(
        &self,
        cmd: CreateStoreBalanceRequestCommand,
    ) -> ApiResult<StoreBalanceRequestRow> {
        Self::validate_wallet_address(cmd.wallet_address.as_str())?;
        let rate = self.settings_repo.load_settings().await?.usdt_rate_rub;
        if rate.is_zero() {
            return Err(ApiError::InternalServerError(
                "USDT rate is not set".to_string(),
            ));
        }
        if cmd.request_type == StoreBalanceRequestType::Withdrawal {
            let current_balance = match self.transaction_service.get_last().await {
                Ok(tx) => tx.store_balance_after,
                Err(ApiError::NotFound(_)) => Decimal::ZERO,
                Err(err) => return Err(err),
            };

            if cmd.amount_rub > current_balance {
                return Err(ApiError::BadRequest("Not enough balance".to_string()));
            }
        }

        let debit_transaction_id = {
            if cmd.request_type == StoreBalanceRequestType::Withdrawal {
                let new_transaction = self
                    .transaction_service
                    .create(NewTransaction {
                        amount: cmd.amount_rub,
                        store_balance_delta: -cmd.amount_rub,
                        bot_id: None,
                        customer_id: None,
                        description: None,
                        details: None,
                        gateway_commission: dec!(0),
                        order_id: None,
                        payment_gateway: None,
                        platform_commission: dec!(0),
                        r#type: TransactionType::BalanceRequestWithdrawalDebit,
                    })
                    .await?;
                Some(new_transaction.id)
            } else {
                None
            }
        };

        let amount_usdt = cmd.amount_rub / rate;

        let request = self
            .repo
            .create(NewStoreBalanceRequest {
                amount_rub: cmd.amount_rub,
                request_type: cmd.request_type,
                wallet_address: cmd.wallet_address,
                amount_usdt,
                fx_rate_rub_to_usdt: rate,
                status: StoreBalanceRequestStatus::PendingOperator,
                debit_transaction_id,
            })
            .await?;

        self.notification_service
            .dispatch_admin_message(DispatchAdminMessage::StoreBalanceRequestNotification {
                store_balance_request_id: request.id,
                amount_in_rub: cmd.amount_rub.to_f64().unwrap_or_default(),
                amount_in_usdt: amount_usdt.to_f64().unwrap_or_default(),
                r#type: cmd.request_type,
            })
            .await?;

        self.audit_log_service
            .create(NewAuditLog {
                action: match cmd.request_type {
                    StoreBalanceRequestType::Deposit => AuditAction::BalanceDeposit,
                    StoreBalanceRequestType::Withdrawal => AuditAction::BalanceWithdrawal,
                },
                admin_user_id: Some(cmd.admin_user_id),
                customer_id: None,
                ip_address: cmd.ctx.ip_address,
                user_agent: cmd.ctx.user_agent,
                request_id: Some(cmd.ctx.request_id),
                error_message: None,
                new_values: serde_json::to_value(request.clone()).ok(),
                old_values: None,
                status: AuditStatus::Success,
                target_id: request.id.to_string(),
                target_table: "store_balance_requests".to_string(),
            })
            .await?;

        Ok(request)
    }

    async fn get_list(
        &self,
        query: StoreBalanceRequestListQuery,
    ) -> ApiResult<PaginatedResult<StoreBalanceRequestRow>> {
        self.repo.get_list(query).await.map_err(ApiError::from)
    }

    async fn complete(
        &self,
        cmd: CompleteStoreBalanceRequestCommand,
    ) -> ApiResult<StoreBalanceRequestRow> {
        let prev = self.repo.get_by_id(cmd.id).await?;
        if prev.status != StoreBalanceRequestStatus::PendingOperator {
            return Err(ApiError::BadRequest(
                "Request is not in pending state".to_string(),
            ));
        }
        let credit_transaction_id = {
            if prev.request_type == StoreBalanceRequestType::Deposit {
                Some(
                    self.transaction_service
                        .create(NewTransaction {
                            amount: prev.amount_rub,
                            store_balance_delta: prev.amount_rub,
                            bot_id: None,
                            customer_id: None,
                            description: None,
                            details: None,
                            gateway_commission: dec!(0),
                            order_id: None,
                            payment_gateway: None,
                            platform_commission: dec!(0),
                            r#type: TransactionType::BalanceRequestDepositCredit,
                        })
                        .await?
                        .id,
                )
            } else {
                None
            }
        };

        let updated = self
            .repo
            .update(
                cmd.id,
                UpdateStoreBalanceRequest {
                    operator_tg_user_id: Some(cmd.tg_user_id),
                    status: Some(StoreBalanceRequestStatus::Completed),
                    operator_action_at: Some(Utc::now()),
                    credit_transaction_id,
                    ..Default::default()
                },
            )
            .await?;

        Ok(updated)
    }

    async fn reject(
        &self,
        cmd: RejectStoreBalanceRequestCommand,
    ) -> ApiResult<StoreBalanceRequestRow> {
        let prev = self.repo.get_by_id(cmd.id).await?;
        if prev.status != StoreBalanceRequestStatus::PendingOperator {
            return Err(ApiError::BadRequest(
                "Request is not in pending state".to_string(),
            ));
        }

        let refund_transaction_id = {
            if prev.request_type == StoreBalanceRequestType::Withdrawal {
                Some(
                    self.transaction_service
                        .create(NewTransaction {
                            amount: prev.amount_rub,
                            store_balance_delta: prev.amount_rub,
                            bot_id: None,
                            customer_id: None,
                            description: None,
                            details: None,
                            gateway_commission: dec!(0),
                            order_id: None,
                            payment_gateway: None,
                            platform_commission: dec!(0),
                            r#type: TransactionType::BalanceRequestWithdrawalRefund,
                        })
                        .await?
                        .id,
                )
            } else {
                None
            }
        };

        let updated = self
            .repo
            .update(
                cmd.id,
                UpdateStoreBalanceRequest {
                    operator_tg_user_id: Some(cmd.tg_user_id),
                    status: Some(StoreBalanceRequestStatus::Rejected),
                    operator_action_at: Some(Utc::now()),
                    refund_transaction_id,
                    ..Default::default()
                },
            )
            .await?;

        Ok(updated)
    }
}
