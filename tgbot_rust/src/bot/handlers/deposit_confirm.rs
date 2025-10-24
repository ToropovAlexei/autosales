use std::sync::Arc;

use crate::api::backend_api::BackendApi;
use crate::bot::{BotState, InvoiceData, MyDialogue};
use crate::errors::AppResult;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode};
use url::Url;

pub async fn deposit_confirm_handler(
    bot: Bot,
    msg: Message,
    dialogue: MyDialogue,
    api_client: Arc<BackendApi>,
    bot_state: BotState,
) -> AppResult<()> {
    let (gateway, amount, invoice_data) = match bot_state {
        BotState::DepositConfirm {
            gateway,
            amount,
            invoice,
        } => (gateway, amount, invoice),
        _ => return Ok(()),
    };
    let telegram_id = dialogue.chat_id().0;
    let response = match api_client
        .create_deposit_invoice(&gateway, amount as f64, telegram_id)
        .await
    {
        Ok(res) => res,
        Err(err) => {
            bot.edit_message_text(
                ChatId(telegram_id),
                msg.id,
                format!("Не удалось создать счет: {}", err),
            )
            .send()
            .await?;

            return Ok(());
        }
    };

    let invoice_data = InvoiceData {
        order_id: response.order_id.clone(),
        pay_url: response.pay_url.clone(),
        details: response.details.clone(),
    };

    dialogue
        .update(BotState::DepositConfirm {
            gateway: gateway.clone(),
            amount,
            invoice: Some(invoice_data.clone()),
        })
        .await?;

    let text = if let Some(pay_url) = &invoice_data.pay_url {
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
    if let Some(pay_url) = &invoice_data.pay_url {
        if let Ok(pay_url) = Url::parse(pay_url) {
            keyboard.push(vec![InlineKeyboardButton::url("Оплатить", pay_url)]);
        }
    }
    keyboard.push(vec![InlineKeyboardButton::callback("⬅️ Назад", "deposit")]);

    bot.edit_message_text(msg.chat.id, msg.id, text)
        .reply_markup(InlineKeyboardMarkup::new(keyboard))
        .parse_mode(ParseMode::Html)
        .send()
        .await?;

    Ok(())
}
