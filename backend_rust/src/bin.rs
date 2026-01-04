use bigdecimal::{BigDecimal, Zero};
use rand::Rng;
use reqwest::Response;
use serde::{Deserialize, de::DeserializeOwned};
use std::sync::Arc;

use crate::{
    infrastructure::repositories::{
        admin_user::{AdminUserRepository, AdminUserRepositoryTrait},
        bot::{BotRepository, BotRepositoryTrait},
        role::{RoleRepository, RoleRepositoryTrait},
        user_role::{UserRoleRepository, UserRoleRepositoryTrait},
    },
    models::{
        admin_user::NewAdminUser,
        bot::{BotType, NewBot},
        role::NewRole,
        user_role::NewUserRole,
    },
    services::topt_encryptor::TotpEncryptor,
};

pub async fn create_admin_user_if_not_exists(
    admin_user_repo: &Arc<AdminUserRepository>,
    totp_encryptor: &TotpEncryptor,
    login: String,
    password: Option<String>,
    two_fa: Option<String>,
) -> i64 {
    if let Ok(admin) = admin_user_repo.get_by_login(&login).await {
        return admin.id;
    }
    let two_fa = two_fa.unwrap_or(totp_rs::Secret::generate_secret().to_encoded().to_string());
    let password = password.unwrap_or(generate_random_password());

    let id = admin_user_repo
        .create(NewAdminUser {
            created_by: 1, // System
            hashed_password: bcrypt::hash(password.clone(), bcrypt::DEFAULT_COST).unwrap(),
            login: login.to_string(),
            telegram_id: None,
            two_fa_secret: totp_encryptor.encrypt(&two_fa).unwrap(),
        })
        .await
        .unwrap()
        .id;
    println!("Login: {login} Password: {password} 2FA: {two_fa}");
    id
}

pub async fn create_admin_user_role_if_not_exists(
    role_repo: &Arc<RoleRepository>,
    role_name: &str,
) -> i64 {
    if let Ok(roles) = role_repo.get_roles().await
        && let Some(admin_role) = roles.iter().find(|role| role.name == role_name)
    {
        return admin_role.id;
    }
    role_repo
        .create_role(NewRole {
            name: role_name.to_string(),
            created_by: 1, // System,
            description: None,
        })
        .await
        .unwrap()
        .id
}

pub async fn assign_role_to_admin_user(
    admin_user_id: i64,
    role_id: i64,
    user_role_repo: &Arc<UserRoleRepository>,
) {
    if let Ok(user_roles) = user_role_repo.get_user_roles(admin_user_id).await
        && user_roles.is_empty()
    {
        user_role_repo
            .create_user_role(NewUserRole {
                user_id: admin_user_id,
                role_id,
                created_by: 1, // System
            })
            .await
            .unwrap();
    }
}

pub fn generate_random_password() -> String {
    let mut rng = rand::rng();
    let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let password: String = (0..12)
        .map(|_| {
            let idx = rng.random_range(0..chars.len());
            chars.chars().nth(idx).unwrap()
        })
        .collect();
    password
}

#[derive(Debug, Deserialize)]
struct GetMeResponse {
    result: GetMeResult,
}

#[derive(Debug, Deserialize)]
struct GetMeResult {
    username: String,
}

#[derive(Debug)]
pub struct InitBot {
    pub token: String,
    pub is_active: bool,
    pub is_primary: bool,
    pub r#type: BotType,
}

pub async fn create_bot_if_not_exists(
    bot: InitBot,
    bot_repo: &Arc<BotRepository>,
    client: &Arc<reqwest::Client>,
) -> i64 {
    if let Ok(bot) = bot_repo.get_by_token(bot.token.clone()).await {
        return bot.id;
    }
    let bot_name = parse_response::<GetMeResponse>(
        client
            .get(format!("https://api.telegram.org/bot{}/getMe", bot.token))
            .send()
            .await
            .expect("Error requesting getMe"),
    )
    .await
    .map(|r| r.result.username)
    .expect("Error getting bot name");

    bot_repo
        .create(NewBot {
            username: bot_name,
            created_by: None, // System
            is_active: bot.is_active,
            is_primary: bot.is_primary,
            owner_id: None,
            referral_percentage: BigDecimal::zero(),
            token: bot.token,
            r#type: bot.r#type,
        })
        .await
        .expect("Error creating bot")
        .id
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
