use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::bot::CallbackData;

pub fn balance_menu_inline_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback(
            "💰 Пополнить баланс",
            CallbackData::ToDepositSelectGateway,
        )],
        vec![InlineKeyboardButton::callback(
            "⬅️ Назад",
            CallbackData::ToMainMenu,
        )],
    ])
}
