use std::sync::Arc;

use shared_dtos::order::PurchaseDetails;
use shared_dtos::product::ProductDetails;
use shared_dtos::user_subscription::UserSubscriptionDetails;
use teloxide::dispatching::dialogue::GetChatId;
use teloxide::{
    Bot,
    types::CallbackQuery,
    utils::html::{bold, code_block},
};

use crate::api::api_errors::ApiClientError;
use crate::bot::MyDialogue;
use crate::bot::utils::{MessageImage, MsgBy, edit_msg};
use crate::{
    api::backend_api::BackendApi,
    bot::keyboards::back_to_main_menu::back_to_main_menu_inline_keyboard, errors::AppResult,
};

pub async fn buy_handler(
    bot: Bot,
    dialogue: MyDialogue,
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
            let price = format!("{:.2}", response.price);
            let balance = format!("{:.2}", response.balance);
            let mut success_message = format!(
                "{}\n\n{} {}\n{} {} ₽\n{} {} ₽",
                bold("✅ Покупка успешна"),
                bold("Товар:"),
                response.product_name,
                bold("Цена:"),
                price,
                bold("Баланс:"),
                balance,
            );

            if let Some(fulfilled_content) = response.fulfilled_text {
                success_message.push_str(&format!(
                    "\n\n{}{}\n{}",
                    bold("📦 Ваш товар"),
                    ":",
                    code_block(&fulfilled_content)
                ));
            }
            if let Some(details) = response.details {
                match details {
                    PurchaseDetails::ProductDetails(details) => match details {
                        ProductDetails::ContMs { host: _, port: _ } => {}
                    },
                    PurchaseDetails::UserSubscriptionDetails(details) => match details {
                        UserSubscriptionDetails::ContMs {
                            host,
                            port,
                            username,
                            password,
                        } => {
                            let address = format!("{}:{}", host, port);
                            let access =
                                format!("{}\nlogin: {}\npassword: {}", address, username, password);
                            success_message.push_str(&format!(
                                "\n\n{}{}\n{}",
                                bold("🔐 Доступ"),
                                ":",
                                code_block(&access)
                            ));
                        }
                    },
                }
            }
            (
                success_message,
                response.fulfilled_image_id.map(MessageImage::Uuid),
            )
        }
        Err(e) => {
            let msg = match e {
                ApiClientError::Unsuccessful(msg) => {
                    if msg.contains("Not enough balance") {
                        "😔 Недостаточно средств на балансе для совершения покупки. Пожалуйста, пополните баланс.".to_string()
                    } else if msg.contains("Not enough stock") {
                        "😔 К сожалению, этот товар закончился.".to_string()
                    } else {
                        "Произошла непредвиденная ошибка. Попробуйте позже".to_string()
                    }
                }
                _ => "Произошла непредвиденная ошибка. Попробуйте позже.".to_string(),
            };
            (msg, None)
        }
    };

    edit_msg(
        &api_client,
        &dialogue,
        &bot,
        &MsgBy::CallbackQuery(&q),
        &msg,
        img,
        back_to_main_menu_inline_keyboard(),
    )
    .await?;

    Ok(())
}
