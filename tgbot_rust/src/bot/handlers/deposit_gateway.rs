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
        MyDialogue,
        keyboards::{
            back_to_main_menu::back_to_main_menu_inline_keyboard,
            payment_gateways_menu::payment_gateways_menu,
        },
    },
    errors::AppResult,
};
use teloxide::dispatching::dialogue::GetChatId;
use teloxide::prelude::Requester;

pub async fn deposit_gateway_handler(
    bot: Bot,
    _dialogue: MyDialogue,
    q: CallbackQuery,
    _username: String,
    api_client: Arc<BackendApi>,
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

    let payment_gateways = api_client.get_payment_gateways().await;
    let public_settings = match api_client.get_settings().await {
        Ok(settings) => settings,
        Err(err) => {
            tracing::error!("Error getting settings: {err}");
            bot.edit_message_text(
                chat_id,
                message_id,
                "Что-то пошло не так. Попробуйте ещё раз",
            )
            .reply_markup(back_to_main_menu_inline_keyboard())
            .send()
            .await?;
            return Ok(());
        }
    };

    bot.edit_message_text(chat_id, message_id, "💰 Выберите способ пополнения:")
        .reply_markup(payment_gateways_menu(payment_gateways, public_settings))
        .parse_mode(ParseMode::Html)
        .send()
        .await?;
    Ok(())
}
