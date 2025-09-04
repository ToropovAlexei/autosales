from aiogram import Router
from aiogram.types import CallbackQuery
from aiohttp import ClientResponseError

from api import api_client

router = Router()


@router.callback_query(lambda c: c.data.startswith("buy_"))
async def buy_handler(callback_query: CallbackQuery):
    product_id = int(callback_query.data.split("_")[1])
    user_id = callback_query.from_user.id
    try:
        result = await api_client.buy_product(user_id, product_id)
        new_balance = result["balance"]
        await callback_query.message.edit_text(
            f"Вы успешно купили товар! Ваш новый баланс: {new_balance} ₽"
        )
    except ClientResponseError as e:
        error_message = "Произошла неизвестная ошибка."
        if e.status == 400:
            try:
                data = await e.json()
                detail = data.get("detail", "")
                if detail == "Insufficient balance":
                    error_message = "Недостаточно средств. Пополните баланс."
                elif detail == "Product out of stock":
                    error_message = "Товар закончился."
            except:
                pass
        elif e.status == 404:
            error_message = "Товар не найден."

        await callback_query.message.edit_text(error_message)
    except Exception as e:
        await callback_query.message.edit_text(f"Произошла ошибка: {e}")
    await callback_query.answer()

