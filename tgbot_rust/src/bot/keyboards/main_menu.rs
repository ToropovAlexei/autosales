use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::bot::{BotState, CategoryAction};

pub fn main_menu_inline_keyboard(show_referral_program: bool) -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![
        vec![InlineKeyboardButton::callback(
            "🛍️ Каталог",
            BotState::Category {
                action: CategoryAction::View,
                category_id: 0,
            },
        )],
        vec![InlineKeyboardButton::callback(
            "💳 Баланс",
            BotState::Balance,
        )],
        vec![InlineKeyboardButton::callback(
            "🧾 Мои заказы",
            BotState::MyOrders,
        )],
        vec![InlineKeyboardButton::callback(
            "🧾 Мои подписки",
            BotState::MySubscriptions,
        )],
        vec![InlineKeyboardButton::callback(
            "💰 Пополнить баланс",
            BotState::Deposit,
        )],
    ];

    if show_referral_program {
        keyboard.push(vec![InlineKeyboardButton::callback(
            "🤝 Реферальный магазин",
            BotState::ReferralProgram,
        )]);
    }

    keyboard.push(vec![InlineKeyboardButton::callback(
        "💬 Поддержка",
        BotState::Support,
    )]);

    InlineKeyboardMarkup::new(keyboard)
}
