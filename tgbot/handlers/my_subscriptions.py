from aiogram import Router, F
from aiogram.types import CallbackQuery
from aiogram.utils.markdown import hbold, hitalic, hcode
import logging
import json
from datetime import datetime

from api import api_client
from keyboards.inline import main_menu # Assuming main_menu is needed for a back button

router = Router()

@router.callback_query(F.data == "my_subscriptions")
async def my_subscriptions_handler(callback_query: CallbackQuery):
    user_id = callback_query.from_user.id
    try:
        result = await api_client.get_user_subscriptions(user_id)

        if result.get("success"):
            subscriptions = result.get("data")
            if not subscriptions:
                await callback_query.message.edit_text("У вас пока нет активных подписок.")
                return

            response_text = f"{hbold('🧾 Ваши подписки:')}\n\n"
            for sub in subscriptions:
                product_name = sub.get('Product', {}).get('name', 'Неизвестный продукт')
                expires_at_str = sub.get('expires_at', '')
                
                try:
                    expires_dt = datetime.fromisoformat(expires_at_str.replace('Z', '+00:00'))
                    expires_formatted = expires_dt.strftime('%d.%m.%Y %H:%M')
                    status = "✅ Активна до" if expires_dt > datetime.now(expires_dt.tzinfo) else "❌ Неактивна"
                except ValueError:
                    expires_formatted = "неизвестно"
                    status = ""

                response_text += f"🔹 {hbold(product_name)}\n"
                response_text += f"   {status} {hitalic(expires_formatted)}\n"

                details_json = sub.get('details')
                if details_json:
                    try:
                        # The details might be a string that needs to be loaded, or already a dict
                        details = json.loads(details_json) if isinstance(details_json, str) else details_json
                        if details:
                            response_text += f"   {hbold('Данные для доступа:')}\n"
                            if 'username' in details:
                                response_text += f"     - Логин: {hcode(str(details['username']))}\n"
                            if 'password' in details:
                                response_text += f"     - Пароль: {hcode(str(details['password']))}\n"
                    except (json.JSONDecodeError, TypeError):
                        logging.warning(f"Could not parse subscription details: {details_json}")
                
                response_text += "\n"

            # TODO: Add a back button to the main menu
            await callback_query.message.edit_text(response_text, parse_mode="HTML")

        else:
            error = result.get("error", "Произошла неизвестная ошибка.")
            await callback_query.message.edit_text(f"Произошла ошибка: {error}")

    except Exception as e:
        logging.exception("An unexpected error occurred in my_subscriptions_handler")
        await callback_query.message.edit_text("Произошла непредвиденная ошибка. Попробуйте позже.")
    finally:
        await callback_query.answer()
