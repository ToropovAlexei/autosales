use std::sync::Arc;

use async_trait::async_trait;
use reqwest::Response;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal_macros::dec;
use serde::{Deserialize, de::DeserializeOwned};
use shared_dtos::{
    audit_log::{AuditAction, AuditStatus},
    bot::BotType,
};

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
        audit_log::NewAuditLog,
        bot::{BotListQuery, BotRow, NewBot, UpdateBot},
        common::PaginatedResult,
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
    pub referral_percentage: Option<Decimal>,
    pub ctx: Option<RequestContext>,
}

#[async_trait]
pub trait BotServiceTrait: Send + Sync {
    async fn get_list(&self, query: BotListQuery) -> ApiResult<PaginatedResult<BotRow>>;
    async fn create(&self, command: CreateBotCommand) -> ApiResult<BotRow>;
    async fn get_by_id(&self, id: i64) -> ApiResult<BotRow>;
    async fn update(&self, command: UpdateBotCommand) -> ApiResult<BotRow>;
    async fn can_operate(&self) -> ApiResult<bool>;
    async fn get_primary_bots(&self) -> ApiResult<Vec<BotRow>>;
    async fn delete(&self, id: i64) -> ApiResult<()>;
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
            BotType::Main => dec!(0),
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
            .get_last()
            .await
            .map(|t| t.store_balance_after.to_i64().unwrap_or_default() > 1000)
            .unwrap_or_default())
    }

    async fn get_primary_bots(&self) -> ApiResult<Vec<BotRow>> {
        self.bot_repo
            .get_primary_bots()
            .await
            .map_err(ApiError::from)
    }

    async fn delete(&self, id: i64) -> ApiResult<()> {
        self.bot_repo.delete(id).await.map_err(ApiError::from)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::repositories::{
        audit_log::AuditLogRepository, bot::BotRepository, settings::SettingsRepository,
        transaction::TransactionRepository,
    };
    use rust_decimal::Decimal;
    use sqlx::PgPool;
    use std::sync::Arc;

    async fn create_customer(pool: &PgPool, telegram_id: i64) -> i64 {
        sqlx::query_scalar!(
            "INSERT INTO customers (telegram_id, registered_with_bot, last_seen_with_bot) VALUES ($1, 1, 1) RETURNING id",
            telegram_id
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn create_bot(
        pool: &PgPool,
        owner_id: Option<i64>,
        token: &str,
        username: &str,
    ) -> BotRow {
        sqlx::query_as!(
            BotRow,
            r#"
            INSERT INTO bots (
                owner_id, token, username, type, is_active, is_primary, referral_percentage, created_by
            )
            VALUES ($1, $2, $3, 'main', true, false, 0.0, NULL)
            RETURNING
                id, owner_id, token, username, type as "type: _", is_active,
                is_primary, referral_percentage, created_at, updated_at, created_by
            "#,
            owner_id,
            token,
            username
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    fn build_service(
        pool: &PgPool,
    ) -> BotService<
        BotRepository,
        SettingsRepository,
        AuditLogService<AuditLogRepository>,
        TransactionRepository,
    > {
        let pool = Arc::new(pool.clone());
        let audit_log_service = Arc::new(AuditLogService::new(Arc::new(AuditLogRepository::new(
            pool.clone(),
        ))));
        BotService::new(
            Arc::new(BotRepository::new(pool.clone())),
            Arc::new(SettingsRepository::new(pool.clone())),
            Arc::new(TransactionRepository::new(pool.clone())),
            audit_log_service,
            Arc::new(reqwest::Client::new()),
        )
    }

    #[sqlx::test]
    async fn test_update_sets_primary_for_owner(pool: PgPool) {
        let service = build_service(&pool);
        let owner_id = create_customer(&pool, 10001).await;

        let bot1 = create_bot(&pool, Some(owner_id), "primary_token_1", "primary_bot_1").await;
        let bot2 = create_bot(&pool, Some(owner_id), "primary_token_2", "primary_bot_2").await;

        let updated = service
            .update(UpdateBotCommand {
                id: bot2.id,
                updated_by: None,
                username: None,
                is_active: None,
                is_primary: Some(true),
                referral_percentage: None,
                ctx: None,
            })
            .await
            .unwrap();

        assert_eq!(updated.id, bot2.id);
        assert!(updated.is_primary);

        let primary_ids: Vec<i64> = sqlx::query_scalar!(
            "SELECT id FROM bots WHERE owner_id = $1 AND is_primary = true",
            owner_id
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(primary_ids, vec![bot2.id]);

        let bot1_after = sqlx::query_scalar!("SELECT is_primary FROM bots WHERE id = $1", bot1.id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert!(!bot1_after);
    }

    #[sqlx::test]
    async fn test_delete_sets_deleted_at(pool: PgPool) {
        let service = build_service(&pool);
        let owner_id = create_customer(&pool, 10002).await;
        let bot = create_bot(&pool, Some(owner_id), "delete_token", "delete_bot").await;

        service.delete(bot.id).await.unwrap();

        let deleted_at = sqlx::query_scalar!("SELECT deleted_at FROM bots WHERE id = $1", bot.id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert!(deleted_at.is_some());
    }

    #[sqlx::test]
    async fn test_can_operate_based_on_store_balance(pool: PgPool) {
        let service = build_service(&pool);

        let can_operate_empty = service.can_operate().await.unwrap();
        assert!(!can_operate_empty);

        // create a transaction with store_balance_after > 1000
        let transaction_repo = TransactionRepository::new(Arc::new(pool.clone()));
        transaction_repo
            .create(crate::models::transaction::NewTransaction {
                customer_id: None,
                order_id: None,
                r#type: crate::models::transaction::TransactionType::Deposit,
                amount: Decimal::from(1200),
                store_balance_delta: Decimal::from(1200),
                platform_commission: Decimal::from(0),
                gateway_commission: Decimal::from(0),
                description: None,
                payment_gateway: None,
                details: None,
                bot_id: None,
            })
            .await
            .unwrap();

        let can_operate = service.can_operate().await.unwrap();
        assert!(can_operate);
    }
}
