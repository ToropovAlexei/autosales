use shared_dtos::bot::BotBotResponse;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::bot::CallbackData;

pub fn my_bots_inline_keyboard(bots: &[BotBotResponse]) -> InlineKeyboardMarkup {
    let mut buttons: Vec<Vec<InlineKeyboardButton>> = Vec::new();
    bots.iter().for_each(|bot| {
        let status = match bot.is_primary {
            true => "(Основной)",
            false => match bot.is_active {
                true => "(Активен)",
                false => "(Неактивен)",
            },
        };
        buttons.push(vec![InlineKeyboardButton::callback(
            format!("@{} {status} - {}%", bot.username, bot.referral_percentage),
            CallbackData::ShowBotInfo { id: bot.id },
        )]);
        let mut action_buttons = Vec::new();
        if !bot.is_primary {
            action_buttons.push(InlineKeyboardButton::callback(
                "Сделать основным",
                CallbackData::SetBotPrimary { id: bot.id },
            ))
        }
        action_buttons.push(InlineKeyboardButton::callback(
            "Удалить",
            CallbackData::DeleteBot { id: bot.id },
        ));
        buttons.push(action_buttons);
    });

    if bots.len() < 3 {
        buttons.push(vec![InlineKeyboardButton::callback(
            "➕ Добавить бота",
            CallbackData::AddBot,
        )]);
    }

    buttons.push(vec![InlineKeyboardButton::callback(
        "📊 Статистика",
        CallbackData::BotStats,
    )]);
    buttons.push(vec![InlineKeyboardButton::callback(
        "⬅️ Назад",
        CallbackData::ToMainMenu,
    )]);

    InlineKeyboardMarkup::new(buttons)
}
