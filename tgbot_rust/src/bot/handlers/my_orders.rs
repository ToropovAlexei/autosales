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
    utils::html::{bold, italic},
};

pub async fn my_orders_handler(
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

    match api_client.get_user_orders(chat_id.0).await {
        Ok(orders) => {
            if orders.is_empty() {
                bot.edit_message_text(chat_id, message_id, "У вас пока нет заказов.")
                    .reply_markup(back_to_main_menu_inline_keyboard())
                    .send()
                    .await?;
                return Ok(());
            }

            let mut response_text = format!("{}\n\n", bold("🧾 Ваши заказы:"));

            for order in orders {
                let product_name = order.product_name;
                let created_formatted = order.created_at.format("%d.%m.%Y %H:%M").to_string();

                response_text.push_str(&format!(
                    "🔹 {} - {} ₽\n",
                    bold(&product_name),
                    order.amount
                ));
                response_text.push_str(&format!("   {}\n", italic(&created_formatted)));

                if let Some(fulfilled_content) = order.fulfilled_content {
                    response_text.push_str(&format!(
                        "   {}\n<pre>{}</pre>\n",
                        bold("Ваш товар:"),
                        fulfilled_content
                    ));
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
            tracing::error!("Error getting user orders: {err}");
            bot.edit_message_text(
                chat_id,
                message_id,
                "Произошла ошибка при получении заказов. Попробуйте позже.",
            )
            .parse_mode(ParseMode::Html)
            .reply_markup(back_to_main_menu_inline_keyboard())
            .send()
            .await?;
        }
    }

    Ok(())
}
