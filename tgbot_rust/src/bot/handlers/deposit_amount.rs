use std::sync::Arc;

use teloxide::{
    Bot,
    payloads::EditMessageTextSetters,
    prelude::Request,
    types::{CallbackQuery, MaybeInaccessibleMessage, ParseMode},
};

use crate::{
    api::backend_api::BackendApi,
    bot::{
        BotState, MyDialogue, PaymentAction, keyboards::deposit_amount_menu::deposit_amount_menu,
    },
    errors::AppResult,
};
use teloxide::dispatching::dialogue::GetChatId;
use teloxide::prelude::Requester;

pub async fn deposit_amount_handler(
    bot: Bot,
    _dialogue: MyDialogue,
    q: CallbackQuery,
    _username: String,
    _api_client: Arc<BackendApi>,
) -> AppResult<()> {
    let chat_id = match q.chat_id() {
        Some(chat_id) => chat_id,
        None => {
            tracing::error!("No chat id found");
            return Ok(());
        }
    };
    let message_id = match &q.message {
        Some(MaybeInaccessibleMessage::Regular(msg)) => msg.id,
        Some(MaybeInaccessibleMessage::Inaccessible(_)) => {
            tracing::error!("Inaccessible message found");
            return Ok(());
        }
        None => {
            tracing::error!("No message found");
            return Ok(());
        }
    };

    let gateway = match BotState::from_query(&q) {
        Some(data) => match data {
            BotState::Payment { action } => match action {
                PaymentAction::SelectGateway { gateway } => gateway,
                _ => {
                    tracing::error!("Invalid bot state");
                    bot.send_message(chat_id, "Что-то пошло не так. Попробуйте ещё раз")
                        .send()
                        .await?;
                    return Ok(());
                }
            },
            _ => {
                tracing::error!("Invalid bot state");
                bot.send_message(chat_id, "Что-то пошло не так. Попробуйте ещё раз")
                    .send()
                    .await?;
                return Ok(());
            }
        },
        None => {
            tracing::error!("No callback data");
            bot.send_message(chat_id, "Что-то пошло не так. Попробуйте ещё раз")
                .send()
                .await?;
            return Ok(());
        }
    };

    bot.edit_message_text(chat_id, message_id, "Выберите сумму для пополнения:")
        .reply_markup(deposit_amount_menu(&gateway))
        .parse_mode(ParseMode::Html)
        .send()
        .await?;
    Ok(())
}
