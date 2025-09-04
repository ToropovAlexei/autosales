from aiogram import Router
from aiogram.types import CallbackQuery

from api import api_client

router = Router()

@router.callback_query(lambda c: c.data.startswith('buy_'))
async def buy_handler(callback_query: CallbackQuery):
    product_id = int(callback_query.data.split('_')[1])
    user_id = callback_query.from_user.id
    try:
        result = await api_client.buy_product(user_id, product_id)
        if result.get('success'):
            new_balance = result['data']['balance']
            await callback_query.message.edit_text(f"Вы успешно купили товар! Ваш новый баланс: {new_balance} ₽")
        else:
            await callback_query.message.edit_text(f"Недостаточно средств. Пополните баланс.")
    except Exception as e:
        await callback_query.message.edit_text(f"Произошла ошибка: {e}")
    await callback_query.answer()
