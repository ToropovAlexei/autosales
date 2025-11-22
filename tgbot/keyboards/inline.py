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
    force: bool = False

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

def balance_menu():
    buttons = [
        [InlineKeyboardButton(text="💰 Пополнить баланс", callback_data="deposit")],
        [InlineKeyboardButton(text="⬅️ Назад", callback_data="main_menu")]
    ]
    return InlineKeyboardMarkup(inline_keyboard=buttons)


def payment_gateways_menu(gateways: list, public_settings: dict, instructions_url: str):
    buttons = []
    if instructions_url:
        buttons.append([InlineKeyboardButton(text="ℹ️ Как пополнить баланс?", url=instructions_url)])

    gateways_with_bonuses = []
    for gw in gateways:
        bonus_key = f"GATEWAY_BONUS_{gw['name']}"
        bonus_value = float(public_settings.get(bonus_key, "0"))
        gateways_with_bonuses.append({
            "name": gw['name'],
            "display_name": gw['display_name'],
            "bonus": bonus_value
        })

    gateways_with_bonuses.sort(key=lambda x: (-x['bonus'], x['display_name']))

    for i, gw in enumerate(gateways_with_bonuses):
        display_name = gw['display_name']
        if gw['bonus'] > 0:
            bonus_text = ""
            if gw['bonus'].is_integer():
                bonus_text = str(int(gw['bonus']))
            else:
                bonus_text = str(gw['bonus'])
            display_name += f" (скидка {bonus_text}%)"
        
        if i == 0 and gw['bonus'] > 0:
            display_name = f"🔥🔥 {display_name} 🔥🔥"

        buttons.append([InlineKeyboardButton(
            text=display_name, 
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

def categories_menu(categories: list, parent_id: int = 0, products: list = [], category_id: int = 0):
    buttons = []
    
    for category in categories:
        buttons.append([InlineKeyboardButton(
            text=category['name'], 
            callback_data=CategoryCallback(action="view", category_id=category['id'], parent_id=parent_id).pack()
        )])
    
    for product in products:
        buttons.append([InlineKeyboardButton(
            text=f"🔹 {product['name']} - {product['price']} ₽", 
            callback_data=f"product_{product['id']}_{category_id}"
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
        buttons.append([InlineKeyboardButton(text=f"{product['name']} - {product['price']} ₽", callback_data=f"product_{product['id']}_{category_id}")])
    
    buttons.append([InlineKeyboardButton(
        text="⬅️ Назад к категориям", 
        callback_data=CategoryCallback(action="view", category_id=parent_id).pack()
    )])
    return InlineKeyboardMarkup(inline_keyboard=buttons)

def product_card(product: dict):
    buttons = []
    buttons.append([InlineKeyboardButton(text="✅ Купить", callback_data=f"buy_{product['id']}")])
    buttons.append([InlineKeyboardButton(
        text="⬅️ Назад к товарам", 
        callback_data=CategoryCallback(action="view", category_id=product['category_id']).pack()
    )])
    return InlineKeyboardMarkup(inline_keyboard=buttons)

def back_to_main_menu_keyboard():
    buttons = [
        [InlineKeyboardButton(text="⬅️ Главное меню", callback_data="main_menu")]
    ]
    return InlineKeyboardMarkup(inline_keyboard=buttons)

def insufficient_balance_keyboard():
    buttons = [
        [InlineKeyboardButton(text="💰 Пополнить баланс", callback_data="deposit")],
        [InlineKeyboardButton(text="⬅️ Назад", callback_data="main_menu")]
    ]
    return InlineKeyboardMarkup(inline_keyboard=buttons)

def suggest_best_gateway_keyboard(selected_gateway: dict, best_gateway: dict):
    buttons = [
        [
            InlineKeyboardButton(
                text=f"Продолжить с {selected_gateway['display_name']}",
                callback_data=PaymentCallback(action="select_gateway", gateway=selected_gateway['name'], force=True).pack()
            )
        ],
        [
            InlineKeyboardButton(
                text=f"Выбрать {best_gateway['display_name']}",
                callback_data=PaymentCallback(action="select_gateway", gateway=best_gateway['name'], force=True).pack()
            )
        ],
    ]
    return InlineKeyboardMarkup(inline_keyboard=buttons)