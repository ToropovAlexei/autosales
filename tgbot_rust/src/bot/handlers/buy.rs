use std::sync::Arc;

use teloxide::dispatching::dialogue::GetChatId;
use teloxide::{Bot, types::CallbackQuery};

use crate::api::api_errors::ApiClientError;
use crate::bot::utils::{MessageImage, MsgBy, edit_msg};
use crate::{
    api::backend_api::BackendApi,
    bot::keyboards::back_to_main_menu::back_to_main_menu_inline_keyboard, errors::AppResult,
};

pub async fn buy_handler(
    bot: Bot,
    q: CallbackQuery,
    api_client: Arc<BackendApi>,
    product_id: i64,
) -> AppResult<()> {
    let chat_id = match q.chat_id() {
        Some(chat_id) => chat_id,
        None => return Ok(()),
    };

    let buy_result = api_client.buy_product(chat_id.0, product_id).await;

    let (msg, img) = match buy_result {
        Ok(response) => {
            let mut success_message = format!(
                "✅ Поздравляем! Вы успешно купили товар <b>{}</b> за <b>{} ₽</b>.\n\n💳 Ваш новый баланс: <b>{} ₽</b>",
                response.product_name, response.price, response.balance
            );

            if let Some(fulfilled_content) = response.fulfilled_text {
                success_message.push_str(&format!(
                    "\n\n<b>Ваш товар:</b>\n<pre>{}</pre>",
                    fulfilled_content
                ));
            }
            if let Some(details) = response.details {
                success_message.push_str(&format!("\n\n<b>Подробности:</b>\n{}", details));
            }
            (
                success_message,
                response.fulfilled_image_id.map(MessageImage::Uuid),
            )
        }
        Err(e) => {
            let msg = match e {
                ApiClientError::Unsuccessful(msg) => {
                    if msg.contains("Insufficient Balance") {
                        "😔 Недостаточно средств на балансе для совершения покупки. Пожалуйста, пополните баланс.".to_string()
                    } else if msg.contains("Product out of stock") {
                        "😔 К сожалению, этот товар закончился.".to_string()
                    } else {
                        format!("Произошла непредвиденная ошибка: {msg}")
                    }
                }
                _ => "Произошла непредвиденная ошибка. Попробуйте позже.".to_string(),
            };
            (msg, None)
        }
    };

    edit_msg(
        &api_client,
        &bot,
        &MsgBy::CallbackQuery(&q),
        &msg,
        img,
        back_to_main_menu_inline_keyboard(),
    )
    .await?;

    Ok(())
}
