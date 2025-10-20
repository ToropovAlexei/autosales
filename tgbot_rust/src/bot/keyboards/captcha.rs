use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub fn captcha_keyboard_inline(options: &[String]) -> InlineKeyboardMarkup {
    let buttons: Vec<Vec<InlineKeyboardButton>> = options
        .iter()
        .map(|option| {
            vec![InlineKeyboardButton::callback(
                option.clone(),
                format!("captcha_{}", option),
            )]
        })
        .collect();

    InlineKeyboardMarkup::new(buttons)
}
