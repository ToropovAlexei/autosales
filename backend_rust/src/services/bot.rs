use std::{marker::PhantomData, sync::Arc};

use async_trait::async_trait;
use bigdecimal::{BigDecimal, ToPrimitive, Zero};
use reqwest::Response;
use serde::{Deserialize, de::DeserializeOwned};

use crate::{
    errors::api::{ApiError, ApiResult},
    infrastructure::repositories::{
        audit_log::AuditLogRepository,
        bot::{BotRepository, BotRepositoryTrait},
        settings::{SettingsRepository, SettingsRepositoryTrait},
        transaction::{TransactionRepository, TransactionRepositoryTrait},
    },
    middlewares::context::RequestContext,
    models::{
        audit_log::{AuditAction, AuditStatus, NewAuditLog},
        bot::{BotListQuery, BotRow, BotType, NewBot, UpdateBot},
        common::{OrderDir, PaginatedResult, Pagination},
        transaction::{TransactionListQuery, TransactionOrderFields},
    },
    services::audit_log::{AuditLogService, AuditLogServiceTrait},
};

#[derive(Debug, Deserialize)]
struct GetMeResponse {
    result: GetMeResult,
}

#[derive(Debug, Deserialize)]
struct GetMeResult {
    username: String,
}

#[derive(Debug)]
pub struct CreateBotCommand {
    pub owner_id: Option<i64>,
    pub token: String,
    pub r#type: BotType,
    pub is_active: bool,
    pub is_primary: bool,
    pub created_by: Option<i64>,
    pub ctx: Option<RequestContext>,
}

#[derive(Debug)]
pub struct UpdateBotCommand {
    pub id: i64,
    pub updated_by: Option<i64>,
    pub username: Option<String>,
    pub is_active: Option<bool>,
    pub is_primary: Option<bool>,
    pub referral_percentage: Option<BigDecimal>,
    pub ctx: Option<RequestContext>,
}

#[async_trait]
pub trait BotServiceTrait: Send + Sync {
    async fn get_list(&self, query: BotListQuery) -> ApiResult<PaginatedResult<BotRow>>;
    async fn create(&self, command: CreateBotCommand) -> ApiResult<BotRow>;
    async fn get_by_id(&self, id: i64) -> ApiResult<BotRow>;
    async fn update(&self, command: UpdateBotCommand) -> ApiResult<BotRow>;
    async fn can_operate(&self) -> ApiResult<bool>;
}

pub struct BotService<R, S, A, T> {
    bot_repo: Arc<R>,
    settings_repo: Arc<S>,
    transaction_repo: Arc<T>,
    audit_log_service: Arc<A>,
    client: Arc<reqwest::Client>,
}

impl<R, S, A, T> BotService<R, S, A, T>
where
    R: BotRepositoryTrait + Send + Sync,
    S: SettingsRepositoryTrait + Send + Sync,
    A: AuditLogServiceTrait + Send + Sync,
    T: TransactionRepositoryTrait + Send + Sync,
{
    pub fn new(
        bot_repo: Arc<R>,
        settings_repo: Arc<S>,
        transaction_repo: Arc<T>,
        audit_log_service: Arc<A>,
        client: Arc<reqwest::Client>,
    ) -> Self {
        Self {
            bot_repo,
            settings_repo,
            transaction_repo,
            audit_log_service,
            client,
        }
    }
}

#[async_trait]
impl BotServiceTrait
    for BotService<
        BotRepository,
        SettingsRepository,
        AuditLogService<AuditLogRepository>,
        TransactionRepository,
    >
{
    async fn get_list(&self, query: BotListQuery) -> ApiResult<PaginatedResult<BotRow>> {
        self.bot_repo.get_list(query).await.map_err(ApiError::from)
    }

    async fn create(&self, command: CreateBotCommand) -> ApiResult<BotRow> {
        let referral_percentage = match command.r#type {
            BotType::Referral => self
                .settings_repo
                .load_settings()
                .await
                .map(|s| s.referral_percentage)?,
            BotType::Main => BigDecimal::zero(),
        };

        let created = self
            .bot_repo
            .create(NewBot {
                created_by: command.created_by,
                owner_id: command.owner_id,
                username: get_bot_name(&command.token, &self.client).await?,
                // TODO: Hash token?
                token: command.token,
                r#type: command.r#type,
                is_active: command.is_active,
                is_primary: command.is_primary,
                referral_percentage,
            })
            .await?;

        self.audit_log_service
            .create(NewAuditLog {
                action: AuditAction::BotCreate,
                status: AuditStatus::Success,
                admin_user_id: command.created_by,
                customer_id: None,
                error_message: None,
                new_values: serde_json::to_value(created.clone()).ok(),
                old_values: None,
                target_id: created.id.to_string(),
                target_table: "bots".to_string(),
                ip_address: command.ctx.clone().and_then(|ctx| ctx.ip_address),
                request_id: command.ctx.clone().map(|ctx| ctx.request_id),
                user_agent: command.ctx.and_then(|ctx| ctx.user_agent),
            })
            .await?;

        Ok(created)
    }

    async fn get_by_id(&self, id: i64) -> ApiResult<BotRow> {
        let res = self.bot_repo.get_by_id(id).await?;
        Ok(res)
    }

    async fn update(&self, command: UpdateBotCommand) -> ApiResult<BotRow> {
        let prev = self.bot_repo.get_by_id(command.id).await?;
        let updated = self
            .bot_repo
            .update(
                command.id,
                UpdateBot {
                    is_active: command.is_active,
                    is_primary: command.is_primary,
                    referral_percentage: command.referral_percentage,
                    username: command.username,
                },
            )
            .await?;

        if let Some(is_primary) = command.is_primary
            && is_primary
        {
            self.bot_repo
                .set_primary_bot_for_owner(command.id, prev.owner_id)
                .await?;
        }

        self.audit_log_service
            .create(NewAuditLog {
                action: AuditAction::BotUpdate,
                status: AuditStatus::Success,
                admin_user_id: command.updated_by,
                customer_id: None,
                error_message: None,
                new_values: serde_json::to_value(updated.clone()).ok(),
                old_values: serde_json::to_value(prev.clone()).ok(),
                target_id: prev.id.to_string(),
                target_table: "bots".to_string(),
                ip_address: command.ctx.clone().and_then(|ctx| ctx.ip_address),
                request_id: command.ctx.clone().map(|ctx| ctx.request_id),
                user_agent: command.ctx.and_then(|ctx| ctx.user_agent),
            })
            .await?;

        Ok(updated)
    }

    async fn can_operate(&self) -> ApiResult<bool> {
        Ok(self
            .transaction_repo
            .get_list(TransactionListQuery {
                filters: vec![],
                pagination: Pagination {
                    page: 1,
                    page_size: 1,
                },
                order_by: Some(TransactionOrderFields::Id),
                order_dir: OrderDir::Desc,
                _phantom: PhantomData,
            })
            .await?
            .items
            .last()
            .is_some_and(|t| t.store_balance_after.to_i64().unwrap_or_default() > 1000))
    }
}

pub async fn get_bot_name(token: &str, client: &Arc<reqwest::Client>) -> ApiResult<String> {
    parse_response::<GetMeResponse>(
        client
            .get(format!("https://api.telegram.org/bot{token}/getMe"))
            .send()
            .await
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?,
    )
    .await
    .map(|r| r.result.username)
    .ok_or_else(|| ApiError::InternalServerError("Error getting bot name".to_string()))
}

async fn parse_response<T>(response: Response) -> Option<T>
where
    T: DeserializeOwned,
{
    let status = response.status();
    if let Ok(body) = response.text().await
        && status.is_success()
        && let Ok(parsed) = serde_json::from_str::<T>(&body)
    {
        return Some(parsed);
    }
    None
}
