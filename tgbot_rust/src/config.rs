use dotenvy::dotenv;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub service_api_key: String,
    pub telegram_api_id: String,
    pub telegram_api_hash: String,
    pub redis_host: String,
    pub redis_port: u16,
    pub backend_api_url: String,
    pub webhook_host: String,
    pub webhook_port: u16,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv().ok();
        let config = envy::from_env::<Config>().expect("Invalid env config");
        config.validate().expect("Invalid config values");
        config
    }

    fn validate(&self) -> Result<(), String> {
        if !self.backend_api_url.ends_with('/') {
            return Err(format!(
                "Invalid backend_api_url: '{}' must end with a `/`",
                self.backend_api_url
            ));
        }

        if self.webhook_port == 0 {
            return Err("Webhook port must be > 0".into());
        }

        if url::Url::parse(&self.backend_api_url).is_err() {
            return Err(format!(
                "backend_api_url is not a valid URL: '{}'",
                self.backend_api_url
            ));
        }

        Ok(())
    }
}
