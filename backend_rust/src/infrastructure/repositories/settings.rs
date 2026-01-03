use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use bigdecimal::{BigDecimal, Zero};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::repository::{RepositoryError, RepositoryResult},
    models::settings::{Settings, UpdateSettings},
};

#[async_trait]
pub trait SettingsRepositoryTrait {
    async fn load_settings(&self) -> RepositoryResult<Settings>;
    async fn update(&self, update: UpdateSettings) -> RepositoryResult<Settings>;
}

#[derive(Clone)]
pub struct SettingsRepository {
    pool: Arc<PgPool>,
}

impl SettingsRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SettingsRepositoryTrait for SettingsRepository {
    async fn load_settings(&self) -> RepositoryResult<Settings> {
        let rows = sqlx::query!("SELECT key, value FROM settings")
            .fetch_all(&*self.pool)
            .await?;

        let map: HashMap<String, Option<String>> = rows
            .iter()
            .map(|row| (row.key.clone(), row.value.clone()))
            .collect();

        Ok(Settings {
            bot_messages_support: get_string(&map, "bot_messages_support", "Служба поддержки"),
            bot_messages_support_image_id: get_uuid(&map, "bot_messages_support_image_id"),

            bot_messages_new_user_welcome: get_string(
                &map,
                "bot_messages_new_user_welcome",
                "Добро пожаловать, {username}!",
            ),
            bot_messages_new_user_welcome_image_id: get_uuid(
                &map,
                "bot_messages_new_user_welcome_image_id",
            ),

            bot_messages_returning_user_welcome: get_string(
                &map,
                "bot_messages_returning_user_welcome",
                "С возвращением, {username}!",
            ),
            bot_messages_returning_user_welcome_image_id: get_uuid(
                &map,
                "bot_messages_returning_user_welcome_image_id",
            ),

            pricing_global_markup: get_bigdecimal(
                &map,
                "pricing_global_markup",
                BigDecimal::zero(),
            ),
            pricing_platform_commission: get_bigdecimal(
                &map,
                "pricing_platform_commission",
                BigDecimal::zero(),
            ),

            pricing_gateway_commission: get_bigdecimal(
                &map,
                "pricing_gateway_commission",
                BigDecimal::zero(),
            ),

            pricing_gateway_bonus_mock_provider: get_bigdecimal(
                &map,
                "pricing_gateway_bonus_mock_provider",
                BigDecimal::zero(),
            ),
            pricing_gateway_bonus_platform_card: get_bigdecimal(
                &map,
                "pricing_gateway_bonus_platform_card",
                BigDecimal::zero(),
            ),
            pricing_gateway_bonus_platform_sbp: get_bigdecimal(
                &map,
                "pricing_gateway_bonus_platform_sbp",
                BigDecimal::zero(),
            ),

            referral_program_enabled: get_bool(&map, "referral_program_enabled", false),
            referral_percentage: get_bigdecimal(&map, "referral_percentage", BigDecimal::zero()),
        })
    }

    async fn update(&self, update: UpdateSettings) -> RepositoryResult<Settings> {
        let mut tx = self.pool.begin().await?;
        macro_rules! update_setting {
        ($key:expr, $value:expr) => {
            if let Some(val) = $value {
                let json = serde_json::to_string(&val).map_err(|e| RepositoryError::Validation(e.to_string()))?;
                sqlx::query!(
                    r#"INSERT INTO settings (key, value) 
                       VALUES ($1, $2) 
                       ON CONFLICT (key) DO UPDATE 
                       SET value = $2"#,
                    $key,
                    json
                )
                .execute(&mut *tx)
                .await?;
            }
        };
    }

        macro_rules! update_nullable_setting {
        ($key:expr, $value:expr) => {
            match $value {
                Some(Some(v)) => {
                    let json = serde_json::to_string(&v).map_err(|e| RepositoryError::Validation(e.to_string()))?;
                    sqlx::query!(
                        r#"INSERT INTO settings (key, value) 
                           VALUES ($1, $2) 
                           ON CONFLICT (key) DO UPDATE 
                           SET value = $2"#,
                        $key,
                        json
                    )
                    .execute(&mut *tx)
                    .await?;
                }
                Some(None) => {
                    sqlx::query!(
                        r#"INSERT INTO settings (key, value) 
                           VALUES ($1, NULL) 
                           ON CONFLICT (key) DO UPDATE 
                           SET value = NULL"#,
                        $key
                    )
                    .execute(&mut *tx)
                    .await?;
                }
                None => {}
            }
        };
    }

        update_setting!("bot_messages_support", update.bot_messages_support);
        update_nullable_setting!(
            "bot_messages_support_image_id",
            update.bot_messages_support_image_id
        );
        update_setting!(
            "bot_messages_new_user_welcome",
            update.bot_messages_new_user_welcome
        );
        update_nullable_setting!(
            "bot_messages_new_user_welcome_image_id",
            update.bot_messages_new_user_welcome_image_id
        );
        update_setting!(
            "bot_messages_returning_user_welcome",
            update.bot_messages_returning_user_welcome
        );
        update_nullable_setting!(
            "bot_messages_returning_user_welcome_image_id",
            update.bot_messages_returning_user_welcome_image_id
        );
        update_setting!("pricing_global_markup", update.pricing_global_markup);
        update_setting!(
            "pricing_platform_commission",
            update.pricing_platform_commission
        );
        update_setting!(
            "pricing_gateway_commission",
            update.pricing_gateway_commission
        );
        update_setting!(
            "pricing_gateway_bonus_mock_provider",
            update.pricing_gateway_bonus_mock_provider
        );
        update_setting!(
            "pricing_gateway_bonus_platform_card",
            update.pricing_gateway_bonus_platform_card
        );
        update_setting!(
            "pricing_gateway_bonus_platform_sbp",
            update.pricing_gateway_bonus_platform_sbp
        );
        update_setting!("referral_program_enabled", update.referral_program_enabled);
        update_setting!("referral_percentage", update.referral_percentage);

        Ok(self.load_settings().await?)
    }
}

fn get_string(map: &HashMap<String, Option<String>>, key: &str, default: &str) -> String {
    map.get(key)
        .and_then(|v| v.as_ref())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            tracing::warn!(key, "missing or null settings key, using default");
            default.to_string()
        })
}

fn get_uuid(map: &HashMap<String, Option<String>>, key: &str) -> Option<Uuid> {
    map.get(key)
        .and_then(|v| v.as_ref())
        .and_then(|s| Uuid::parse_str(s).ok())
}

fn get_bigdecimal(
    map: &HashMap<String, Option<String>>,
    key: &str,
    default: BigDecimal,
) -> BigDecimal {
    map.get(key)
        .and_then(|v| v.as_ref())
        .and_then(|s| s.parse::<BigDecimal>().ok())
        .unwrap_or(default)
}

fn get_bool(map: &HashMap<String, Option<String>>, key: &str, default: bool) -> bool {
    map.get(key)
        .and_then(|v| v.as_ref())
        .and_then(|s| match s.trim().to_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => Some(true),
            "false" | "0" | "no" | "off" => Some(false),
            _ => None,
        })
        .unwrap_or(default)
}
