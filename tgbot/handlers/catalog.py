from aiogram import Router, F
import logging
from aiogram.types import CallbackQuery
from aiogram.utils.markdown import hbold, hitalic

from api import api_client
from keyboards.inline import CategoryCallback, categories_menu, products_menu, product_card

router = Router()

# --- Helper functions ---

def find_category_by_id(categories: list, category_id: int):
    """Recursively find a category in the tree."""
    for category in categories:
        if category['id'] == category_id:
            return category
        if category.get('sub_categories'):
            found = find_category_by_id(category['sub_categories'], category_id)
            if found:
                return found
    return None

def find_parent_id(categories: list, child_id: int):
    """Recursively find the parent_id of a category."""
    for category in categories:
        if category['id'] == child_id:
            return category.get('parent_id')
        if category.get('sub_categories'):
            parent_id = find_parent_id(category['sub_categories'], child_id)
            if parent_id is not None:
                return parent_id
    return None

# --- Handlers ---

@router.callback_query(CategoryCallback.filter(F.action == 'view'))
async def navigate_categories(callback_query: CallbackQuery, callback_data: CategoryCallback):
    category_id = callback_data.category_id

    try:
        response = await api_client.get_categories()
        if not response.get("success"):
            await callback_query.message.edit_text("Не удалось загрузить категории.")
            await callback_query.answer()
            return

        all_categories = response["data"]

        if category_id == 0: # Root level
            current_level_categories = all_categories
            parent_id = 0
        else:
            selected_category = find_category_by_id(all_categories, category_id)
            if not selected_category:
                await callback_query.message.edit_text("Категория не найдена.")
                await callback_query.answer()
                return
            
            current_level_categories = selected_category.get('sub_categories', [])
            parent_id = selected_category.get('parent_id') or 0

        # Если есть подкатегории, показываем их. Иначе - показываем товары.
        if current_level_categories:
            await callback_query.message.edit_text(
                "🛍️ Выберите категорию:",
                reply_markup=categories_menu(current_level_categories, category_id)
            )
        else:
            # Это конечная категория, показываем товары
            products_response = await api_client.get_products(category_id)
            if products_response.get("success"):
                products = products_response["data"]
                await callback_query.message.edit_text(
                    "Выберите товар:",
                    reply_markup=products_menu(products, category_id)
                )
            else:
                await callback_query.message.edit_text(
                    f"Не удалось загрузить товары: {products_response.get('error')}",
                    reply_markup=categories_menu([], parent_id=category_id) # Позволяет вернуться назад
                )

    except Exception:
        logging.exception("An error occurred in navigate_categories")
        await callback_query.message.edit_text("Произошла непредвиденная ошибка. Попробуйте позже.")
    
    await callback_query.answer()

@router.callback_query(CategoryCallback.filter(F.action == 'back'))
async def go_back_category(callback_query: CallbackQuery, callback_data: CategoryCallback):
    # ID категории, к которой мы возвращаемся (это родитель)
    target_category_id = callback_data.category_id

    try:
        response = await api_client.get_categories()
        if not response.get("success"):
            await callback_query.message.edit_text("Не удалось загрузить категории.")
            await callback_query.answer()
            return

        all_categories = response["data"]

        if target_category_id == 0:
            # Возвращаемся в корень
            await callback_query.message.edit_text(
                "🛍️ Выберите категорию:",
                reply_markup=categories_menu(all_categories, 0)
            )
        else:
            # Находим "дедушку", чтобы знать, куда вернется кнопка "Назад" со следующего уровня
            grandparent_id = find_parent_id(all_categories, target_category_id) or 0
            parent_category = find_category_by_id(all_categories, target_category_id)
            
            # Нам нужно показать категории того же уровня, что и target_category_id
            # Для этого найдем их общего родителя
            if grandparent_id == 0:
                categories_to_show = all_categories
            else:
                grandparent = find_category_by_id(all_categories, grandparent_id)
                categories_to_show = grandparent.get('sub_categories', [])

            await callback_query.message.edit_text(
                "🛍️ Выберите категорию:",
                reply_markup=categories_menu(categories_to_show, grandparent_id)
            )

    except Exception:
        logging.exception("An error occurred in go_back_category")
        await callback_query.message.edit_text("Произошла непредвиденная ошибка. Попробуйте позже.")

    await callback_query.answer()


@router.callback_query(F.data.startswith('product_'))
async def product_handler(callback_query: CallbackQuery):
    try:
        _, product_id_str, category_id_str = callback_query.data.split('_')
        product_id = int(product_id_str)
        category_id = int(category_id_str)

        response = await api_client.get_products(category_id)
        if response.get("success"):
            products = response["data"]
            product = next((p for p in products if p['id'] == product_id), None)
            if product:
                await callback_query.message.edit_text(
                    f"{hbold(product['name'])}\n\n"
                    f"{hitalic('Цена:')} {product['price']} ₽",
                    reply_markup=product_card(product),
                    parse_mode="HTML"
                )
            else:
                await callback_query.message.edit_text("Товар не найден.")
        else:
            await callback_query.message.edit_text(f"Не удалось загрузить товар: {response.get('error')}")

    except Exception:
        logging.exception("An error occurred in product_handler")
        await callback_query.message.edit_text("Произошла непредвиденная ошибка. Попробуйте позже.")
    await callback_query.answer()