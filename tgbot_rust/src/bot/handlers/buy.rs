use std::sync::Arc;

use teloxide::dispatching::dialogue::GetChatId;
use teloxide::prelude::Requester;
use teloxide::types::MaybeInaccessibleMessage;
use teloxide::{
    Bot,
    payloads::EditMessageTextSetters,
    prelude::Request,
    types::{CallbackQuery, ParseMode},
};

use crate::bot::BotUsername;
use crate::{
    api::backend_api::BackendApi,
    bot::keyboards::back_to_main_menu::back_to_main_menu_inline_keyboard, errors::AppResult,
};

pub async fn buy_handler(
    bot: Bot,
    q: CallbackQuery,
    api_client: Arc<BackendApi>,
    product_id: i64,
    bot_username: BotUsername,
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

    let buy_result = api_client
        .buy_product(chat_id.0, product_id, bot_username)
        .await;

    match buy_result {
        Ok(response) => {
            let mut success_message = format!(
                "✅ Поздравляем! Вы успешно купили товар <b>{}</b> за <b>{} ₽</b>.\n\n💳 Ваш новый баланс: <b>{} ₽</b>",
                response.product_name, response.product_price, response.balance
            );

            if let Some(fulfilled_content) = response.fulfilled_content {
                success_message.push_str(&format!(
                    "\n\n<b>Ваш товар:</b>\n<pre>{}</pre>",
                    fulfilled_content
                ));
            }

            bot.edit_message_text(chat_id, message_id, success_message)
                .parse_mode(ParseMode::Html)
                .reply_markup(back_to_main_menu_inline_keyboard())
                .send()
                .await?;
        }
        Err(e) => {
            let error_message = match e {
                crate::errors::AppError::BadRequest(msg) => {
                    if msg.contains("Insufficient Balance") {
                        "😔 Недостаточно средств на балансе для совершения покупки. Пожалуйста, пополните баланс.".to_string()
                    } else if msg.contains("Product out of stock") {
                        "😔 К сожалению, этот товар закончился.".to_string()
                    } else {
                        format!("Произошла непредвиденная ошибка: {}", msg)
                    }
                }
                _ => "Произошла непредвиденная ошибка. Попробуйте позже.".to_string(),
            };
            bot.edit_message_text(chat_id, message_id, error_message)
                .parse_mode(ParseMode::Html)
                .send()
                .await?;
        }
    }

    Ok(())
}
