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
    pub cors_origins: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv().expect("Failed to load .env file");
        envy::from_env::<Config>().expect("Failed to load config from environment variables")
    }
}
