use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::bot::CallbackData;

pub fn deposit_amount_menu() -> InlineKeyboardMarkup {
    let amounts = [100, 500, 1000];

    let keyboard: Vec<Vec<InlineKeyboardButton>> = amounts
        .iter()
        .map(|&amount| {
            vec![InlineKeyboardButton::callback(
                format!("{amount} ₽"),
                CallbackData::SelectAmount { amount },
            )]
        })
        .collect();

    InlineKeyboardMarkup::new(keyboard)
}
