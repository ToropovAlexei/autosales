use std::{result::Result::Ok, sync::Arc};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use teloxide::{
    Bot,
    dispatching::{
        HandlerExt, UpdateFilterExt,
        dialogue::{RedisStorage, serializer::Json},
    },
    dptree,
    macros::BotCommands,
    payloads::{SendMessageSetters, SendPhotoSetters},
    prelude::{Dialogue, Dispatcher, Request, Requester},
    types::{CallbackQuery, ChatId, InputFile, Message, ParseMode, Update},
};
use tokio_stream::StreamExt;

use crate::{
    AppState,
    api::backend_api::BackendApi,
    bot::{
        handlers::{
            balance::balance_handler, buy::buy_handler, captcha_answer::captcha_answer_handler,
            catalog::catalog_handler, deposit_amount::deposit_amount_handler,
            deposit_confirm::deposit_confirm_handler, deposit_gateway::deposit_gateway_handler,
            fallback_bot_msg::fallback_bot_msg, main_menu::main_menu_handler,
            my_bots::my_bots_handler, my_orders::my_orders_handler,
            my_subscriptions::my_subscriptions_handler, product::product_handler,
            start::start_handler, support::support_handler,
        },
        keyboards::back_to_main_menu::back_to_main_menu_inline_keyboard,
    },
    errors::{AppError, AppResult},
    models::{DispatchMessagePayload, payment::PaymentSystem},
};

pub mod utils;

mod handlers {
    pub mod balance;
    pub mod buy;
    pub mod captcha_answer;
    pub mod catalog;
    pub mod deposit_amount;
    pub mod deposit_confirm;
    pub mod deposit_gateway;
    pub mod fallback_bot_msg;
    pub mod main_menu;
    pub mod my_bots;
    pub mod my_orders;
    pub mod my_subscriptions;
    pub mod product;
    pub mod start;
    pub mod support;
}
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InvoiceData {
    pub id: i64,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MockDetails {
    pub pay_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(tag = "t", content = "c")]
pub enum BotState {
    #[default]
    Initial,
    WaitingForCaptcha {
        correct_answer: String,
    },
    WaitingForReferralBotToken,
    Category {
        category_id: Option<i64>,
    },
    DepositSelectGateway,
    DepositSelectAmount {
        gateway: PaymentSystem,
    },
    DepositConfirm {
        amount: i64,
        gateway: PaymentSystem,
        invoice: Option<InvoiceData>,
    },
    Balance,
    MyOrders,
    MySubscriptions,
    ReferralProgram,
    Support,
    MainMenu,
    Product {
        id: i64,
    },
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

#[derive(Clone)]
pub struct BotUsername(pub String);

impl std::fmt::Display for BotUsername {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "t", content = "d")]
pub enum CallbackData {
    AnswerCaptcha { answer: String },
    SelectGateway { gateway: PaymentSystem },
    SelectAmount { amount: i64 },
    ToCategory { category_id: Option<i64> },
    ToMainMenu,
    ToDepositSelectGateway,
    ToBalance,
    ToMyOrders,
    ToMySubscriptions,
    ToReferralProgram,
    ToSupport,
    ToProduct { id: i64 },
    Buy { id: i64 },
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

#[derive(Debug, Clone, BotCommands)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    Start,
    MyBots,
}

type MyDialogue = Dialogue<BotState, RedisStorage<Json>>;

pub async fn run_bot(
    bot_token: String,
    bot_id: i64,
    app_state: AppState,
    client: Arc<BackendApi>,
    fallback_bot_username: BotUsername,
) -> AppResult<()> {
    let bot = Bot::new(bot_token);
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
        bot_id,
        client.clone(),
    ));

    let handler = Update::filter_message()
        .enter_dialogue::<Message, RedisStorage<Json>, BotState>()
        .branch(
            dptree::entry()
                .filter_command::<Command>()
                .endpoint(command_handler),
        )
        .branch(
            dptree::filter(|state: BotState| state == BotState::WaitingForReferralBotToken)
                .endpoint(handlers::my_bots::referral_bot_token_handler),
        );

    let callback_router = dptree::entry().endpoint(
        async move |dialogue: MyDialogue,
                    q: CallbackQuery,
                    bot: Bot,
                    api_client: Arc<BackendApi>,
                    bot_state: BotState|
                    -> AppResult<()> {
            let data = match CallbackData::from_query(&q) {
                Some(data) => data,
                None => return Ok(()),
            };

            match data {
                CallbackData::AnswerCaptcha { .. } => {
                    captcha_answer_handler(bot, dialogue, q, api_client).await?;
                }
                CallbackData::SelectGateway { gateway } => {
                    dialogue
                        .update(BotState::DepositSelectAmount { gateway })
                        .await
                        .map_err(AppError::from)?;
                    deposit_amount_handler(bot, dialogue, q, api_client, bot_state).await?;
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
                    main_menu_handler(bot, dialogue, q, api_client).await?;
                }
                CallbackData::ToDepositSelectGateway => {
                    dialogue
                        .update(BotState::DepositSelectGateway)
                        .await
                        .map_err(AppError::from)?;
                    deposit_gateway_handler(bot, dialogue, q, api_client).await?;
                }
                CallbackData::ToBalance => {
                    dialogue
                        .update(BotState::Balance)
                        .await
                        .map_err(AppError::from)?;
                    balance_handler(bot, dialogue, q, api_client).await?;
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
                    support_handler(bot, dialogue, q, api_client).await?;
                }
                CallbackData::ToCategory { category_id } => {
                    dialogue
                        .update(BotState::Category { category_id })
                        .await
                        .map_err(AppError::from)?;
                    catalog_handler(bot, dialogue, q, api_client, category_id).await?;
                }
                CallbackData::ToProduct { id } => {
                    dialogue
                        .update(BotState::Product { id })
                        .await
                        .map_err(AppError::from)?;
                    product_handler(bot, q, api_client, id).await?;
                }
                CallbackData::Buy { id } => {
                    buy_handler(bot, q, api_client, id).await?;
                }
            }

            Ok(())
        },
    );

    let callback_query_handler = Update::filter_callback_query()
        .enter_dialogue::<CallbackQuery, RedisStorage<Json>, BotState>()
        .branch(callback_router);

    let mut dispatcher = Dispatcher::builder(
        bot.clone(),
        dptree::entry()
            .branch(handler)
            .branch(callback_query_handler),
    )
    .dependencies(dptree::deps![
        app_state.clone(),
        storage,
        client.clone(),
        fallback_bot_username
    ])
    .default_handler(|upd| async move {
        tracing::warn!("Unhandled update: {upd:?}");
    })
    .enable_ctrlc_handler()
    .build();

    dispatcher.dispatch().await;

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
    api_client: Arc<BackendApi>,
    app_state: AppState,
    fallback_bot_username: BotUsername,
) -> AppResult<()> {
    match cmd {
        Command::Start => {
            fallback_bot_msg(bot.clone(), msg.chat.id, fallback_bot_username).await?;
            dialogue.update(BotState::Initial).await?;
            start_handler(bot, dialogue, msg, api_client).await
        }
        Command::MyBots => {
            dialogue
                .update(BotState::WaitingForReferralBotToken)
                .await?;
            my_bots_handler(bot, dialogue, app_state).await
        }
    }
}

async fn start_redis_listener(
    bot: Bot,
    redis_url: String,
    bot_id: i64,
    api_client: Arc<BackendApi>,
) -> AppResult<()> {
    let channel = format!("bot-notifications:{bot_id}");
    tracing::info!("Subscribing to Redis channel: {channel}");

    let conn = redis::Client::open(redis_url)?;

    let mut pubsub = conn.get_async_pubsub().await?;
    pubsub.subscribe(&channel).await?;
    let mut msg_stream = pubsub.on_message();

    while let Some(msg) = msg_stream.next().await {
        if let Ok(payload_str) = msg.get_payload::<String>()
            && let Ok(parsed) = serde_json::from_str::<DispatchMessagePayload>(&payload_str)
        {
            let res = handle_msg(bot.clone(), parsed, api_client.clone()).await;
            if let Err(e) = res {
                tracing::error!("Error handling message: {e}");
            }
        }
    }

    Ok(())
}

async fn handle_msg(
    bot: Bot,
    payload: DispatchMessagePayload,
    api_client: Arc<BackendApi>,
) -> AppResult<()> {
    let chat_id = ChatId(payload.telegram_id);

    if let Some(image_id) = payload.image_id {
        let bytes = api_client.get_image_bytes(&image_id).await?;
        if let Err(e) = bot
            .send_photo(chat_id, InputFile::memory(bytes))
            .caption(payload.message)
            .parse_mode(ParseMode::Html)
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
        return Ok(());
    }

    if let Err(e) = bot
        .send_message(chat_id, payload.message)
        .parse_mode(ParseMode::Html)
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
    api_client: &Arc<BackendApi>,
) -> AppResult<(Bytes, String, Vec<String>)> {
    let captcha = api_client.get_captcha().await?;

    let image = STANDARD.decode(
        captcha
            .image_data
            .split_once(',')
            // TODO Refactor it
            .ok_or("Invalid data URL")
            .unwrap_or(("", ""))
            .1,
    )?;
    Ok((Bytes::from(image), captcha.answer, captcha.variants))
}
