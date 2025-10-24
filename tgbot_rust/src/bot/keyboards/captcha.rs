use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::bot::CallbackData;

pub fn captcha_keyboard_inline(options: &[String]) -> InlineKeyboardMarkup {
    let buttons: Vec<Vec<InlineKeyboardButton>> = options
        .iter()
        .map(|option| {
            vec![InlineKeyboardButton::callback(
                option.clone(),
                CallbackData::AnswerCaptcha {
                    answer: option.into(),
                },
            )]
        })
        .collect();

    InlineKeyboardMarkup::new(buttons)
}
