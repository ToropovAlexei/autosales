use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::PgPool;
use std::str::FromStr;
use uuid::Uuid;

use crate::{
    errors::repository::RepositoryResult,
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

            pricing_global_markup: get_decimal(&map, "pricing_global_markup", dec!(0)),
            pricing_platform_commission: get_decimal(&map, "pricing_platform_commission", dec!(0)),

            pricing_gateway_markup: get_decimal(&map, "pricing_gateway_markup", dec!(0)),

            pricing_gateway_bonus_mock_provider: get_decimal(
                &map,
                "pricing_gateway_bonus_mock_provider",
                dec!(0),
            ),
            pricing_gateway_bonus_platform_card: get_decimal(
                &map,
                "pricing_gateway_bonus_platform_card",
                dec!(0),
            ),
            pricing_gateway_bonus_platform_sbp: get_decimal(
                &map,
                "pricing_gateway_bonus_platform_sbp",
                dec!(0),
            ),

            referral_program_enabled: get_bool(&map, "referral_program_enabled", false),
            referral_percentage: get_decimal(&map, "referral_percentage", dec!(0)),
            bot_payment_system_support_operators: get_string_vec(
                &map,
                "bot_payment_system_support_operators",
            ),
            bot_about: get_string(&map, "bot_about", ""),
            bot_description: get_string(&map, "bot_description", ""),
        })
    }

    async fn update(&self, update: UpdateSettings) -> RepositoryResult<Settings> {
        let mut tx = self.pool.begin().await?;
        macro_rules! update_setting {
        ($key:expr, $value:expr) => {
            if let Some(val) = $value {
                sqlx::query!(
                    r#"INSERT INTO settings (key, value) 
                       VALUES ($1, $2) 
                       ON CONFLICT (key) DO UPDATE 
                       SET value = $2"#,
                    $key,
                    val.to_string()
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
                    sqlx::query!(
                        r#"INSERT INTO settings (key, value) 
                           VALUES ($1, $2) 
                           ON CONFLICT (key) DO UPDATE 
                           SET value = $2"#,
                        $key,
                        v.to_string()
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

        macro_rules! update_vec_setting {
        ($key:expr, $value:expr) => {
            if let Some(val) = $value {
                sqlx::query!(
                    r#"INSERT INTO settings (key, value) 
                       VALUES ($1, $2) 
                       ON CONFLICT (key) DO UPDATE 
                       SET value = $2"#,
                    $key,
                    val.join(",")
                )
                .execute(&mut *tx)
                .await?;
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
        update_setting!("pricing_gateway_markup", update.pricing_gateway_markup);
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
        update_vec_setting!(
            "bot_payment_system_support_operators",
            update.bot_payment_system_support_operators
        );
        update_setting!("bot_about", update.bot_about);
        update_setting!("bot_description", update.bot_description);

        tx.commit().await?;

        Ok(self.load_settings().await?)
    }
}

fn get_string(map: &HashMap<String, Option<String>>, key: &str, default: &str) -> String {
    map.get(key)
        .and_then(|v| v.as_ref())
        .map(|s| s.to_string())
        .unwrap_or_else(|| default.to_string())
}

fn get_uuid(map: &HashMap<String, Option<String>>, key: &str) -> Option<Uuid> {
    map.get(key)
        .and_then(|v| v.as_ref())
        .and_then(|s| Uuid::parse_str(s).ok())
}

fn get_decimal(map: &HashMap<String, Option<String>>, key: &str, default: Decimal) -> Decimal {
    map.get(key)
        .and_then(|v| v.as_ref())
        .and_then(|s| Decimal::from_str(s).ok())
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

fn get_string_vec(map: &HashMap<String, Option<String>>, key: &str) -> Vec<String> {
    map.get(key)
        .and_then(|v| v.as_ref())
        .map(|s| s.split(',').map(|s| s.to_string()).collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    use uuid::Uuid;

    #[sqlx::test]
    async fn test_load_settings_empty_db(pool: PgPool) {
        let repo = SettingsRepository::new(Arc::new(pool));

        let settings = repo.load_settings().await.unwrap();

        // Check a few default values
        assert_eq!(settings.bot_messages_support, "Служба поддержки");
        assert_eq!(settings.pricing_global_markup, dec!(0));
        assert!(!settings.referral_program_enabled);
        assert!(settings.bot_messages_support_image_id.is_none());
    }

    #[sqlx::test]
    async fn test_load_settings_with_data(pool: PgPool) {
        let repo = SettingsRepository::new(Arc::new(pool.clone()));

        // Insert some data manually
        let image_id = Uuid::new_v4();
        sqlx::query!(
            r#"
            INSERT INTO settings (key, value) VALUES
            ('bot_messages_support', 'New Support Message'),
            ('pricing_global_markup', '15.5'),
            ('referral_program_enabled', 'true'),
            ('bot_messages_support_image_id', $1)
            "#,
            image_id.to_string()
        )
        .execute(&pool)
        .await
        .unwrap();

        let settings = repo.load_settings().await.unwrap();

        assert_eq!(settings.bot_messages_support, "New Support Message");
        assert_eq!(
            settings.pricing_global_markup,
            Decimal::from_str("15.5").unwrap()
        );
        assert!(settings.referral_program_enabled);
        assert_eq!(settings.bot_messages_support_image_id, Some(image_id));
        // Check that a non-set value has its default
        assert_eq!(settings.pricing_platform_commission, dec!(0));
    }

    #[sqlx::test]
    async fn test_update_settings(pool: PgPool) {
        let repo = SettingsRepository::new(Arc::new(pool.clone()));

        // First, load defaults
        let initial_settings = repo.load_settings().await.unwrap();
        assert_eq!(
            initial_settings.bot_messages_new_user_welcome,
            "Добро пожаловать, {username}!"
        );
        assert!(
            initial_settings
                .bot_messages_new_user_welcome_image_id
                .is_none()
        );

        // Prepare an update
        let new_image_id = Uuid::new_v4();
        let update = UpdateSettings {
            bot_messages_new_user_welcome: Some("Welcome, new user!".to_string()),
            bot_messages_new_user_welcome_image_id: Some(Some(new_image_id)),
            pricing_platform_commission: Some(Decimal::from_str("2.5").unwrap()),
            referral_program_enabled: Some(true),
            // Keep some fields as None to ensure they are not updated
            ..Default::default()
        };

        // Perform the update
        let updated_settings = repo.update(update).await.unwrap();

        // Verify the returned settings object
        assert_eq!(
            updated_settings.bot_messages_new_user_welcome,
            "Welcome, new user!"
        );
        assert_eq!(
            updated_settings.bot_messages_new_user_welcome_image_id,
            Some(new_image_id)
        );
        assert_eq!(
            updated_settings.pricing_platform_commission,
            Decimal::from_str("2.5").unwrap()
        );
        assert!(updated_settings.referral_program_enabled);
        // Check that a non-updated field remains at its default
        assert_eq!(
            updated_settings.pricing_global_markup,
            initial_settings.pricing_global_markup
        );

        // Verify by loading from DB again
        let reloaded_settings = repo.load_settings().await.unwrap();
        assert_eq!(
            reloaded_settings.bot_messages_new_user_welcome,
            "Welcome, new user!"
        );
        assert_eq!(
            reloaded_settings.bot_messages_new_user_welcome_image_id,
            Some(new_image_id)
        );
        assert_eq!(
            reloaded_settings.pricing_platform_commission,
            Decimal::from_str("2.5").unwrap()
        );
    }

    #[sqlx::test]
    async fn test_update_settings_to_null(pool: PgPool) {
        let repo = SettingsRepository::new(Arc::new(pool.clone()));

        // Set an initial value for a nullable field
        let initial_image_id = Uuid::new_v4();
        let initial_update = UpdateSettings {
            bot_messages_returning_user_welcome_image_id: Some(Some(initial_image_id)),
            ..Default::default()
        };
        repo.update(initial_update).await.unwrap();

        // Verify it's set
        let settings_before_nulling = repo.load_settings().await.unwrap();
        assert_eq!(
            settings_before_nulling.bot_messages_returning_user_welcome_image_id,
            Some(initial_image_id)
        );

        // Now, update it to None (which should set it to NULL in the DB)
        let nulling_update = UpdateSettings {
            bot_messages_returning_user_welcome_image_id: Some(None),
            ..Default::default()
        };
        let nulled_settings = repo.update(nulling_update).await.unwrap();

        // Verify the returned settings object
        assert!(
            nulled_settings
                .bot_messages_returning_user_welcome_image_id
                .is_none()
        );

        // Verify by loading from DB again
        let reloaded_settings = repo.load_settings().await.unwrap();
        assert!(
            reloaded_settings
                .bot_messages_returning_user_welcome_image_id
                .is_none()
        );
    }
}
