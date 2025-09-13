from aiogram import Router, F
from aiogram.types import CallbackQuery
from aiogram.utils.markdown import hbold, hitalic

from api import api_client
from keyboards import inline

router = Router()

@router.callback_query(F.data == 'catalog')
async def catalog_handler(callback_query: CallbackQuery):
    try:
        response = await api_client.get_categories()
        if response.get("success"):
            categories = response["data"]
            await callback_query.message.edit_text(
                "🛍️ Выберите категорию товаров:", 
                reply_markup=inline.categories_menu(categories)
            )
        else:
            seller_info_response = await api_client.get_seller_info()
            referral_program_enabled = seller_info_response.get("data", {}).get("referral_program_enabled", False)
            await callback_query.message.edit_text(
                f"Не удалось загрузить категории: {response.get('error')}",
                reply_markup=inline.main_menu(referral_program_enabled=referral_program_enabled)
            )
    except Exception as e:
        await callback_query.message.edit_text(f"Произошла ошибка: {e}")
    await callback_query.answer()

@router.callback_query(F.data.startswith('category_'))
async def products_handler(callback_query: CallbackQuery):
    category_id = int(callback_query.data.split('_')[1])
    try:
        response = await api_client.get_products(category_id)
        if response.get("success"):
            products = response["data"]
            await callback_query.message.edit_text(
                "Выберите товар:", 
                reply_markup=inline.products_menu(products, category_id)
            )
        else:
            await callback_query.message.edit_text(
                f"Не удалось загрузить товары: {response.get('error')}",
                reply_markup=inline.categories_menu([])
            )
    except Exception as e:
        await callback_query.message.edit_text(f"Произошла ошибка: {e}")
    await callback_query.answer()

@router.callback_query(F.data.startswith('product_'))
async def product_handler(callback_query: CallbackQuery):
    _, product_id, category_id = callback_query.data.split('_')
    product_id = int(product_id)
    category_id = int(category_id)

    try:
        # In a real bot, we should have a get_product_by_id endpoint.
        # For now, we will filter from the list of products.
        response = await api_client.get_products(category_id)
        if response.get("success"):
            products = response["data"]
            product = next((p for p in products if p['id'] == product_id), None)
            if product:
                await callback_query.message.edit_text(
                    f"{hbold(product['name'])}\n\n"
                    f"{hitalic('Цена:')} {product['price']} ₽",
                    reply_markup=inline.product_card(product),
                    parse_mode="HTML"
                )
            else:
                await callback_query.message.edit_text("Товар не найден.")
        else:
            await callback_query.message.edit_text(
                f"Не удалось загрузить товар: {response.get('error')}"
            )

    except Exception as e:
        await callback_query.message.edit_text(f"Произошла ошибка: {e}")
    await callback_query.answer()