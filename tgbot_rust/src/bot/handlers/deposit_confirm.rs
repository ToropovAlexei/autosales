use std::sync::Arc;

use crate::api::backend_api::BackendApi;
use crate::bot::keyboards::back_to_main_menu::back_to_main_menu_inline_keyboard;
use crate::bot::{BotState, CallbackData, InvoiceData, MockDetails, MyDialogue};
use crate::errors::AppResult;
use teloxide::dispatching::dialogue::GetChatId;
use teloxide::prelude::*;
use teloxide::types::{
    InlineKeyboardButton, InlineKeyboardMarkup, MaybeInaccessibleMessage, ParseMode,
};
use url::Url;

pub async fn deposit_confirm_handler(
    bot: Bot,
    q: CallbackQuery,
    dialogue: MyDialogue,
    api_client: Arc<BackendApi>,
    bot_state: BotState,
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

    let (gateway, amount, _invoice_data) = match bot_state {
        BotState::DepositConfirm {
            gateway,
            amount,
            invoice,
        } => (gateway, amount, invoice),
        _ => {
            tracing::error!("Expected DepositConfirm bot state");
            return Ok(());
        }
    };
    let telegram_id = dialogue.chat_id().0;
    let response = match api_client
        .create_deposit_invoice(&gateway, amount as f64, telegram_id)
        .await
    {
        Ok(res) => res,
        Err(err) => {
            tracing::error!("Error creating invoice: {err}");
            bot.edit_message_text(
                ChatId(telegram_id),
                message_id,
                "Что-то пошло не так. Попробуйте ещё раз.",
            )
            .reply_markup(back_to_main_menu_inline_keyboard())
            .send()
            .await?;

            return Ok(());
        }
    };

    let invoice_data = InvoiceData {
        id: response.id,
        details: Some(response.payment_details.clone()),
    };

    dialogue
        .update(BotState::DepositConfirm {
            gateway: gateway.clone(),
            amount,
            invoice: Some(invoice_data.clone()),
        })
        .await?;

    let mock_details = serde_json::from_value::<MockDetails>(response.payment_details);

    let text = if let Ok(_pay_url) = &mock_details {
        format!(
            "✅ Ваш счет на {} ₽ создан.\n\nНажмите на кнопку ниже, чтобы перейти к оплате.",
            amount
        )
    } else if let Some(details) = &invoice_data.details {
        let bank_name = details
            .get("data_bank")
            .and_then(|b| b.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("N/A");
        let card_number = details
            .get("value")
            .and_then(|v| v.as_str())
            .unwrap_or("N/A");
        format!(
            "Реквизиты для оплаты:\n\nБанк: {}\nНомер карты: {}\nСумма: {} ₽\n\nПосле оплаты, пожалуйста, подождите. Статус платежа обновится автоматически.",
            bank_name, card_number, amount
        )
    } else {
        "Не удалось получить реквизиты для оплаты. Попробуйте другой способ.".into()
    };

    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];
    if let Ok(details) = &mock_details
        && let Ok(pay_url) = Url::parse(details.pay_url.as_str())
    {
        keyboard.push(vec![InlineKeyboardButton::url("Оплатить", pay_url)]);
    }

    keyboard.push(vec![InlineKeyboardButton::callback(
        "⬅️ Назад",
        CallbackData::ToMainMenu,
    )]);

    bot.edit_message_text(chat_id, message_id, text)
        .reply_markup(InlineKeyboardMarkup::new(keyboard))
        .parse_mode(ParseMode::Html)
        .send()
        .await?;

    Ok(())
}
