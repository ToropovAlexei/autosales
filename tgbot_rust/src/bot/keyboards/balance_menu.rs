use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::bot::BotState;

pub fn balance_menu_inline_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback(
            "💰 Пополнить баланс",
            BotState::Deposit,
        )],
        vec![InlineKeyboardButton::callback(
            "⬅️ Назад",
            BotState::MainMenu,
        )],
    ])
}
