use std::sync::Arc;

use async_trait::async_trait;
use rust_decimal::Decimal;
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
    pub pricing_global_markup: Option<Decimal>,
    pub pricing_platform_commission: Option<Decimal>,
    pub pricing_gateway_markup: Option<Decimal>,
    pub pricing_gateway_bonus_mock_provider: Option<Decimal>,
    pub pricing_gateway_bonus_platform_card: Option<Decimal>,
    pub pricing_gateway_bonus_platform_sbp: Option<Decimal>,
    pub referral_program_enabled: Option<bool>,
    pub referral_percentage: Option<Decimal>,
    pub bot_payment_system_support_operators: Option<Vec<String>>,
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
            bot_payment_system_support_operators: r.bot_payment_system_support_operators.map(
                |operators| {
                    operators
                        .iter()
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<String>>()
                },
            ),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::repositories::{
        audit_log::AuditLogRepository, settings::SettingsRepository,
    };
    use crate::services::audit_log::AuditLogService;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use sqlx::PgPool;
    use std::net::IpAddr;
    use std::str::FromStr;
    use std::sync::Arc;
    use uuid::Uuid;

    fn build_service(
        pool: &PgPool,
    ) -> SettingsService<SettingsRepository, AuditLogService<AuditLogRepository>> {
        let pool = Arc::new(pool.clone());
        let audit_log_service = Arc::new(AuditLogService::new(Arc::new(AuditLogRepository::new(
            pool.clone(),
        ))));
        SettingsService::new(Arc::new(SettingsRepository::new(pool)), audit_log_service)
    }

    #[sqlx::test]
    async fn test_update_writes_audit_log(pool: PgPool) {
        let service = build_service(&pool);
        let request_id = Uuid::new_v4();

        let updated = service
            .update(
                UpdateSettingsCommand {
                    updated_by: 1,
                    pricing_global_markup: Some(dec!(5.0)),
                    referral_program_enabled: Some(true),
                    referral_percentage: Some(Decimal::from(12)),
                    ..Default::default()
                },
                RequestContext {
                    ip_address: Some(IpAddr::from_str("127.0.0.1").unwrap()),
                    user_agent: Some("test-agent".to_string()),
                    request_id,
                },
            )
            .await
            .unwrap();

        assert_eq!(updated.pricing_global_markup, dec!(5.0));
        assert!(updated.referral_program_enabled);
        assert_eq!(updated.referral_percentage, Decimal::from(12));

        let audit_row = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM audit_logs WHERE action = 'system_settings_update'"
        )
        .fetch_one(&pool)
        .await
        .unwrap()
        .unwrap();
        assert!(audit_row >= 1);
    }

    #[sqlx::test]
    async fn test_load_settings(pool: PgPool) {
        let service = build_service(&pool);
        let settings = service.load_settings().await.unwrap();
        assert_eq!(settings.pricing_global_markup, dec!(0));
    }
}
