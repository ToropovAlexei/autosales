from aiogram import Router
from aiogram.types import CallbackQuery

from api import api_client
from keyboards import inline

router = Router()

@router.callback_query(lambda c: c.data == 'catalog')
async def catalog_handler(callback_query: CallbackQuery):
    try:
        categories_data = await api_client.get_categories()
        categories = categories_data
        await callback_query.message.edit_text("Выберите категорию:", reply_markup=inline.categories_menu(categories))
    except Exception as e:
        await callback_query.message.edit_text(f"Произошла ошибка: {e}")
    await callback_query.answer()

@router.callback_query(lambda c: c.data.startswith('category_'))
async def products_handler(callback_query: CallbackQuery):
    category_id = int(callback_query.data.split('_')[1])
    try:
        products_data = await api_client.get_products(category_id)
        products = products_data
        await callback_query.message.edit_text("Выберите товар:", reply_markup=inline.products_menu(products, category_id))
    except Exception as e:
        await callback_query.message.edit_text(f"Произошла ошибка: {e}")
    await callback_query.answer()

@router.callback_query(lambda c: c.data.startswith('product_'))
async def product_handler(callback_query: CallbackQuery):
    _, product_id, category_id = callback_query.data.split('_')
    product_id = int(product_id)
    category_id = int(category_id)

    # This is a simplification. In a real bot, you would fetch product details first.
    # Here we assume the product details are already known or not needed for the card.
    # We will just show the buy button.
    # A better approach would be to get the product details from the API.
    # For now, we will just create a dummy product dict to pass to the keyboard.
    dummy_product = {'id': product_id, 'category_id': category_id}
    await callback_query.message.edit_text("Карточка товара", reply_markup=inline.product_card(dummy_product))
    await callback_query.answer()
