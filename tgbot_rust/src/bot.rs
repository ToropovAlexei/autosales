use std::{result::Result::Ok, sync::Arc, time::Duration};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use teloxide::{
    ApiError, Bot, RequestError,
    dispatching::{
        HandlerExt, UpdateFilterExt,
        dialogue::{RedisStorage, serializer::Json},
    },
    dptree,
    macros::BotCommands,
    prelude::{Dialogue, Dispatcher, Requester},
    types::{
        CallbackQuery, ChatId, InlineKeyboardButton, InlineKeyboardMarkup, Message, MessageId,
        Update,
    },
};
use tokio::time::interval;
use tokio_stream::StreamExt;

use crate::{
    AppState,
    api::backend_api::BackendApi,
    bot::{
        handlers::{
            balance::balance_handler, buy::buy_handler, cancel_invoice::cancel_invoice_handler,
            captcha_answer::captcha_answer_handler, catalog::catalog_handler,
            confirm_invoice::confirm_invoice_handler, deposit_amount::deposit_amount_handler,
            deposit_confirm::deposit_confirm_handler, deposit_gateway::deposit_gateway_handler,
            fallback_bot_msg::fallback_bot_msg, main_menu::main_menu_handler,
            my_bots::my_bots_handler, my_orders::my_orders_handler,
            my_payments::my_payments_handler, my_subscriptions::my_subscriptions_handler,
            order_details::order_details_handler, product::product_handler,
            receipt_submitted_handler::receipt_submitted_handler, start::start_handler,
            support::support_handler,
        },
        keyboards::back_to_main_menu::back_to_main_menu_inline_keyboard,
        utils::{MessageImage, send_msg},
    },
    errors::{AppError, AppResult},
    models::{
        DispatchMessage, DispatchMessagePayload,
        customer::UpdateCustomerRequest,
        payment::{PaymentDetails, PaymentSystem},
    },
};

pub mod handlers;
pub mod utils;

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
    pub details: Option<PaymentDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MockDetails {
    pub pay_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(tag = "t", content = "c")]
pub enum BotStep {
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
    MyPayments,
    ReferralProgram,
    Support,
    MainMenu,
    Product {
        id: i64,
    },
    ReceiptRequested {
        invoice_id: i64,
    },
    ReceiptSubmitted {
        invoice_id: i64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct BotState {
    pub last_bot_msg_id: Option<i64>,
    pub last_user_msg_id: Option<i64>,
    pub step: BotStep,
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
    ToMyPayments,
    ToReferralProgram,
    ToSupport,
    ToProduct { id: i64 },
    ToDepositConfirm { id: i64 },
    ToOrderDetails { id: i64 },
    Buy { id: i64 },
    ConfirmPayment { id: i64 },
    CancelPayment { id: i64 },
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
    fallback_bot_username: Option<BotUsername>,
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
        storage.clone(),
    ));

    let handler = Update::filter_message()
        .enter_dialogue::<Message, RedisStorage<Json>, BotState>()
        .branch(
            dptree::entry()
                .filter_command::<Command>()
                .endpoint(command_handler),
        )
        .branch(
            dptree::filter(|state: BotState| state.step == BotStep::WaitingForReferralBotToken)
                .endpoint(handlers::my_bots::referral_bot_token_handler),
        )
        .branch(
            dptree::filter(|state: BotState| {
                matches!(state.step, BotStep::ReceiptRequested { .. })
            })
            .endpoint(receipt_submitted_handler),
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

            let telegram_id = q.from.id;
            api_client
                .update_customer_last_seen(telegram_id.0 as i64)
                .await?;

            match data {
                CallbackData::AnswerCaptcha { .. } => {
                    captcha_answer_handler(bot, dialogue, q, api_client).await?;
                }
                CallbackData::SelectGateway { gateway } => {
                    dialogue
                        .update(BotState {
                            step: BotStep::DepositSelectAmount { gateway },
                            ..bot_state
                        })
                        .await
                        .map_err(AppError::from)?;
                    deposit_amount_handler(bot, dialogue, q, api_client, bot_state).await?;
                }
                CallbackData::SelectAmount { amount } => {
                    let gateway = match &bot_state.step {
                        BotStep::DepositSelectAmount { gateway } => gateway.clone(),
                        _ => return Ok(()),
                    };
                    let new_state = BotState {
                        step: BotStep::DepositConfirm {
                            gateway,
                            amount,
                            invoice: None,
                        },
                        ..bot_state
                    };
                    dialogue
                        .update(new_state.clone())
                        .await
                        .map_err(AppError::from)?;
                    deposit_confirm_handler(bot, q, dialogue, api_client, new_state).await?;
                }
                CallbackData::ToMainMenu => {
                    dialogue
                        .update(BotState {
                            step: BotStep::MainMenu,
                            ..bot_state
                        })
                        .await
                        .map_err(AppError::from)?;
                    main_menu_handler(bot, dialogue, q, api_client).await?;
                }
                CallbackData::ToDepositSelectGateway => {
                    dialogue
                        .update(BotState {
                            step: BotStep::DepositSelectGateway,
                            ..bot_state
                        })
                        .await
                        .map_err(AppError::from)?;
                    deposit_gateway_handler(bot, dialogue, q, api_client).await?;
                }
                CallbackData::ToBalance => {
                    dialogue
                        .update(BotState {
                            step: BotStep::Balance,
                            ..bot_state
                        })
                        .await
                        .map_err(AppError::from)?;
                    balance_handler(bot, dialogue, q, api_client).await?;
                }
                CallbackData::ToMyOrders => {
                    dialogue
                        .update(BotState {
                            step: BotStep::MyOrders,
                            ..bot_state
                        })
                        .await
                        .map_err(AppError::from)?;
                    my_orders_handler(bot, dialogue, q, api_client).await?;
                }
                CallbackData::ToMyPayments => {
                    dialogue
                        .update(BotState {
                            step: BotStep::MyPayments,
                            ..bot_state
                        })
                        .await
                        .map_err(AppError::from)?;
                    my_payments_handler(bot, dialogue, q, api_client).await?;
                }
                CallbackData::ToMySubscriptions => {
                    dialogue
                        .update(BotState {
                            step: BotStep::MySubscriptions,
                            ..bot_state
                        })
                        .await
                        .map_err(AppError::from)?;
                    my_subscriptions_handler(bot, dialogue, q, api_client).await?;
                }
                CallbackData::ToReferralProgram => {
                    dialogue
                        .update(BotState {
                            step: BotStep::ReferralProgram,
                            ..bot_state
                        })
                        .await
                        .map_err(AppError::from)?;
                }
                CallbackData::ToSupport => {
                    dialogue
                        .update(BotState {
                            step: BotStep::Support,
                            ..bot_state
                        })
                        .await
                        .map_err(AppError::from)?;
                    support_handler(bot, dialogue, q, api_client).await?;
                }
                CallbackData::ToCategory { category_id } => {
                    dialogue
                        .update(BotState {
                            step: BotStep::Category { category_id },
                            ..bot_state
                        })
                        .await
                        .map_err(AppError::from)?;
                    catalog_handler(bot, dialogue, q, api_client, category_id).await?;
                }
                CallbackData::ToProduct { id } => {
                    dialogue
                        .update(BotState {
                            step: BotStep::Product { id },
                            ..bot_state
                        })
                        .await
                        .map_err(AppError::from)?;
                    product_handler(bot, dialogue, q, api_client, id).await?;
                }
                CallbackData::ToDepositConfirm { id } => {
                    let invoice = api_client.get_invoice(id).await?;
                    let new_state = BotState {
                        step: BotStep::DepositConfirm {
                            gateway: invoice.gateway,
                            amount: invoice.amount as i64,
                            invoice: Some(InvoiceData {
                                id,
                                details: invoice.payment_details,
                            }),
                        },
                        ..bot_state
                    };
                    dialogue
                        .update(new_state.clone())
                        .await
                        .map_err(AppError::from)?;
                    deposit_confirm_handler(bot, q, dialogue, api_client, new_state).await?;
                }
                CallbackData::ToOrderDetails { id } => {
                    order_details_handler(bot, dialogue, q, api_client, id).await?;
                }
                CallbackData::Buy { id } => {
                    buy_handler(bot, dialogue, q, api_client, id).await?;
                }
                CallbackData::CancelPayment { id } => {
                    cancel_invoice_handler(bot, dialogue, q, api_client, id).await?;
                }
                CallbackData::ConfirmPayment { id } => {
                    confirm_invoice_handler(bot, dialogue, q, api_client, id).await?;
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
    .error_handler(Arc::new(|err| {
        Box::pin(async move {
            tracing::error!(?err, "dispatcher error");
        })
    }))
    .enable_ctrlc_handler()
    .build();

    let health_check_fut = async {
        let mut interval = interval(Duration::from_secs(30));
        let mut retries = 0u32;

        loop {
            interval.tick().await;

            match bot.get_me().await {
                Ok(_) => {
                    retries = 0;
                }
                Err(e) => {
                    tracing::error!("[Bot][{bot_id}] Health check failed: {}", e);

                    if let RequestError::Api(ApiError::InvalidToken) = e {
                        tracing::error!(
                            "Received 401 Unauthorized — bot token is invalid. Exiting..."
                        );
                        return Err(e);
                    };

                    retries += 1;
                    if retries >= 5 {
                        tracing::error!(
                            "[Bot][{bot_id}] Too many health check failures. Exiting..."
                        );
                        return Err(e);
                    }
                }
            }
        }
    };

    let dispatch_fut = dispatcher.dispatch();

    tokio::select! {
        result = dispatch_fut => {
            tracing::info!("[Bot][{bot_id}] Dispatcher stopped: {:?}", result);
        },
        result = health_check_fut => {
            match result {
                Ok(()) => unreachable!(),
                Err(e) => {
                    if let RequestError::Api(ApiError::InvalidToken) = e {
                        return Err(AppError::BotUnauthorized(e.to_string()));
                    };
                    return Err(AppError::BotHealthcheckFailed(e.to_string()));
                }
            }
        }
    }

    listener_handle.abort();
    bot.delete_webhook().await?;
    Ok(())
}

async fn command_handler(
    bot: Bot,
    msg: Message,
    cmd: Command,
    dialogue: MyDialogue,
    api_client: Arc<BackendApi>,
    app_state: AppState,
    fallback_bot_username: Option<BotUsername>,
) -> AppResult<()> {
    match cmd {
        Command::Start => {
            if let Some(fallback_bot_username) = fallback_bot_username {
                fallback_bot_msg(bot.clone(), msg.chat.id, fallback_bot_username).await?;
            }
            dialogue
                .update(BotState {
                    step: BotStep::Initial,
                    // TODO NOT DEFAULT!
                    ..Default::default()
                })
                .await?;
            start_handler(bot, dialogue, msg, api_client).await
        }
        Command::MyBots => {
            dialogue
                .update(BotState {
                    step: BotStep::WaitingForReferralBotToken,
                    // TODO NOT DEFAULT!
                    ..Default::default()
                })
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
    storage: Arc<RedisStorage<Json>>,
) -> AppResult<()> {
    let channel = format!("bot-notifications:{bot_id}");
    tracing::info!("Subscribing to Redis channel: {channel}");

    let conn = redis::Client::open(redis_url)?;

    let mut pubsub = conn.get_async_pubsub().await?;
    pubsub.subscribe(&channel).await?;
    let mut msg_stream = pubsub.on_message();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<DispatchMessagePayload>();

    tokio::spawn(async move {
        while let Some(payload) = rx.recv().await {
            let res = handle_msg(bot.clone(), payload, api_client.clone(), storage.clone()).await;
            if let Err(e) = res {
                tracing::error!("Error handling message: {e}");
            }
            // Because of the telegram rate limit https://core.telegram.org/bots/faq#my-bot-is-hitting-limits-how-do-i-avoid-this
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });

    while let Some(msg) = msg_stream.next().await {
        if let Ok(payload_str) = msg.get_payload::<String>()
            && let Ok(parsed) = serde_json::from_str::<DispatchMessagePayload>(&payload_str)
            && let Err(e) = tx.send(parsed)
        {
            tracing::error!("Error sending message: {e}");
        }
    }

    Ok(())
}

async fn handle_msg(
    bot: Bot,
    payload: DispatchMessagePayload,
    api_client: Arc<BackendApi>,
    storage: Arc<RedisStorage<Json>>,
) -> AppResult<()> {
    let chat_id = ChatId(payload.telegram_id);
    let dialogue = MyDialogue::new(storage, chat_id);
    let state = dialogue.get_or_default().await.unwrap_or_default();

    if let Some(msg_id) = state.last_bot_msg_id {
        // Ignore error is ok
        let _ = bot.delete_message(chat_id, MessageId(msg_id as i32)).await;
    }

    let (msg, img, keyboard) = match payload.message {
        DispatchMessage::GenericMessage { image_id, message } => (
            message,
            image_id.map(MessageImage::Uuid),
            back_to_main_menu_inline_keyboard(),
        ),
        DispatchMessage::InvoiceTroublesNotification { amount, invoice_id } => (
            format!(
                "Вы недавно пытались пополнить баланс на {amount} ₽. Возникли ли у вас какие-либо проблемы с оплатой?"
            ),
            None,
            InlineKeyboardMarkup::new(vec![
                vec![InlineKeyboardButton::callback(
                    "Я все оплатил",
                    CallbackData::ConfirmPayment { id: invoice_id },
                )],
                vec![InlineKeyboardButton::callback(
                    "Отменить платеж",
                    CallbackData::CancelPayment { id: invoice_id },
                )],
            ]),
        ),
        DispatchMessage::RequestReceiptNotification { invoice_id } => {
            dialogue
                .update(BotState {
                    step: BotStep::ReceiptRequested { invoice_id },
                    ..state
                })
                .await?;
            (
            "Не видим Ваш платеж. Пожалуйста, загрузите чек (в формате jpg или pdf) и отправьте нам.".to_string(),
            None,
            back_to_main_menu_inline_keyboard(),
            )
        }
    };

    if let Err(AppError::RequestError(RequestError::Api(ApiError::BotBlocked))) =
        send_msg(&api_client, &dialogue, &bot, &msg, img, keyboard).await
    {
        tracing::info!("Bot is blocked by user: {}", payload.telegram_id);
        api_client
            .update_customer(
                payload.telegram_id,
                &UpdateCustomerRequest {
                    bot_is_blocked_by_user: Some(true),
                    ..Default::default()
                },
            )
            .await?;
    };

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
