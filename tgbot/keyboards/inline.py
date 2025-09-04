from aiogram.types import InlineKeyboardMarkup, InlineKeyboardButton

def main_menu():
    buttons = [
        [InlineKeyboardButton(text="Каталог", callback_data="catalog")],
        [InlineKeyboardButton(text="Пополнить баланс", callback_data="deposit")],
        [InlineKeyboardButton(text="Баланс", callback_data="balance")]
    ]
    return InlineKeyboardMarkup(inline_keyboard=buttons)

def deposit_menu():
    buttons = [
        [InlineKeyboardButton(text="100 ₽", callback_data="deposit_100")],
        [InlineKeyboardButton(text="500 ₽", callback_data="deposit_500")],
        [InlineKeyboardButton(text="1000 ₽", callback_data="deposit_1000")]
    ]
    return InlineKeyboardMarkup(inline_keyboard=buttons)

def categories_menu(categories: list):
    buttons = []
    for category in categories:
        buttons.append([InlineKeyboardButton(text=category['name'], callback_data=f"category_{category['id']}")])
    return InlineKeyboardMarkup(inline_keyboard=buttons)

def products_menu(products: list, category_id: int):
    buttons = []
    for product in products:
        buttons.append([InlineKeyboardButton(text=f"{product['name']} - {product['price']} ₽", callback_data=f"product_{product['id']}_{category_id}")])
    buttons.append([InlineKeyboardButton(text="Назад к категориям", callback_data="catalog")])
    return InlineKeyboardMarkup(inline_keyboard=buttons)

def product_card(product: dict):
    buttons = [
        [InlineKeyboardButton(text="Купить", callback_data=f"buy_{product['id']}")],
        [InlineKeyboardButton(text="Назад к товарам", callback_data=f"category_{product['category_id']}")]
    ]
    return InlineKeyboardMarkup(inline_keyboard=buttons)
