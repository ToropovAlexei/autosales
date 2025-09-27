from aiogram import Router, F
from aiogram.types import CallbackQuery
from aiogram.utils.markdown import hbold
import logging

from api import api_client

router = Router()

async def process_buy_result(callback_query: CallbackQuery, result: dict):
    if result.get("success"):
        data = result.get("data")
        if not isinstance(data, dict):
            logging.error(f"API returned success but data is not a dict: {data}")
            await callback_query.message.edit_text("Произошла ошибка при обработке ответа сервера.")
            return

        new_balance = data.get("balance")
        product_name = data.get("product_name")
        product_price = data.get("product_price")

        if new_balance is not None and product_name and product_price is not None:
            await callback_query.message.edit_text(
                f"✅ Поздравляем! Вы успешно купили товар {hbold(product_name)} за {hbold(f'{product_price} ₽')}.\n\n"
                f"💳 Ваш новый баланс: {hbold(f'{new_balance} ₽')}",
                parse_mode="HTML"
            )
        else:
            logging.error(f"Missing keys in successful buy response data: {data}")
            await callback_query.message.edit_text("Произошла ошибка при обработке покупки.")
    else:
        error = result.get("error", "Произошла неизвестная ошибка.")
        if error == "Insufficient Balance":
            error_message = "😔 Недостаточно средств на балансе для совершения покупки. Пожалуйста, пополните баланс."
        elif error == "Product out of stock":
            error_message = "😔 К сожалению, этот товар закончился."
        else:
            error_message = "Произошла непредвиденная ошибка. Попробуйте позже."
        await callback_query.message.edit_text(error_message)

@router.callback_query(F.data.startswith("buy_"))
async def buy_handler(callback_query: CallbackQuery):
    try:
        parts = callback_query.data.split('_')
        telegram_id = callback_query.from_user.id

        if len(parts) >= 2 and parts[1] == 'ext':
            # External product: buy_ext_{provider}_{external_id}
            # Provider name can contain underscores, so we reassemble it.
            if len(parts) < 4:
                raise ValueError("Invalid external buy callback format")
            
            provider = '_'.join(parts[2:-1])
            external_id = parts[-1]
            result = await api_client.buy_external_product(telegram_id, provider, external_id)
        else:
            # Internal product: buy_{product_id}
            _, product_id_str = parts
            product_id = int(product_id_str)
            result = await api_client.buy_product(telegram_id, product_id)
        
        await process_buy_result(callback_query, result)

    except Exception as e:
        logging.exception("An unexpected error occurred in buy_handler")
        await callback_query.message.edit_text("Произошла непредвиденная ошибка. Попробуйте позже.")
    finally:
        await callback_query.answer()