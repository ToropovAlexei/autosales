from aiogram import Router, F
import logging
import aiohttp
from aiogram.types import CallbackQuery, BufferedInputFile
from aiogram.utils.markdown import hbold, hitalic

from api import APIClient
from keyboards.inline import CategoryCallback, categories_menu, products_menu, product_card
from config import settings

from aiogram.exceptions import TelegramBadRequest

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
async def navigate_categories(callback_query: CallbackQuery, callback_data: CategoryCallback, api_client: APIClient):
    category_id = callback_data.category_id

    try:
        categories_response = await api_client.get_categories()
        if not categories_response.get("success"):
            await callback_query.message.edit_text("Не удалось загрузить категории.")
            await callback_query.answer()
            return
        all_categories = categories_response["data"]

        if category_id == 0:  # Root level
            await callback_query.message.edit_text(
                "🛍️ Выберите категорию:",
                reply_markup=categories_menu(all_categories, 0, category_id=0)
            )
        else:  # Category level
            selected_category = find_category_by_id(all_categories, category_id)
            if not selected_category:
                await callback_query.message.edit_text("Категория не найдена.")
                await callback_query.answer()
                return

            sub_categories = selected_category.get('sub_categories', [])
            parent_id_for_back = selected_category.get('parent_id')
            image_id = selected_category.get('image_id')

            # Always fetch products for the current category
            products = []
            products_response = await api_client.get_products_for_bot(category_id)
            if products_response.get("success"):
                products = products_response.get("data") or []
            else:
                logging.warning(f"Could not fetch products for category {category_id}")

            # Determine caption and reply markup
            caption = "🛍️ Выберите товар или категорию:"
            reply_markup = categories_menu(sub_categories, parent_id_for_back or 0, products, category_id)

            # Now, decide how to send the message
            if image_id:
                image_url = f"{settings.api_url.rstrip('/')}/images/{image_id}"
                try:
                    async with aiohttp.ClientSession() as session:
                        async with session.get(image_url) as resp:
                            if resp.status == 200:
                                image_bytes = await resp.read()
                                # Send the new photo message FIRST
                                await callback_query.message.answer_photo(
                                    photo=BufferedInputFile(image_bytes, filename="image.jpg"),
                                    caption=caption,
                                    reply_markup=reply_markup
                                )
                                # If successful, THEN delete the old one
                                await callback_query.message.delete()
                            else:
                                logging.warning(f"Failed to download image {image_url}, status: {resp.status}. Falling back to text.")
                                await callback_query.message.edit_text(caption, reply_markup=reply_markup)
                except Exception:
                    logging.exception(f"Failed to process or send image {image_url}")
                    # If anything fails, try to edit the original message as a fallback
                    try:
                        await callback_query.message.edit_text(caption, reply_markup=reply_markup)
                    except Exception as inner_e:
                        logging.exception(f"Failed to fall back to edit_text: {inner_e}")
            else:
                # The target is a text menu. The source could be a photo menu.
                try:
                    await callback_query.message.edit_text(
                        caption,
                        reply_markup=reply_markup
                    )
                except TelegramBadRequest:
                    # Failed, so we're coming from a photo. Delete and send new text message.
                    await callback_query.message.delete()
                    await callback_query.message.answer(
                        caption,
                        reply_markup=reply_markup
                    )

    except Exception:
        logging.exception("An error occurred in navigate_categories")
        await callback_query.message.edit_text("Произошла непредвиденная ошибка. Попробуйте позже.")

    await callback_query.answer()




@router.callback_query(CategoryCallback.filter(F.action == 'back'))
async def go_back_category(callback_query: CallbackQuery, callback_data: CategoryCallback, api_client: APIClient):
    target_category_id = callback_data.category_id

    try:
        # Fetch all categories for navigation tree structure
        categories_response = await api_client.get_categories()
        if not categories_response.get("success"):
            await callback_query.message.edit_text("Не удалось загрузить категории.")
            await callback_query.answer()
            return
        all_categories = categories_response["data"]

        # Fetch products for the category we are going back to
        products = []
        products_response = await api_client.get_products_for_bot(target_category_id)
        if products_response.get("success"):
            products = products_response.get("data") or []

        # Determine which categories to show in the menu
        categories_to_show = []
        parent_id_for_back = 0
        if target_category_id == 0:
            categories_to_show = all_categories
        else:
            parent_id_for_back = find_parent_id(all_categories, target_category_id) or 0
            parent_category = find_category_by_id(all_categories, parent_id_for_back)
            if parent_category:
                categories_to_show = parent_category.get('sub_categories', [])
            elif parent_id_for_back == 0:
                 categories_to_show = all_categories


        caption = "🛍️ Выберите категорию или товар:"
        reply_markup = categories_menu(categories_to_show, parent_id_for_back, products, category_id=target_category_id)

        try:
            await callback_query.message.edit_text(
                caption,
                reply_markup=reply_markup
            )
        except TelegramBadRequest:
            await callback_query.message.delete()
            await callback_query.message.answer(
                caption,
                reply_markup=reply_markup
            )

    except Exception:
        logging.exception("An error occurred in go_back_category")
        await callback_query.message.edit_text("Произошла непредвиденная ошибка. Попробуйте позже.")

    await callback_query.answer()


@router.callback_query(F.data.startswith('product_'))
async def product_handler(callback_query: CallbackQuery, api_client: APIClient):
    try:
        _, product_id_str, category_id_str = callback_query.data.split('_')
        product_id = int(product_id_str)

        response = await api_client.get_product_for_bot(product_id)
        if response.get("success"):
            product = response["data"]
            if product:
                caption = (
                    f"{hbold(product['name'])}\n\n"
                    f"{hitalic('Цена:')} {int(product['price'])} ₽"
                )
                reply_markup = product_card(product)

                if product.get('image_id'):
                    image_url = f"{settings.api_url.rstrip('/')}/images/{product['image_id']}"
                    try:
                        async with aiohttp.ClientSession() as session:
                            async with session.get(image_url) as resp:
                                if resp.status == 200:
                                    image_bytes = await resp.read()
                                    await callback_query.message.answer_photo(
                                        photo=BufferedInputFile(image_bytes, filename="image.jpg"),
                                        caption=caption,
                                        reply_markup=reply_markup,
                                        parse_mode="HTML"
                                    )
                                    await callback_query.message.delete()
                                else:
                                    await callback_query.message.edit_text(
                                        caption,
                                        reply_markup=reply_markup,
                                        parse_mode="HTML"
                                    )
                    except Exception as e:
                        logging.exception(f"Error sending photo for product {product_id}: {e}")
                        await callback_query.message.edit_text(
                            caption,
                            reply_markup=reply_markup,
                            parse_mode="HTML"
                        )
                else:
                    await callback_query.message.edit_text(
                        caption,
                        reply_markup=reply_markup,
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

@router.callback_query(F.data.startswith('extproduct_'))
async def external_product_handler(callback_query: CallbackQuery, api_client: APIClient):
    try:
        parts = callback_query.data.split('_')
        if len(parts) < 3:
            raise ValueError("Invalid extproduct callback format")

        provider = '_'.join(parts[1:-1])
        external_id = parts[-1]
        
        # This is not efficient, but we don't have a direct way to get an external product by its ID
        response = await api_client.get_products_for_bot()
        if response.get("success"):
            products = response["data"]
            product = next((p for p in products if p.get('provider') == provider and p.get('external_id') == external_id), None)
            if product:
                # Assuming external products are always subscriptions
                description = f"Подписка на {product.get('subscription_period_days', 30)} дней"
                caption = (
                    f"{hbold(product['name'])}\n\n"
                    f"{hitalic(description)}\n\n"
                    f"{hitalic('Цена:')} {product['price']:.2f} ₽"
                )
                reply_markup = product_card(product)

                if product.get('image_id'):
                    image_url = f"{settings.api_url.rstrip('/')}/images/{product['image_id']}"
                    try:
                        async with aiohttp.ClientSession() as session:
                            async with session.get(image_url) as resp:
                                if resp.status == 200:
                                    image_bytes = await resp.read()
                                    await callback_query.message.answer_photo(
                                        photo=BufferedInputFile(image_bytes, filename="image.jpg"),
                                        caption=caption,
                                        reply_markup=reply_markup,
                                        parse_mode="HTML"
                                    )
                                    await callback_query.message.delete()
                                else:
                                    await callback_query.message.edit_text(
                                        caption,
                                        reply_markup=reply_markup,
                                        parse_mode="HTML"
                                    )
                    except Exception as e:
                        logging.exception(f"Error sending photo for external product {external_id}: {e}")
                        await callback_query.message.edit_text(
                            caption,
                            reply_markup=reply_markup,
                            parse_mode="HTML"
                        )
                else:
                    await callback_query.message.edit_text(
                        caption,
                        reply_markup=reply_markup,
                        parse_mode="HTML"
                    )
            else:
                await callback_query.message.edit_text("Товар не найден.")
        else:
            await callback_query.message.edit_text(f"Не удалось загрузить товар: {response.get('error')}")

    except Exception:
        logging.exception("An error occurred in external_product_handler")
        await callback_query.message.edit_text("Произошла непредвиденная ошибка. Попробуйте позже.")
    await callback_query.answer()


