use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::bot::CallbackData;

pub fn deposit_amount_menu() -> InlineKeyboardMarkup {
    let amounts = [500, 1000, 1500];

    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = amounts
        .iter()
        .map(|&amount| {
            vec![InlineKeyboardButton::callback(
                format!("{amount} ₽"),
                CallbackData::SelectAmount { amount },
            )]
        })
        .collect();

    keyboard.push(vec![InlineKeyboardButton::callback(
        "⬅️ Назад",
        CallbackData::ToDepositSelectGateway,
    )]);

    InlineKeyboardMarkup::new(keyboard)
}
