use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::bot::callback_data::CallbackData;

pub fn balance_menu_inline_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback(
            "💰 Пополнить баланс",
            CallbackData::Deposit,
        )],
        vec![InlineKeyboardButton::callback(
            "⬅️ Назад",
            CallbackData::MainMenu,
        )],
    ])
}
