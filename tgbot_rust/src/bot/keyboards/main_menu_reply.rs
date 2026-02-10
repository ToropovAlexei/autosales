use teloxide::types::{KeyboardButton, KeyboardMarkup};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainMenuReplyAction {
    Catalog,
    Balance,
    Support,
}

impl MainMenuReplyAction {
    pub fn label(self) -> &'static str {
        match self {
            Self::Catalog => "🛍 Каталог",
            Self::Balance => "💰 Баланс",
            Self::Support => "🛟 Поддержка",
        }
    }

    pub fn from_text(text: &str) -> Option<Self> {
        match text.trim() {
            "Каталог" | "🛍 Каталог" => Some(Self::Catalog),
            "Баланс" | "💰 Баланс" => Some(Self::Balance),
            "Поддержка" | "🛟 Поддержка" => Some(Self::Support),
            _ => None,
        }
    }
}

pub fn main_menu_reply_keyboard() -> KeyboardMarkup {
    KeyboardMarkup::new(vec![
        vec![
            KeyboardButton::new(MainMenuReplyAction::Catalog.label()),
            KeyboardButton::new(MainMenuReplyAction::Balance.label()),
        ],
        vec![KeyboardButton::new(MainMenuReplyAction::Support.label())],
    ])
    .resize_keyboard()
    .selective()
    .persistent()
}
