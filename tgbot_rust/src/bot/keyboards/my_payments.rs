use shared_dtos::invoice::PaymentInvoiceBotResponse;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::bot::CallbackData;

pub fn my_payments_inline_keyboard(
    active_payments: &Vec<&PaymentInvoiceBotResponse>,
) -> InlineKeyboardMarkup {
    let mut buttons = active_payments
        .iter()
        .map(|payment| {
            vec![InlineKeyboardButton::callback(
                format!("Посмотреть счет #{}", payment.id),
                CallbackData::ToDepositConfirm { id: payment.id },
            )]
        })
        .collect::<Vec<_>>();

    buttons.push(vec![InlineKeyboardButton::callback(
        "⬅️ Назад",
        CallbackData::ToMainMenu,
    )]);

    InlineKeyboardMarkup::new(buttons)
}
