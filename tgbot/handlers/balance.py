from aiogram import Router
from aiogram.types import CallbackQuery

from api import api_client
from keyboards import inline

router = Router()

@router.callback_query(lambda c: c.data == 'balance')
async def balance_handler(callback_query: CallbackQuery):
    try:
        user_id = callback_query.from_user.id
        balance_data = await api_client.get_user_balance(user_id)
        balance = balance_data['data']['balance']
        await callback_query.message.answer(f"Ваш баланс: {balance} ₽")
    except Exception as e:
        await callback_query.message.answer(f"Произошла ошибка: {e}")
    await callback_query.answer()

@router.callback_query(lambda c: c.data == 'deposit')
async def deposit_handler(callback_query: CallbackQuery):
    await callback_query.message.edit_text("Выберите сумму для пополнения:", reply_markup=inline.deposit_menu())
    await callback_query.answer()

@router.callback_query(lambda c: c.data.startswith('deposit_'))
async def deposit_amount_handler(callback_query: CallbackQuery):
    amount = int(callback_query.data.split('_')[1])
    user_id = callback_query.from_user.id
    try:
        deposit_data = await api_client.create_deposit(user_id, amount)
        # Here you would typically get a payment URL and show it to the user.
        # For MVP, we'll just confirm the creation.
        await callback_query.message.edit_text(f"Заявка на пополнение на {amount} ₽ создана.")
    except Exception as e:
        await callback_query.message.edit_text(f"Произошла ошибка: {e}")
    await callback_query.answer()
