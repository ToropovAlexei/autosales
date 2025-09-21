from aiogram.types import InlineKeyboardMarkup, InlineKeyboardButton
from aiogram.filters.callback_data import CallbackData
from typing import Optional

# Фабрика колбэков для навигации по категориям
class CategoryCallback(CallbackData, prefix="cat"):
    action: str  # "view", "back"
    category_id: int = 0
    parent_id: int = 0 # ID родителя, чтобы знать, куда возвращаться

def main_menu(referral_program_enabled: bool = False, fallback_bot_username: Optional[str] = None):
    buttons = [
        [InlineKeyboardButton(text="🛍️ Каталог", callback_data=CategoryCallback(action="view", category_id=0).pack())],
        [InlineKeyboardButton(text="💰 Пополнить баланс", callback_data="deposit")],
        [InlineKeyboardButton(text="💳 Баланс", callback_data="balance")],
    ]
    if referral_program_enabled:
        buttons.append([InlineKeyboardButton(text="🤝 Реферальный магазин", callback_data="referral_program")])
    
    buttons.append([InlineKeyboardButton(text="💬 Поддержка", callback_data="support")])

    return InlineKeyboardMarkup(inline_keyboard=buttons)

def deposit_menu():
    buttons = [
        [InlineKeyboardButton(text="100 ₽", callback_data="deposit_100")],
        [InlineKeyboardButton(text="500 ₽", callback_data="deposit_500")],
        [InlineKeyboardButton(text="1000 ₽", callback_data="deposit_1000")],
        [InlineKeyboardButton(text="⬅️ Назад", callback_data="main_menu")]
    ]
    return InlineKeyboardMarkup(inline_keyboard=buttons)

def categories_menu(categories: list, parent_id: int = 0):
    buttons = []
    for category in categories:
        buttons.append([InlineKeyboardButton(
            text=category['name'], 
            callback_data=CategoryCallback(action="view", category_id=category['id'], parent_id=parent_id).pack()
        )])
    
    if parent_id == 0:
        # Если мы в корне, кнопка "Назад" ведет в главное меню
        buttons.append([InlineKeyboardButton(text="⬅️ Назад", callback_data="main_menu")])
    else:
        # Иначе, кнопка "Назад" ведет к родительской категории
        buttons.append([InlineKeyboardButton(
            text="⬅️ Назад", 
            callback_data=CategoryCallback(action="back", category_id=parent_id).pack()
        )])
        
    return InlineKeyboardMarkup(inline_keyboard=buttons)

def products_menu(products: list, category_id: int):
    buttons = []
    for product in products:
        buttons.append([InlineKeyboardButton(text=f"{product['name']} - {product['price']} ₽", callback_data=f"product_{product['id']}_{category_id}")])
    
    # Кнопка "Назад" теперь возвращает к просмотру родительской категории
    buttons.append([InlineKeyboardButton(
        text="⬅️ Назад к категориям", 
        callback_data=CategoryCallback(action="view", category_id=category_id).pack()
    )])
    return InlineKeyboardMarkup(inline_keyboard=buttons)

def product_card(product: dict):
    buttons = [
        [InlineKeyboardButton(text="✅ Купить", callback_data=f"buy_{product['id']}")],
        # Кнопка "Назад" возвращает к списку товаров в той же категории
        [InlineKeyboardButton(
            text="⬅️ Назад к товарам", 
            callback_data=CategoryCallback(action="view", category_id=product['category_id']).pack()
        )]
    ]
    return InlineKeyboardMarkup(inline_keyboard=buttons)
