from aiogram import Router, F, Bot
from aiogram.types import CallbackQuery, InlineKeyboardButton
from aiogram.utils.keyboard import InlineKeyboardBuilder
from aiogram.utils.markdown import hbold, hitalic
import logging
from datetime import datetime

from api import APIClient
from keyboards.inline import back_to_main_menu_keyboard

router = Router()

@router.callback_query(F.data == "my_orders")
async def my_orders_handler(callback_query: CallbackQuery, api_client: APIClient):
    user_id = callback_query.from_user.id
    try:
        result = await api_client.get_user_orders(user_id)

        if result.get("success"):
            orders = result.get("data")
            if not orders:
                await callback_query.message.edit_text("У вас пока нет заказов.", reply_markup=back_to_main_menu_keyboard())
                return

            builder = InlineKeyboardBuilder()
            for order in orders:
                product_name = order.get('product_name', 'Неизвестный продукт')
                created_at_str = order.get('created_at', '')
                try:
                    created_dt = datetime.fromisoformat(created_at_str.replace('Z', '+00:00'))
                    created_formatted = created_dt.strftime('%d.%m.%y')
                except ValueError:
                    created_formatted = ""
                
                button_text = f"{product_name} - {created_formatted}"
                builder.row(InlineKeyboardButton(text=button_text, callback_data=f"order_details:{order.get('id')}"))

            builder.row(InlineKeyboardButton(text="⬅️ Назад в меню", callback_data="main_menu"))

            await callback_query.message.edit_text(
                f"{hbold('🧾 Ваши заказы:')}\n\nНажмите на заказ, чтобы посмотреть детали.",
                reply_markup=builder.as_markup()
            )

        else:
            error = result.get("error", "Произошла неизвестная ошибка.")
            await callback_query.message.edit_text(f"Произошла ошибка: {error}")

    except Exception as e:
        logging.exception("An unexpected error occurred in my_orders_handler")
        await callback_query.message.edit_text("Произошла непредвиденная ошибка. Попробуйте позже.")
    finally:
        await callback_query.answer()

@router.callback_query(F.data.startswith("order_details:"))
async def order_details_handler(callback_query: CallbackQuery, api_client: APIClient, bot: Bot):
    order_id = int(callback_query.data.split(":")[1])
    try:
        result = await api_client.get_order(order_id)

        if result.get("success"):
            order = result.get("data")
            product_name = order.get('product_name', 'Неизвестный продукт')
            amount = order.get('amount', 0)
            created_at_str = order.get('created_at', '')
            fulfilled_content = order.get('fulfilled_content')
            image_url = order.get('image_url')

            try:
                created_dt = datetime.fromisoformat(created_at_str.replace('Z', '+00:00'))
                created_formatted = created_dt.strftime('%d.%m.%Y %H:%M')
            except ValueError:
                created_formatted = "неизвестно"

            caption = (
                f"Детали заказа:\n\n"
                f"🔹 {hbold(product_name)} - {amount} ₽\n"
                f"   {hitalic(created_formatted)}\n"
            )

            if fulfilled_content:
                caption += f"\n{hbold('Ваш товар:')}\n<pre>{fulfilled_content}</pre>\n"

            await callback_query.message.delete()

            if image_url:
                full_image_url = f"{api_client.base_url}{image_url}"
                await bot.send_photo(
                    chat_id=callback_query.from_user.id,
                    photo=full_image_url,
                    caption=caption,
                    parse_mode="HTML",
                    reply_markup=back_to_main_menu_keyboard()
                )
            else:
                await bot.send_message(
                    chat_id=callback_query.from_user.id,
                    text=caption,
                    parse_mode="HTML",
                    reply_markup=back_to_main_menu_keyboard()
                )

        else:
            error = result.get("error", "Произошла неизвестная ошибка.")
            await callback_query.answer(f"Ошибка: {error}", show_alert=True)

    except Exception as e:
        logging.exception("An unexpected error occurred in order_details_handler")
        await callback_query.answer("Произошла непредвиденная ошибка.", show_alert=True)
