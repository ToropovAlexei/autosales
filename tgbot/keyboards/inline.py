from aiogram.types import InlineKeyboardMarkup, InlineKeyboardButton
from typing import Optional

def main_menu(referral_program_enabled: bool = False, fallback_bot_username: Optional[str] = None):
    buttons = [
        [InlineKeyboardButton(text="🛍️ Каталог", callback_data="catalog")],
        [InlineKeyboardButton(text="💰 Пополнить баланс", callback_data="deposit")],
        [InlineKeyboardButton(text="💳 Баланс", callback_data="balance")],
    ]
    if referral_program_enabled:
        buttons.append([InlineKeyboardButton(text="🤝 Реферальный магазин", callback_data="referral_program")])
    
    buttons.append([InlineKeyboardButton(text="💬 Поддержка", callback_data="support")])

    if fallback_bot_username:
        buttons.append([InlineKeyboardButton(text="🤖 Резервный бот", url=f"https://t.me/{fallback_bot_username}")])

    return InlineKeyboardMarkup(inline_keyboard=buttons)

def deposit_menu():
    buttons = [
        [InlineKeyboardButton(text="100 ₽", callback_data="deposit_100")],
        [InlineKeyboardButton(text="500 ₽", callback_data="deposit_500")],
        [InlineKeyboardButton(text="1000 ₽", callback_data="deposit_1000")],
        [InlineKeyboardButton(text="⬅️ Назад", callback_data="main_menu")]
    ]
    return InlineKeyboardMarkup(inline_keyboard=buttons)

def categories_menu(categories: list):
    buttons = []
    for category in categories:
        buttons.append([InlineKeyboardButton(text=category['name'], callback_data=f"category_{category['id']}")])
    buttons.append([InlineKeyboardButton(text="⬅️ Назад", callback_data="main_menu")])
    return InlineKeyboardMarkup(inline_keyboard=buttons)

def products_menu(products: list, category_id: int):
    buttons = []
    for product in products:
        buttons.append([InlineKeyboardButton(text=f"{product['name']} - {product['price']} ₽", callback_data=f"product_{product['id']}_{category_id}")])
    buttons.append([InlineKeyboardButton(text="⬅️ Назад к категориям", callback_data="catalog")])
    return InlineKeyboardMarkup(inline_keyboard=buttons)

def product_card(product: dict):
    buttons = [
        [InlineKeyboardButton(text="✅ Купить", callback_data=f"buy_{product['id']}")],
        [InlineKeyboardButton(text="⬅️ Назад к товарам", callback_data=f"category_{product['category_id']}")]
    ]
    return InlineKeyboardMarkup(inline_keyboard=buttons)