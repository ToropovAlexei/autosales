from aiogram import Router, F, Bot
from aiogram.types import CallbackQuery
from aiogram.fsm.context import FSMContext

from api import APIClient

router = Router()


@router.callback_query(F.data.startswith("payment_confirm:"))
async def confirm_payment_handler(query: CallbackQuery, state: FSMContext, api_client: APIClient, bot: Bot):
    order_id = query.data.split(":")[1]
    
    response = await api_client.confirm_payment(order_id)
    
    if response and response.get("success"):
        await query.answer("Ваш платеж подтверждается, пожалуйста, подождите.", show_alert=True)
        await bot.edit_message_reply_markup(chat_id=query.message.chat.id, message_id=query.message.message_id, reply_markup=None)
    else:
        error_message = response.get("error", "Произошла ошибка. Попробуйте позже.")
        await query.answer(f"Ошибка: {error_message}", show_alert=True)


@router.callback_query(F.data.startswith("payment_cancel:"))
async def cancel_payment_handler(query: CallbackQuery, state: FSMContext, api_client: APIClient, bot: Bot):
    order_id = query.data.split(":")[1]
    
    response = await api_client.cancel_payment(order_id)
    
    if response and response.get("success"):
        await query.answer("Платеж успешно отменен.", show_alert=True)
        await bot.edit_message_text("Платеж отменен.", chat_id=query.message.chat.id, message_id=query.message.message_id, reply_markup=None)
    else:
        error_message = response.get("error", "Произошла ошибка. Попробуйте позже.")
        await query.answer(f"Ошибка: {error_message}", show_alert=True)
