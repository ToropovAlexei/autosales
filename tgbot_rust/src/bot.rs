use std::sync::Arc;

use rand::distr::Alphanumeric;
use serde::{Deserialize, Serialize};
use teloxide::{
    Bot,
    dispatching::dialogue::{GetChatId, RedisStorage, serializer::Json},
    dptree::{self},
    macros::BotCommands,
    prelude::{Dialogue, Dispatcher, Requester},
    types::{CallbackQuery, ChatId, Me, Message, MessageId, ParseMode, Update},
};

use crate::{
    AppState,
    api::{backend_api::BackendApi, captcha_api::CaptchaApi},
    bot::handlers::{captcha_answer::captcha_answer_handler, start::start_handler},
    errors::AppResult,
    models::DispatchMessagePayload,
};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use rand::Rng;
use rand::prelude::SliceRandom;
use teloxide::dispatching::HandlerExt;
use teloxide::dispatching::UpdateFilterExt;
use teloxide::payloads::EditMessageTextSetters;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Request;
use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;

mod handlers;
mod keyboards;
mod middlewares;

#[derive(Clone, Serialize, Deserialize, Default)]
pub enum BotState {
    #[default]
    Idle,
    Start,
    WaitingForCaptcha {
        correct_answer: String,
    },
    WaitingForReferralBotToken,
}

#[derive(Clone, BotCommands)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    Start,
}

type MyDialogue = Dialogue<BotState, RedisStorage<Json>>;

pub async fn start_bot(
    app_state: AppState,
    token: &str,
    api_client: Arc<BackendApi>,
    captcha_api_client: Arc<CaptchaApi>,
    cancel_token: CancellationToken,
) -> AppResult<()> {
    let bot = Bot::new(token);
    let me = bot.get_me().await?;
    let username = me.user.username.unwrap_or_default();
    tracing::info!("Starting bot: @{}", username);
    let redis_url = format!(
        "redis://{}:{}",
        app_state.config.redis_host, app_state.config.redis_port
    );

    let storage = RedisStorage::open(redis_url.as_str(), Json).await?;

    let listener_handle = tokio::spawn(start_redis_listener(
        bot.clone(),
        redis_url,
        username.clone(),
    ));

    let handler = Update::filter_message()
        .filter_async(
            async move |msg: Message, api_client: Arc<BackendApi>, me: Me| match msg.chat_id() {
                Some(telegram_id) => {
                    match api_client
                        .get_user(telegram_id.0, &me.user.username.unwrap_or_default())
                        .await
                    {
                        Ok(user) => !user.is_blocked,
                        Err(_) => true,
                    }
                }
                None => true,
            },
        )
        .enter_dialogue::<Message, RedisStorage<Json>, BotState>()
        .branch(
            dptree::entry()
                .filter_command::<Command>()
                .endpoint(command_handler),
        )
        .branch(dptree::case![BotState::Start].endpoint(start_handler));

    let captcha_branch = Update::filter_callback_query()
        .filter(|q: CallbackQuery| {
            q.data
                .as_ref()
                .map(|d| d.starts_with("captcha_"))
                .unwrap_or(false)
        })
        .enter_dialogue::<CallbackQuery, RedisStorage<Json>, BotState>()
        .endpoint(captcha_answer_handler);

    let mut dispatcher = Dispatcher::builder(
        bot.clone(),
        dptree::entry().branch(handler).branch(captcha_branch),
    )
    .dependencies(dptree::deps![
        app_state.clone(),
        storage,
        username.clone(),
        api_client,
        captcha_api_client
    ])
    .default_handler(|upd| async move {
        tracing::warn!("Unhandled update: {upd:?}");
    })
    .enable_ctrlc_handler()
    .build();

    tokio::select! {
        _ = dispatcher.dispatch() => {},
        _ = cancel_token.cancelled() => {
            tracing::info!("Cancellation requested for bot {username}");
        }
    }

    listener_handle.abort();
    bot.delete_webhook().await?;
    tracing::info!("Bot stopped.");
    Ok(())
}

async fn command_handler(
    bot: Bot,
    msg: Message,
    cmd: Command,
    dialogue: MyDialogue,
    username: String,
    api_client: Arc<BackendApi>,
    captcha_api_client: Arc<CaptchaApi>,
) -> AppResult<()> {
    match cmd {
        Command::Start => {
            dialogue.update(BotState::Start).await?;
            start_handler(bot, dialogue, msg, username, api_client, captcha_api_client).await
        }
    }
}

async fn start_redis_listener(bot: Bot, redis_url: String, bot_username: String) -> AppResult<()> {
    let channel = format!("bot-notifications:{bot_username}");
    tracing::info!("Subscribing to Redis channel: {channel}");

    let conn = redis::Client::open(redis_url)?;

    let mut pubsub = conn.get_async_pubsub().await?;
    pubsub.subscribe(&channel).await?;
    let mut msg_stream = pubsub.on_message();

    while let Some(msg) = msg_stream.next().await {
        if let Ok(payload_str) = msg.get_payload::<String>() {
            if let Ok(parsed) = serde_json::from_str::<DispatchMessagePayload>(&payload_str) {
                let res = handle_msg(bot.clone(), parsed).await;
                if let Err(e) = res {
                    tracing::error!("Error handling message: {e}");
                }
            }
        }
    }

    Ok(())
}

async fn handle_msg(bot: Bot, payload: DispatchMessagePayload) -> AppResult<()> {
    let chat_id = ChatId(payload.telegram_id);

    if let Some(msg_id) = payload.message_to_delete {
        if let Err(e) = bot.delete_message(chat_id, MessageId(msg_id)).send().await {
            tracing::warn!(
                "Could not delete message {} for user {}. It might have been deleted already. Error: {}",
                msg_id,
                chat_id,
                e
            );
        } else {
            tracing::info!("Deleted message {} for user {}", msg_id, chat_id);
        }
    }

    let text = payload.message;
    let parse_mode = ParseMode::Html;

    if let Some(msg_id) = payload.message_to_edit {
        match bot
            .edit_message_text(chat_id, MessageId(msg_id), text.clone())
            .parse_mode(parse_mode.clone())
            .send()
            .await
        {
            Ok(_) => tracing::info!("Edited message {} for user {}", msg_id, chat_id),
            Err(e) => {
                tracing::warn!(
                    "Could not edit message {}, sending new one. Error: {}",
                    msg_id,
                    e
                );
                if let Err(e) = bot
                    .send_message(chat_id, text)
                    .parse_mode(parse_mode)
                    .send()
                    .await
                {
                    tracing::warn!(
                        "Failed to send new message to user {}. Error: {}",
                        chat_id,
                        e
                    );
                }
            }
        }
        return Ok(());
    }

    if let Err(e) = bot
        .send_message(chat_id, text)
        .parse_mode(parse_mode)
        .send()
        .await
    {
        tracing::warn!(
            "Failed to send notification to user {}. Error: {}",
            chat_id,
            e
        );
    } else {
        tracing::info!("Sent notification to user {}", chat_id);
    }

    Ok(())
}

pub async fn generate_captcha_and_options(
    captcha_api_client: Arc<CaptchaApi>,
    chars: u8,
    answers: u8,
) -> AppResult<(Vec<u8>, String, Vec<String>)> {
    let captcha = captcha_api_client.get_captcha().await?;
    let mut options = vec![captcha.solution.clone()];
    let mut rng = rand::rng();

    while options.len() < answers as usize {
        let option: String = (0..chars)
            .map(|_| {
                let c = rng.sample(Alphanumeric) as char;
                c.to_ascii_uppercase()
            })
            .collect();

        if !options.contains(&option) {
            options.push(option);
        }
    }

    options.shuffle(&mut rng);
    let image = STANDARD.decode(&captcha.image)?;
    Ok((image, captcha.solution, options))
}
