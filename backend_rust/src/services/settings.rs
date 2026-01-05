use std::sync::Arc;

use async_trait::async_trait;
use bigdecimal::BigDecimal;
use uuid::Uuid;

use crate::{
    errors::api::{ApiError, ApiResult},
    infrastructure::repositories::{
        audit_log::AuditLogRepository,
        settings::{SettingsRepository, SettingsRepositoryTrait},
    },
    middlewares::context::RequestContext,
    models::{
        audit_log::{AuditAction, AuditStatus, NewAuditLog},
        settings::{Settings, UpdateSettings},
    },
    services::audit_log::{AuditLogService, AuditLogServiceTrait},
};

#[derive(Debug, Default)]
pub struct UpdateSettingsCommand {
    pub updated_by: i64,
    pub bot_messages_support: Option<String>,
    pub bot_messages_support_image_id: Option<Option<Uuid>>,
    pub bot_messages_new_user_welcome: Option<String>,
    pub bot_messages_new_user_welcome_image_id: Option<Option<Uuid>>,
    pub bot_messages_returning_user_welcome: Option<String>,
    pub bot_messages_returning_user_welcome_image_id: Option<Option<Uuid>>,
    pub pricing_global_markup: Option<BigDecimal>,
    pub pricing_platform_commission: Option<BigDecimal>,
    pub pricing_gateway_markup: Option<BigDecimal>,
    pub pricing_gateway_bonus_mock_provider: Option<BigDecimal>,
    pub pricing_gateway_bonus_platform_card: Option<BigDecimal>,
    pub pricing_gateway_bonus_platform_sbp: Option<BigDecimal>,
    pub referral_program_enabled: Option<bool>,
    pub referral_percentage: Option<BigDecimal>,
}

impl From<UpdateSettingsCommand> for UpdateSettings {
    fn from(r: UpdateSettingsCommand) -> Self {
        UpdateSettings {
            bot_messages_support: r.bot_messages_support,
            bot_messages_support_image_id: r.bot_messages_support_image_id,
            bot_messages_new_user_welcome: r.bot_messages_new_user_welcome,
            bot_messages_new_user_welcome_image_id: r.bot_messages_new_user_welcome_image_id,
            bot_messages_returning_user_welcome: r.bot_messages_returning_user_welcome,
            bot_messages_returning_user_welcome_image_id: r
                .bot_messages_returning_user_welcome_image_id,
            pricing_global_markup: r.pricing_global_markup,
            pricing_platform_commission: r.pricing_platform_commission,
            pricing_gateway_markup: r.pricing_gateway_markup,
            pricing_gateway_bonus_mock_provider: r.pricing_gateway_bonus_mock_provider,
            pricing_gateway_bonus_platform_card: r.pricing_gateway_bonus_platform_card,
            pricing_gateway_bonus_platform_sbp: r.pricing_gateway_bonus_platform_sbp,
            referral_program_enabled: r.referral_program_enabled,
            referral_percentage: r.referral_percentage,
        }
    }
}

#[async_trait]
pub trait SettingsServiceTrait: Send + Sync {
    async fn load_settings(&self) -> ApiResult<Settings>;
    async fn update(
        &self,
        settings: UpdateSettingsCommand,
        ctx: RequestContext,
    ) -> ApiResult<Settings>;
}

pub struct SettingsService<R, A> {
    repo: Arc<R>,
    audit_log_service: Arc<A>,
}

impl<R, A> SettingsService<R, A>
where
    R: SettingsRepositoryTrait + Send + Sync,
    A: AuditLogServiceTrait + Send + Sync,
{
    pub fn new(repo: Arc<R>, audit_log_service: Arc<A>) -> Self {
        Self {
            repo,
            audit_log_service,
        }
    }
}

#[async_trait]
impl SettingsServiceTrait
    for SettingsService<SettingsRepository, AuditLogService<AuditLogRepository>>
{
    async fn load_settings(&self) -> ApiResult<Settings> {
        self.repo.load_settings().await.map_err(ApiError::from)
    }

    async fn update(
        &self,
        command: UpdateSettingsCommand,
        ctx: RequestContext,
    ) -> ApiResult<Settings> {
        let updated_by = command.updated_by;
        let prev = self.repo.load_settings().await?;
        let updated = self.repo.update(UpdateSettings::from(command)).await?;

        self.audit_log_service
            .create(NewAuditLog {
                action: AuditAction::SystemSettingsUpdate,
                status: AuditStatus::Success,
                admin_user_id: Some(updated_by),
                customer_id: None,
                error_message: None,
                ip_address: ctx.ip_address,
                new_values: serde_json::to_value(updated.clone()).ok(),
                old_values: serde_json::to_value(prev.clone()).ok(),
                request_id: Some(ctx.request_id),
                target_id: "".to_string(),
                target_table: "settings".to_string(),
                user_agent: ctx.user_agent.clone(),
            })
            .await?;

        Ok(updated)
    }
}
