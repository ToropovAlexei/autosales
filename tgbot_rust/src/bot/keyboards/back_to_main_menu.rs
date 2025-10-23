use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::bot::BotState;

pub fn back_to_main_menu_inline_keyboard() -> InlineKeyboardMarkup {
    let buttons = vec![[InlineKeyboardButton::callback(
        "⬅️ Главное меню",
        BotState::MainMenu,
    )]];
    InlineKeyboardMarkup::new(buttons)
}
