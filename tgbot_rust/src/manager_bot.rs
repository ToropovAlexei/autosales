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
    tokio::spawn(async move {
        loop {
            let result = run_manager_bot(manager_bot_token.clone(), manager_api.clone()).await;

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

pub async fn run_manager_bot(bot_token: String, api: Arc<BackendApi>) -> AppResult<()> {
    let bot = Bot::new(bot_token);
    let me = bot.get_me().await?;
    let username = me.user.username.unwrap_or_default();
    tracing::info!("Starting manager bot: @{}", username);

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

    let callback_text = match action {
        ManagerCallbackAction::Approve {
            request_id,
            request_type,
        } => {
            api.complete_store_balance_request(request_id, tg_user_id)
                .await?;
            match request_type {
                Some(StoreBalanceRequestType::Withdrawal) => {
                    format!("Подтвержден вывод #{request_id}.")
                }
                Some(StoreBalanceRequestType::Deposit) => {
                    format!("Подтверждено пополнение #{request_id}.")
                }
                None => format!("Заявка #{request_id} подтверждена."),
            }
        }
        ManagerCallbackAction::Reject {
            request_id,
            request_type,
        } => {
            api.reject_store_balance_request(request_id, tg_user_id)
                .await?;
            match request_type {
                Some(StoreBalanceRequestType::Withdrawal) => {
                    format!("Отклонен вывод #{request_id}.")
                }
                Some(StoreBalanceRequestType::Deposit) => {
                    format!("Отклонено пополнение #{request_id}.")
                }
                None => format!("Заявка #{request_id} отклонена."),
            }
        }
    };

    if let Some(message) = q.regular_message()
        && let Err(err) = bot.delete_message(message.chat.id, message.id).await
    {
        tracing::warn!(error = %err, "Failed to delete processed manager message");
    }

    bot.answer_callback_query(q.id).text(callback_text).await?;

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
    Approve {
        request_id: i64,
        request_type: Option<StoreBalanceRequestType>,
    },
    Reject {
        request_id: i64,
        request_type: Option<StoreBalanceRequestType>,
    },
}

impl ManagerCallbackAction {
    fn parse(raw: &str) -> Option<Self> {
        let mut parts = raw.split(':');
        let prefix = parts.next()?;
        let action = parts.next()?;
        let request_id = parts.next()?.parse::<i64>().ok()?;
        let request_type = match parts.next() {
            Some("withdrawal") => Some(StoreBalanceRequestType::Withdrawal),
            Some("deposit") => Some(StoreBalanceRequestType::Deposit),
            Some(_) => return None,
            None => None,
        };
        if parts.next().is_some() {
            return None;
        }
        if prefix != "sbr" {
            return None;
        }
        match action {
            "approve" => Some(Self::Approve {
                request_id,
                request_type,
            }),
            "reject" => Some(Self::Reject {
                request_id,
                request_type,
            }),
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
    let request_type_callback = match request_type {
        StoreBalanceRequestType::Withdrawal => "withdrawal",
        StoreBalanceRequestType::Deposit => "deposit",
    };

    let message = format!(
        "Заявка #{request_id}\nТип: {request_type_text}\nСумма: {amount_usdt:.2} USDT (~{amount_rub:.2} RUB)\n\n{action_text}",
    );

    let keyboard = InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::callback(
            "Подтвердить",
            format!("sbr:approve:{request_id}:{request_type_callback}"),
        ),
        InlineKeyboardButton::callback(
            "Отклонить",
            format!("sbr:reject:{request_id}:{request_type_callback}"),
        ),
    ]]);

    (message, keyboard)
}

pub async fn dispatch_admin_message(
    config: Arc<Config>,
    api: Arc<BackendApi>,
    payload: DispatchAdminMessage,
) -> AppResult<()> {
    let Some(manager_bot_token) = config.manager_bot_token.clone() else {
        tracing::warn!("MANAGER_BOT_TOKEN is not set; admin notification skipped");
        return Ok(());
    };

    let settings = api.get_settings().await?;
    let Some(chat_id) = settings.manager_group_chat_id else {
        tracing::warn!("manager_group_chat_id is not set; admin notification skipped");
        return Ok(());
    };

    let bot = Bot::new(manager_bot_token);
    let (text, keyboard) = build_admin_request_message(&payload);
    bot.send_message(ChatId(chat_id), text)
        .reply_markup(keyboard)
        .await?;
    Ok(())
}
