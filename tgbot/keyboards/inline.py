from aiogram.types import InlineKeyboardMarkup, InlineKeyboardButton
from aiogram.filters.callback_data import CallbackData
from typing import Optional

# Фабрика колбэков для навигации по категориям
class CategoryCallback(CallbackData, prefix="cat"):
    action: str  # "view", "back"
    category_id: int = 0
    parent_id: int = 0 # ID родителя, чтобы знать, куда возвращаться

# Фабрика колбэков для процесса оплаты
class PaymentCallback(CallbackData, prefix="pay"):
    action: str       # e.g., 'select_gateway', 'select_amount'
    gateway: Optional[str] = None
    amount: Optional[float] = None

def main_menu(referral_program_enabled: bool = False, bot_type: str = "main"):
    buttons = [
        [InlineKeyboardButton(text="🛍️ Каталог", callback_data=CategoryCallback(action="view", category_id=0).pack())],
        [InlineKeyboardButton(text="💳 Баланс", callback_data="balance")],
        [InlineKeyboardButton(text="🧾 Мои заказы", callback_data="my_orders")],
        [InlineKeyboardButton(text="🧾 Мои подписки", callback_data="my_subscriptions")],
        [InlineKeyboardButton(text="💰 Пополнить баланс", callback_data="deposit")],
    ]
    if referral_program_enabled and bot_type == "main":
        buttons.append([InlineKeyboardButton(text="🤝 Реферальный магазин", callback_data="referral_program")])
    
    buttons.append([InlineKeyboardButton(text="💬 Поддержка", callback_data="support")])

    return InlineKeyboardMarkup(inline_keyboard=buttons)

def payment_gateways_menu(gateways: list, instructions_url: str):
    buttons = []
    if instructions_url:
        buttons.append([InlineKeyboardButton(text="Как пополнить баланс?", url=instructions_url)])
    
    for gw in gateways:
        buttons.append([InlineKeyboardButton(
            text=gw['display_name'], 
            callback_data=PaymentCallback(action="select_gateway", gateway=gw['name']).pack()
        )])

    buttons.append([InlineKeyboardButton(text="⬅️ Назад", callback_data="main_menu")])
    return InlineKeyboardMarkup(inline_keyboard=buttons)

def deposit_amount_menu(gateway: str):
    buttons = [
        [InlineKeyboardButton(text="100 ₽", callback_data=PaymentCallback(action="select_amount", gateway=gateway, amount=100).pack())],
        [InlineKeyboardButton(text="500 ₽", callback_data=PaymentCallback(action="select_amount", gateway=gateway, amount=500).pack())],
        [InlineKeyboardButton(text="1000 ₽", callback_data=PaymentCallback(action="select_amount", gateway=gateway, amount=1000).pack())],
        [InlineKeyboardButton(text="⬅️ Назад", callback_data="deposit")]
    ]
    return InlineKeyboardMarkup(inline_keyboard=buttons)

def categories_menu(categories: list, parent_id: int = 0, products: list = []):
    buttons = []
    for product in products:
        buttons.append([InlineKeyboardButton(
            text=f"🔹 {product['name']} - {product['price']} ₽", 
            callback_data=f"extproduct_{product['provider']}_{product['external_id']}"
        )])

    for category in categories:
        buttons.append([InlineKeyboardButton(
            text=category['name'], 
            callback_data=CategoryCallback(action="view", category_id=category['id'], parent_id=parent_id).pack()
        )])
    
    if parent_id == 0:
        buttons.append([InlineKeyboardButton(text="⬅️ Назад", callback_data="main_menu")])
    else:
        buttons.append([InlineKeyboardButton(
            text="⬅️ Назад", 
            callback_data=CategoryCallback(action="back", category_id=parent_id).pack()
        )])
        
    return InlineKeyboardMarkup(inline_keyboard=buttons)

def products_menu(products: list, category_id: int, parent_id: int):
    buttons = []
    for product in products:
        if product.get('provider'):
            buttons.append([InlineKeyboardButton(text=f"{product['name']} - {product['price']} ₽", callback_data=f"extproduct_{product['provider']}_{product['external_id']}")])
        else:
            buttons.append([InlineKeyboardButton(text=f"{product['name']} - {product['price']} ₽", callback_data=f"product_{product['id']}_{category_id}")])
    
    buttons.append([InlineKeyboardButton(
        text="⬅️ Назад к категориям", 
        callback_data=CategoryCallback(action="view", category_id=parent_id).pack()
    )])
    return InlineKeyboardMarkup(inline_keyboard=buttons)

def product_card(product: dict):
    buttons = []
    if product.get('provider'):
        buttons.append([InlineKeyboardButton(text="✅ Купить", callback_data=f"buy_ext_{product['provider']}_{product['external_id']}")])
        # For external products, "back" returns to the root catalog
        buttons.append([InlineKeyboardButton(
            text="⬅️ Назад к каталогу", 
            callback_data=CategoryCallback(action="view", category_id=0).pack()
        )])
    else:
        buttons.append([InlineKeyboardButton(text="✅ Купить", callback_data=f"buy_{product['id']}")])
        buttons.append([InlineKeyboardButton(
            text="⬅️ Назад к товарам", 
            callback_data=CategoryCallback(action="view", category_id=product['category_id']).pack()
        )])
    return InlineKeyboardMarkup(inline_keyboard=buttons)