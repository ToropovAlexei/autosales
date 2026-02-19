use std::sync::Arc;

use shared_dtos::{balance_request::StoreBalanceRequestType, notification::DispatchAdminMessage};
use teloxide::{ApiError, RequestError};
use teloxide::{
    Bot,
    dispatching::{Dispatcher, UpdateFilterExt, dialogue::GetChatId},
    dptree,
    payloads::{AnswerCallbackQuerySetters, SendMessageSetters},
    prelude::Requester,
    types::{
        CallbackQuery, ChatId, ChatMemberUpdated, InlineKeyboardButton, InlineKeyboardMarkup,
        Message, Update,
    },
};
use tokio::time::{Duration, sleep};
use tokio_stream::StreamExt;

use crate::{api::backend_api::BackendApi, config::Config, errors::AppError, errors::AppResult};

pub fn spawn_manager_bot_supervisor(config: Arc<Config>) {
    let Some(manager_bot_token) = config.manager_bot_token.clone() else {
        tracing::info!("MANAGER_BOT_TOKEN is not set, manager bot is disabled");
        return;
    };

    let manager_api = Arc::new(
        BackendApi::new(&config.backend_api_url, &config.service_api_key, None)
            .expect("Failed to create manager bot BackendApi"),
    );
    let manager_redis_url = format!("redis://{}:{}", config.redis_host, config.redis_port);

    tokio::spawn(async move {
        loop {
            let result = run_manager_bot(
                manager_bot_token.clone(),
                manager_redis_url.clone(),
                manager_api.clone(),
            )
            .await;

            match result {
                Ok(_) => {
                    tracing::warn!("Manager bot exited normally, restarting in 1s");
                }
                Err(AppError::RequestError(RequestError::Api(ApiError::InvalidToken))) => {
                    tracing::error!(
                        "Manager bot token is invalid (401). Stopping manager bot supervisor.",
                    );
                    break;
                }
                Err(err) => {
                    tracing::error!(error = %err, "Manager bot failed, restarting in 1s");
                }
            }

            sleep(Duration::from_secs(1)).await;
        }
    });
}

pub async fn run_manager_bot(
    bot_token: String,
    redis_url: String,
    api: Arc<BackendApi>,
) -> AppResult<()> {
    let bot = Bot::new(bot_token);
    let me = bot.get_me().await?;
    let username = me.user.username.unwrap_or_default();
    tracing::info!("Starting manager bot: @{}", username);

    let listener_handle = tokio::spawn(start_admin_redis_listener(
        bot.clone(),
        redis_url,
        api.clone(),
    ));

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(on_message))
        .branch(Update::filter_callback_query().endpoint(on_callback_query))
        .branch(Update::filter_my_chat_member().endpoint(on_my_chat_member));

    Dispatcher::builder(bot.clone(), handler)
        .dependencies(dptree::deps![api])
        .default_handler(|upd| async move {
            tracing::debug!("Unhandled manager bot update: {upd:?}");
        })
        .error_handler(Arc::new(|err| {
            Box::pin(async move {
                tracing::error!(?err, "manager bot dispatcher error");
            })
        }))
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    listener_handle.abort();
    Ok(())
}

async fn on_message(msg: Message, api: Arc<BackendApi>) -> AppResult<()> {
    sync_manager_group_chat_id(msg.chat.id, api).await;
    Ok(())
}

async fn on_callback_query(bot: Bot, q: CallbackQuery, api: Arc<BackendApi>) -> AppResult<()> {
    if let Some(chat_id) = q.chat_id() {
        sync_manager_group_chat_id(chat_id, api.clone()).await;
    }

    let Some(data) = q.data.clone() else {
        return Ok(());
    };

    let Some(action) = ManagerCallbackAction::parse(&data) else {
        return Ok(());
    };

    let tg_user_id = q.from.id.0 as i64;

    match action {
        ManagerCallbackAction::Approve { request_id } => {
            api.complete_store_balance_request(request_id, tg_user_id)
                .await?;
        }
        ManagerCallbackAction::Reject { request_id } => {
            api.reject_store_balance_request(request_id, tg_user_id)
                .await?;
        }
    }

    if let Some(message) = q.regular_message()
        && let Err(err) = bot.delete_message(message.chat.id, message.id).await
    {
        tracing::warn!(error = %err, "Failed to delete processed manager message");
    }

    bot.answer_callback_query(q.id)
        .text("Действие выполнено")
        .await?;

    Ok(())
}

async fn on_my_chat_member(update: ChatMemberUpdated, api: Arc<BackendApi>) -> AppResult<()> {
    sync_manager_group_chat_id(update.chat.id, api).await;
    Ok(())
}

async fn sync_manager_group_chat_id(chat_id: ChatId, api: Arc<BackendApi>) {
    // Telegram group/supergroup chat ids are negative.
    if chat_id.0 >= 0 {
        return;
    }

    if let Err(err) = api.update_manager_group_chat_id(chat_id.0).await {
        tracing::warn!(
            chat_id = chat_id.0,
            error = %err,
            "Failed to sync manager_group_chat_id",
        );
    }
}

#[derive(Debug, Clone, Copy)]
enum ManagerCallbackAction {
    Approve { request_id: i64 },
    Reject { request_id: i64 },
}

impl ManagerCallbackAction {
    fn parse(raw: &str) -> Option<Self> {
        let mut parts = raw.split(':');
        let prefix = parts.next()?;
        let action = parts.next()?;
        let request_id = parts.next()?.parse::<i64>().ok()?;
        if parts.next().is_some() {
            return None;
        }
        if prefix != "sbr" {
            return None;
        }
        match action {
            "approve" => Some(Self::Approve { request_id }),
            "reject" => Some(Self::Reject { request_id }),
            _ => None,
        }
    }
}

fn build_admin_request_message(payload: &DispatchAdminMessage) -> (String, InlineKeyboardMarkup) {
    let (request_id, amount_rub, amount_usdt, request_type, action_text) = match payload {
        DispatchAdminMessage::StoreBalanceRequestNotification {
            store_balance_request_id,
            amount_in_rub,
            amount_in_usdt,
            r#type,
        } => (
            *store_balance_request_id,
            *amount_in_rub,
            *amount_in_usdt,
            *r#type,
            match r#type {
                StoreBalanceRequestType::Withdrawal => {
                    "Подтвердите, что выплата отправлена клиенту."
                }
                StoreBalanceRequestType::Deposit => {
                    "Подтвердите, что средства от клиента получены."
                }
            },
        ),
    };

    let request_type_text = match request_type {
        StoreBalanceRequestType::Withdrawal => "Вывод",
        StoreBalanceRequestType::Deposit => "Пополнение",
    };

    let message = format!(
        "Заявка #{request_id}\nТип: {request_type_text}\nСумма: {amount_usdt:.2} USDT (~{amount_rub:.2} RUB)\n\n{action_text}",
    );

    let keyboard = InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::callback("Подтвердить", format!("sbr:approve:{request_id}")),
        InlineKeyboardButton::callback("Отклонить", format!("sbr:reject:{request_id}")),
    ]]);

    (message, keyboard)
}

async fn start_admin_redis_listener(
    bot: Bot,
    redis_url: String,
    api: Arc<BackendApi>,
) -> AppResult<()> {
    let channel = "bot-admin-notifications";
    tracing::info!("Manager bot subscribing to Redis channel: {channel}");

    let conn = redis::Client::open(redis_url)?;
    let mut pubsub = conn.get_async_pubsub().await?;
    pubsub.subscribe(channel).await?;
    let mut msg_stream = pubsub.on_message();

    while let Some(msg) = msg_stream.next().await {
        let payload_str = match msg.get_payload::<String>() {
            Ok(payload) => payload,
            Err(err) => {
                tracing::warn!(error = %err, "Invalid admin notification payload");
                continue;
            }
        };

        let payload = match serde_json::from_str::<DispatchAdminMessage>(&payload_str) {
            Ok(payload) => payload,
            Err(err) => {
                tracing::warn!(error = %err, "Failed to decode admin notification json");
                continue;
            }
        };

        let settings = match api.get_settings().await {
            Ok(settings) => settings,
            Err(err) => {
                tracing::warn!(error = %err, "Failed to load settings for manager chat id");
                continue;
            }
        };

        let Some(chat_id) = settings.manager_group_chat_id else {
            tracing::warn!("manager_group_chat_id is not set; admin notification skipped");
            continue;
        };

        let (text, keyboard) = build_admin_request_message(&payload);
        if let Err(err) = bot
            .send_message(ChatId(chat_id), text)
            .reply_markup(keyboard)
            .await
        {
            tracing::warn!(error = %err, chat_id, "Failed to send manager notification");
        }
    }

    Ok(())
}
