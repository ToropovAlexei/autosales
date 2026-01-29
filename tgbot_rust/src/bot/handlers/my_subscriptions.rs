use crate::{
    api::backend_api::BackendApi, bot::MyDialogue,
    bot::keyboards::back_to_main_menu::back_to_main_menu_inline_keyboard, errors::AppResult,
};
use std::sync::Arc;
use teloxide::payloads::EditMessageTextSetters;
use teloxide::{
    dispatching::dialogue::GetChatId,
    prelude::{Bot, Request, Requester},
    types::{CallbackQuery, MaybeInaccessibleMessage, ParseMode},
    utils::html::{bold, code_block, italic},
};

pub async fn my_subscriptions_handler(
    bot: Bot,
    _dialogue: MyDialogue,
    q: CallbackQuery,
    api_client: Arc<BackendApi>,
) -> AppResult<()> {
    let chat_id = match q.chat_id() {
        Some(chat_id) => chat_id,
        None => return Ok(()),
    };
    let message_id = match &q.message {
        Some(MaybeInaccessibleMessage::Regular(msg)) => msg.id,
        Some(MaybeInaccessibleMessage::Inaccessible(_)) => return Ok(()),
        None => return Ok(()),
    };

    match api_client.get_user_subscriptions(chat_id.0).await {
        Ok(subscriptions) => {
            if subscriptions.items.is_empty() {
                bot.edit_message_text(chat_id, message_id, "У вас пока нет активных подписок.")
                    .reply_markup(back_to_main_menu_inline_keyboard())
                    .send()
                    .await?;
                return Ok(());
            }

            let mut response_text = format!("{}\n\n", bold("🧾 Ваши подписки:"));

            for sub in subscriptions.items {
                let product_name = sub.product_name;
                let expires_formatted = sub.expires_at.format("%d.%m.%Y %H:%M").to_string();
                let status = if sub.expires_at > chrono::Utc::now() {
                    "✅ Активна до"
                } else {
                    "❌ Неактивна"
                };

                response_text.push_str(&format!("🔹 {}\n", bold(&product_name)));
                response_text.push_str(&format!("   {} {}\n", status, italic(&expires_formatted)));

                if let Some(details) = sub.details
                    && let Some(details_map) = details.as_object()
                {
                    response_text.push_str(&format!("   {}\n", bold("Данные для доступа:")));
                    if let Some(username) = details_map.get("username").and_then(|v| v.as_str()) {
                        response_text
                            .push_str(&format!("     - Логин: {}\n", code_block(username)));
                    }
                    if let Some(password) = details_map.get("password").and_then(|v| v.as_str()) {
                        response_text
                            .push_str(&format!("     - Пароль: {}\n", code_block(password)));
                    }
                }

                response_text.push('\n');
            }

            bot.edit_message_text(chat_id, message_id, response_text)
                .parse_mode(ParseMode::Html)
                .reply_markup(back_to_main_menu_inline_keyboard())
                .send()
                .await?;
        }
        Err(err) => {
            tracing::error!("Error getting user subscriptions: {err}");
            bot.edit_message_text(
                chat_id,
                message_id,
                "Произошла ошибка при получении подписок. Попробуйте позже.",
            )
            .parse_mode(ParseMode::Html)
            .reply_markup(back_to_main_menu_inline_keyboard())
            .send()
            .await?;
        }
    }

    Ok(())
}
