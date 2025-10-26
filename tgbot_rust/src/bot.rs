use std::{result::Result::Ok, sync::Arc};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use rand::{Rng, distr::Alphanumeric, prelude::SliceRandom};
use serde::{Deserialize, Serialize};
use teloxide::{
    Bot,
    dispatching::{
        HandlerExt, UpdateFilterExt,
        dialogue::{RedisStorage, serializer::Json},
    },
    dptree,
    macros::BotCommands,
    payloads::{EditMessageTextSetters, SendMessageSetters},
    prelude::{Dialogue, Dispatcher, Request, Requester},
    types::{CallbackQuery, ChatId, Me, Message, MessageId, ParseMode, Update},
};
use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;

use crate::{
    AppState,
    api::{backend_api::BackendApi, captcha_api::CaptchaApi},
    bot::{
        handlers::{
            balance::balance_handler, captcha_answer::captcha_answer_handler,
            deposit_amount::deposit_amount_handler, deposit_confirm::deposit_confirm_handler,
            deposit_gateway::deposit_gateway_handler, fallback_bot_msg::fallback_bot_msg,
            main_menu::main_menu_handler, my_orders::my_orders_handler,
            my_subscriptions::my_subscriptions_handler, start::start_handler,
            support::support_handler,
        },
        keyboards::back_to_main_menu::back_to_main_menu_inline_keyboard,
    },
    errors::{AppError, AppResult},
    models::DispatchMessagePayload,
};

mod handlers;
mod keyboards;
mod middlewares;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CategoryAction {
    View,
    Back,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PaymentAction {
    SelectGateway { gateway: String },
    SelectAmount { gateway: String, amount: i64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceData {
    pub order_id: String,
    pub pay_url: Option<String>,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum BotState {
    #[default]
    Start,
    WaitingForCaptcha {
        correct_answer: String,
    },
    WaitingForReferralBotToken,
    Category {
        parent_id: i64,
        category_id: i64,
    },
    DepositSelectGateway,
    DepositSelectAmount {
        gateway: String,
    },
    DepositConfirm {
        amount: i64,
        gateway: String,
        invoice: Option<InvoiceData>,
    },
    Balance,
    MyOrders,
    MySubscriptions,
    ReferralProgram,
    Support,
    MainMenu,
}

impl BotState {
    pub fn pack(&self) -> String {
        serde_json::to_string(self).expect("Failed to serialize BotState")
    }

    pub fn unpack(s: &str) -> Option<Self> {
        serde_json::from_str(s).ok()
    }
}

impl From<BotState> for String {
    fn from(value: BotState) -> Self {
        value.pack()
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "t", content = "d")]
pub enum CallbackData {
    AnswerCaptcha { answer: String },
    SelectGateway { gateway: String },
    SelectAmount { amount: i64 },
    ToCategory { category_id: i64, parent_id: i64 },
    ToMainMenu,
    ToDepositSelectGateway,
    ToBalance,
    ToMyOrders,
    ToMySubscriptions,
    ToReferralProgram,
    ToSupport,
}

impl CallbackData {
    pub fn pack(&self) -> String {
        serde_json::to_string(self).expect("Failed to serialize CallbackData")
    }

    pub fn unpack(s: &str) -> Option<Self> {
        serde_json::from_str(s).ok()
    }

    pub fn from_query(query: &CallbackQuery) -> Option<Self> {
        query
            .data
            .as_ref()
            .and_then(|d| serde_json::from_str::<Self>(d).ok())
    }
}

impl From<CallbackData> for String {
    fn from(value: CallbackData) -> Self {
        value.pack()
    }
}

impl From<String> for CallbackData {
    fn from(value: String) -> Self {
        CallbackData::unpack(&value).unwrap()
    }
}

#[derive(Clone, BotCommands)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    Start,
}

type MyDialogue = Dialogue<BotState, RedisStorage<Json>>;

pub async fn start_bot<'a>(
    app_state: AppState,
    token: &str,
    fallback_bot_username: &'a str,
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
        .enter_dialogue::<Message, RedisStorage<Json>, BotState>()
        .branch(
            dptree::entry()
                .filter_command::<Command>()
                .endpoint(command_handler),
        );

    let callback_router = dptree::entry().endpoint(
        async move |dialogue: MyDialogue,
                    q: CallbackQuery,
                    bot: Bot,
                    username: String,
                    api_client: Arc<BackendApi>,
                    bot_state: BotState,
                    captcha_api_client: Arc<CaptchaApi>|
                    -> AppResult<()> {
            let data = match CallbackData::from_query(&q) {
                Some(data) => data,
                None => return Ok(()),
            };

            match data {
                CallbackData::AnswerCaptcha { .. } => {
                    captcha_answer_handler(
                        bot,
                        dialogue,
                        q,
                        username,
                        api_client,
                        captcha_api_client,
                    )
                    .await?;
                }
                CallbackData::SelectGateway { gateway } => {
                    dialogue
                        .update(BotState::DepositSelectAmount { gateway })
                        .await
                        .map_err(AppError::from)?;
                    deposit_amount_handler(bot, dialogue, q, username, api_client, bot_state)
                        .await?;
                }
                CallbackData::SelectAmount { amount } => {
                    let gateway = match &bot_state {
                        BotState::DepositSelectAmount { gateway } => gateway.clone(),
                        _ => return Ok(()),
                    };
                    let new_state = BotState::DepositConfirm {
                        gateway,
                        amount,
                        invoice: None,
                    };
                    dialogue
                        .update(new_state.clone())
                        .await
                        .map_err(AppError::from)?;
                    deposit_confirm_handler(bot, q, dialogue, api_client, new_state).await?;
                }
                CallbackData::ToMainMenu => {
                    dialogue
                        .update(BotState::MainMenu)
                        .await
                        .map_err(AppError::from)?;
                    main_menu_handler(bot, dialogue, q, username, api_client).await?;
                }
                CallbackData::ToDepositSelectGateway => {
                    dialogue
                        .update(BotState::DepositSelectGateway)
                        .await
                        .map_err(AppError::from)?;
                    deposit_gateway_handler(bot, dialogue, q, username.clone(), api_client).await?;
                }
                CallbackData::ToBalance => {
                    dialogue
                        .update(BotState::Balance)
                        .await
                        .map_err(AppError::from)?;
                    balance_handler(bot, dialogue, q, username, api_client).await?;
                }
                CallbackData::ToMyOrders => {
                    dialogue
                        .update(BotState::MyOrders)
                        .await
                        .map_err(AppError::from)?;
                    my_orders_handler(bot, dialogue, q, api_client).await?;
                }
                CallbackData::ToMySubscriptions => {
                    dialogue
                        .update(BotState::MySubscriptions)
                        .await
                        .map_err(AppError::from)?;
                    my_subscriptions_handler(bot, dialogue, q, api_client).await?;
                }
                CallbackData::ToReferralProgram => {
                    dialogue
                        .update(BotState::ReferralProgram)
                        .await
                        .map_err(AppError::from)?;
                }
                CallbackData::ToSupport => {
                    dialogue
                        .update(BotState::Support)
                        .await
                        .map_err(AppError::from)?;
                    support_handler(bot, dialogue, q, username, api_client).await?;
                }
                CallbackData::ToCategory {
                    parent_id,
                    category_id,
                } => {
                    dialogue
                        .update(BotState::Category {
                            parent_id,
                            category_id,
                        })
                        .await
                        .map_err(AppError::from)?;
                }
            }

            Ok(())
        },
    );

    let callback_query_handler = Update::filter_callback_query()
        .enter_dialogue::<CallbackQuery, RedisStorage<Json>, BotState>()
        .branch(callback_router);

    let fallback_bot_username = Arc::new(fallback_bot_username.to_string());

    let mut dispatcher = Dispatcher::builder(
        bot.clone(),
        dptree::entry()
            .filter_async(
                async move |update: Update, api_client: Arc<BackendApi>, me: Me| {
                    let chat_id = match update.chat() {
                        Some(chat) => chat.id,
                        None => return false,
                    };
                    match api_client
                        .get_user(chat_id.0, &me.user.username.unwrap_or_default())
                        .await
                    {
                        Ok(user) => !user.is_blocked,
                        Err(_) => true,
                    }
                },
            )
            .inspect_async(
                async |bot: Bot, update: Update, fallback_bot_username: String| {
                    let chat_id = match update.chat() {
                        Some(chat) => chat.id,
                        None => return,
                    };

                    if let Err(err) = fallback_bot_msg(bot, chat_id, fallback_bot_username).await {
                        tracing::error!("Failed to send fallback bot message: {}", err);
                    }
                },
            )
            .branch(handler)
            .branch(callback_query_handler),
    )
    .dependencies(dptree::deps![
        app_state.clone(),
        storage,
        username.clone(),
        api_client,
        captcha_api_client,
        fallback_bot_username
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
            start_handler(
                bot,
                dialogue,
                msg,
                username.clone(),
                api_client,
                captcha_api_client,
            )
            .await
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
            .reply_markup(back_to_main_menu_inline_keyboard())
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
                    .reply_markup(back_to_main_menu_inline_keyboard())
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
        .reply_markup(back_to_main_menu_inline_keyboard())
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
