use serde::{Deserialize, Serialize};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CategoryCallback {
    pub action: String,
    pub category_id: u32,
    pub parent_id: u32,
}

impl CategoryCallback {
    pub fn pack(&self) -> String {
        format!(
            "cat:action={}&category_id={}&parent_id={}",
            self.action, self.category_id, self.parent_id
        )
    }

    pub fn unpack(data: &str) -> Option<Self> {
        if !data.starts_with("cat:") {
            return None;
        }

        let query = &data[4..];
        let parsed: Result<CategoryCallback, _> = serde_qs::from_str(query);
        parsed.ok()
    }
}

pub fn main_menu_inline_keyboard(show_referral_program: bool) -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![
        vec![InlineKeyboardButton::callback(
            "🛍️ Каталог",
            CategoryCallback {
                action: "view".to_string(),
                category_id: 0,
                ..Default::default()
            }
            .pack(),
        )],
        vec![InlineKeyboardButton::callback("💳 Баланс", "balance")],
        vec![InlineKeyboardButton::callback("🧾 Мои заказы", "my_orders")],
        vec![InlineKeyboardButton::callback(
            "🧾 Мои подписки",
            "my_subscriptions",
        )],
        vec![InlineKeyboardButton::callback(
            "💰 Пополнить баланс",
            "deposit",
        )],
    ];

    if show_referral_program {
        keyboard.push(vec![InlineKeyboardButton::callback(
            "🤝 Реферальный магазин",
            "referral_program",
        )]);
    }

    keyboard.push(vec![InlineKeyboardButton::callback(
        "💬 Поддержка",
        "support",
    )]);

    InlineKeyboardMarkup::new(keyboard)
}
