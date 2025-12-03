use std::{
    io::{BufReader, BufWriter},
    path::Path,
    sync::Arc,
};
use anyhow::anyhow;
use grammers_client::{Client, Config, InitParams, SignInError};
use grammers_session::SqliteSession;

use crate::api::backend_api::BackendApi;
use crate::errors::AppResult;

const SESSION_FILE: &str = "bot_creator.session";

pub struct BotFather {
    backend_api: Arc<BackendApi>,
    api_id: i32,
    api_hash: String,
}

impl BotFather {
    pub fn new(
        backend_api: Arc<BackendApi>,
        api_id_str: &str,
        api_hash: &str,
    ) -> anyhow::Result<Self> {
        let api_id = api_id_str
            .parse::<i32>()
            .context("Failed to parse API_ID")?;

        if api_id == 0 {
            return Err(anyhow!("API_ID is not set"));
        }
        if api_hash.is_empty() {
            return Err(anyhow!("API_HASH is not set"));
        }

        Ok(Self {
            backend_api,
            api_id,
            api_hash: api_hash.to_string(),
        })
    }

    pub async fn request_new_main_bot_token(&self) -> anyhow::Result<bool> {
        tracing::info!("Attempting to create a new bot via BotFather.");

        let session = Arc::new(SqliteSession::open(SESSION_FILE)?);
        let pool = SenderPool::new(Arc::clone(&session), self.api_id);
        let client = Client::new(&pool);

        if !client.is_authorized().await? {
            let phone = prompt("Enter your phone number (international format): ")?;
            let token = client.request_login_code(&phone, &self.api_hash).await?;
            let code = prompt("Enter the code you received: ")?;
            let signed_in = client.sign_in(&token, &code).await;
            match signed_in {
                Err(SignInError::PasswordRequired(password_token)) => {
                    let hint = password_token.hint().unwrap_or("None");
                    let prompt_message = format!("Enter the password (hint {}): ", &hint);
                    let password = prompt(prompt_message.as_str())?;

                    client
                        .check_password(password_token, password.trim())
                        .await?;
                }
                Err(e) => return Err(anyhow!("Failed to sign in: {}", e)),
                Ok(_) => (),
            }
            tracing::info!("Signed in successfully. Session saved to {SESSION_FILE}");
        }

        let bot_father_peer = client
            .resolve_username("BotFather")
            .await?
            .ok_or_else(|| anyhow!("Could not find BotFather"))?;

        let msg = client.send_message(bot_father_peer, "/newbot").await?;
        let reply = client
            .get_reply_to_message(&msg)
            .await?
            .ok_or_else(|| anyhow!("No reply from BotFather"))?;
        if !reply.text().contains("Alright, a new bot.") {
            tracing::error!("BotFather responded unexpectedly: {}", reply.text());
            return Ok(false);
        }

        let bot_name = format!(
            "My Monitored Bot {}",
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs()
        );
        let msg = client.send_message(bot_father_peer, bot_name).await?;
        let reply = client
            .get_reply_to_message(&msg)
            .await?
            .ok_or_else(|| anyhow!("No reply from BotFather"))?;
        if !reply.text().contains("Good. Now let's choose a username") {
            tracing::error!(
                "BotFather responded unexpectedly after name: {}",
                reply.text()
            );
            return Ok(false);
        }

        let bot_username = format!(
            "my_monitored_bot_{}_bot",
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs()
        );
        let msg = client.send_message(bot_father_peer, bot_username).await?;
        let reply = client
            .get_reply_to_message(&msg)
            .await?
            .ok_or_else(|| anyhow!("No reply from BotFather"))?;
    }
}

fn prompt(message: &str) -> AppResult<String> {
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    stdout.write_all(message.as_bytes())?;
    stdout.flush()?;

    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();

    let mut line = String::new();
    stdin.read_line(&mut line)?;
    Ok(line)
}
