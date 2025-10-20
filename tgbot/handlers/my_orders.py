from aiogram import Router, F
from aiogram.types import CallbackQuery
from aiogram.utils.markdown import hbold, hitalic, hcode
import logging
import json
from datetime import datetime

from api import APIClient
from keyboards.inline import main_menu, back_to_main_menu_keyboard

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

            response_text = f"{hbold('🧾 Ваши заказы:')}\n\n"
            for order in orders:
                product_name = order.get('product_name', 'Неизвестный продукт')
                created_at_str = order.get('created_at', '')
                
                try:
                    created_dt = datetime.fromisoformat(created_at_str.replace('Z', '+00:00'))
                    created_formatted = created_dt.strftime('%d.%m.%Y %H:%M')
                except ValueError:
                    created_formatted = "неизвестно"

                response_text += f"🔹 {hbold(product_name)} - {order.get('amount')} ₽\n"
                response_text += f"   {hitalic(created_formatted)}\n"

                fulfilled_content = order.get('fulfilled_content')
                if fulfilled_content:
                    response_text += f"   {hbold('Ваш товар:')}\n<pre>{fulfilled_content}</pre>\n"
                
                response_text += "\n"

            await callback_query.message.edit_text(response_text, parse_mode="HTML", reply_markup=back_to_main_menu_keyboard())

        else:
            error = result.get("error", "Произошла неизвестная ошибка.")
            await callback_query.message.edit_text(f"Произошла ошибка: {error}")

    except Exception as e:
        logging.exception("An unexpected error occurred in my_orders_handler")
        await callback_query.message.edit_text("Произошла непредвиденная ошибка. Попробуйте позже.")
    finally:
        await callback_query.answer()
