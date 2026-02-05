use dotenvy::dotenv;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub backend_port: u16,
    pub database_host: String,
    pub database_port: u16,
    pub database_user: String,
    pub database_password: String,
    pub database_name: String,
    pub redis_host: String,
    pub redis_port: u16,
    pub cors_origins: String,
    pub jwt_secret: String,
    pub totp_encode_secret: String,
    pub two_fa_token_ttl_minutes: i64,
    pub access_token_ttl_minutes: i64,
    pub refresh_token_ttl_minutes: i64,
    pub image_upload_path: String,
    pub service_api_key: String,
    pub captcha_api_url: String,
    #[cfg(feature = "contms-provider")]
    pub contms_api_url: String,
    #[cfg(feature = "mock-payments-provider")]
    pub mock_payments_provider_url: String,
    pub payment_notification_minutes: u64,
    pub platform_payment_system_base_url: String,
    pub platform_payment_system_login: String,
    pub platform_payment_system_password: String,
    pub platform_payment_system_2fa_key: String,
    pub files_fm_upload_token: String,
    pub files_fm_folder_hash: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv().ok();
        envy::from_env::<Config>().expect("Failed to load config from environment variables")
    }
}
