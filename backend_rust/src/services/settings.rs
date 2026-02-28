use std::sync::Arc;

use async_trait::async_trait;
use rust_decimal::Decimal;
use shared_dtos::audit_log::{AuditAction, AuditStatus};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    errors::api::{ApiError, ApiResult},
    infrastructure::repositories::settings::SettingsRepositoryTrait,
    middlewares::context::RequestContext,
    models::{
        audit_log::NewAuditLog,
        settings::{Settings, UpdateSettings},
    },
    services::audit_log::AuditLogServiceTrait,
};

#[derive(Debug, Default, Clone)]
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
    pub bot_description: Option<String>,
    pub bot_about: Option<String>,
    pub manager_group_chat_id: Option<Option<i64>>,
    pub usdt_rate_rub: Option<Decimal>,
    pub ctx: Option<RequestContext>,
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
            bot_about: r.bot_about,
            bot_description: r.bot_description,
            manager_group_chat_id: r.manager_group_chat_id,
            usdt_rate_rub: r.usdt_rate_rub,
        }
    }
}

#[async_trait]
pub trait SettingsServiceTrait: Send + Sync {
    async fn load_settings(&self) -> ApiResult<Settings>;
    async fn update(&self, cmd: UpdateSettingsCommand) -> ApiResult<Settings>;
}

pub struct SettingsService<R, A> {
    repo: Arc<R>,
    audit_log_service: Arc<A>,
    cache: RwLock<Option<Settings>>,
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
            cache: RwLock::new(None),
        }
    }
}

#[async_trait]
impl<R, A> SettingsServiceTrait for SettingsService<R, A>
where
    R: SettingsRepositoryTrait + Send + Sync,
    A: AuditLogServiceTrait + Send + Sync,
{
    async fn load_settings(&self) -> ApiResult<Settings> {
        if let Some(cached) = self.cache.read().await.clone() {
            return Ok(cached);
        }

        let loaded = self.repo.load_settings().await.map_err(ApiError::from)?;
        *self.cache.write().await = Some(loaded.clone());
        Ok(loaded)
    }

    async fn update(&self, cmd: UpdateSettingsCommand) -> ApiResult<Settings> {
        let updated_by = cmd.updated_by;
        let prev = self.load_settings().await?;
        let updated = self.repo.update(UpdateSettings::from(cmd.clone())).await?;
        *self.cache.write().await = Some(updated.clone());

        self.audit_log_service
            .create(NewAuditLog {
                action: AuditAction::SystemSettingsUpdate,
                status: AuditStatus::Success,
                admin_user_id: Some(updated_by),
                customer_id: None,
                error_message: None,
                new_values: serde_json::to_value(updated.clone()).ok(),
                old_values: serde_json::to_value(prev.clone()).ok(),
                target_id: "".to_string(),
                target_table: "settings".to_string(),
                ip_address: cmd.ctx.clone().and_then(|ctx| ctx.ip_address),
                request_id: cmd.ctx.clone().map(|ctx| ctx.request_id),
                user_agent: cmd.ctx.and_then(|ctx| ctx.user_agent),
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
    use crate::models::{
        audit_log::{AuditLogListQuery, AuditLogRow, NewAuditLog},
        common::PaginatedResult,
    };
    use crate::services::audit_log::AuditLogService;
    use async_trait::async_trait;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use sqlx::PgPool;
    use std::net::IpAddr;
    use std::str::FromStr;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
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
            .update(UpdateSettingsCommand {
                updated_by: 1,
                pricing_global_markup: Some(dec!(5.0)),
                referral_program_enabled: Some(true),
                referral_percentage: Some(Decimal::from(12)),
                ctx: Some(RequestContext {
                    ip_address: Some(IpAddr::from_str("127.0.0.1").unwrap()),
                    user_agent: Some("test-agent".to_string()),
                    request_id,
                }),
                ..Default::default()
            })
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

    struct MockSettingsRepo {
        load_calls: AtomicUsize,
        value: Settings,
    }

    #[async_trait]
    impl SettingsRepositoryTrait for MockSettingsRepo {
        async fn load_settings(&self) -> crate::errors::repository::RepositoryResult<Settings> {
            self.load_calls.fetch_add(1, Ordering::SeqCst);
            Ok(self.value.clone())
        }

        async fn update(
            &self,
            _update: UpdateSettings,
        ) -> crate::errors::repository::RepositoryResult<Settings> {
            Ok(self.value.clone())
        }
    }

    struct NoopAuditLogService;

    #[async_trait]
    impl AuditLogServiceTrait for NoopAuditLogService {
        async fn get_list(
            &self,
            _query: AuditLogListQuery,
        ) -> ApiResult<PaginatedResult<AuditLogRow>> {
            Ok(PaginatedResult {
                items: vec![],
                total: 0,
            })
        }

        async fn create(&self, _audit_log: NewAuditLog) -> ApiResult<AuditLogRow> {
            Err(ApiError::InternalServerError(
                "not used in this test".to_string(),
            ))
        }
    }

    #[tokio::test]
    async fn test_load_settings_uses_cache() {
        let repo = Arc::new(MockSettingsRepo {
            load_calls: AtomicUsize::new(0),
            value: Settings {
                pricing_global_markup: dec!(3.5),
                ..Default::default()
            },
        });
        let service = SettingsService::new(repo.clone(), Arc::new(NoopAuditLogService));

        let first = service.load_settings().await.unwrap();
        let second = service.load_settings().await.unwrap();

        assert_eq!(first.pricing_global_markup, dec!(3.5));
        assert_eq!(second.pricing_global_markup, dec!(3.5));
        assert_eq!(repo.load_calls.load(Ordering::SeqCst), 1);
    }
}
