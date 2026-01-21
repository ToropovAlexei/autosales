import logging
import re

from aiogram import Router, types, F
from aiogram.fsm.context import FSMContext
from aiogram.fsm.state import State, StatesGroup

from api import APIClient
from keyboards.inline import back_to_main_menu_keyboard

logger = logging.getLogger(__name__)

class PaymentReceiptStates(StatesGroup):
    awaiting_receipt_link = State()

router = Router()

@router.message(PaymentReceiptStates.awaiting_receipt_link, F.text)
async def process_receipt_link(message: types.Message, state: FSMContext, api_client: APIClient):
    user_id = message.from_user.id
    receipt_url = message.text.strip()
    
    # Basic URL validation
    if not re.match(r'^https?://(?:www\.)?dropmefiles\.com/\S+$', receipt_url):
        await message.answer(
            "Некорректная ссылка. Пожалуйста, отправьте действительную ссылку на чек с dropmefiles.com.",
            reply_markup=back_to_main_menu_keyboard()
        )
        return

    data = await state.get_data()
    order_id = data.get("order_id")

    if not order_id:
        logger.error(f"Order ID not found in state data for user {user_id} while processing receipt link.")
        await message.answer(
            "Произошла ошибка: не удалось найти идентификатор заказа. Пожалуйста, попробуйте снова или свяжитесь с поддержкой.",
            reply_markup=back_to_main_menu_keyboard()
        )
        await state.clear()
        return

    try:
        response = await api_client.submit_receipt_link(order_id, receipt_url)
        if response.get("success"):
            await message.answer(
                "Ссылка на чек успешно отправлена! Ожидайте обновления статуса платежа.",
                reply_markup=back_to_main_menu_keyboard()
            )
            logger.info(f"Receipt link for order {order_id} submitted by user {user_id}.")
        else:
            error_message = response.get("error", "Неизвестная ошибка при отправке ссылки на чек.")
            await message.answer(
                f"Ошибка: {error_message}",
                reply_markup=back_to_main_menu_keyboard()
            )
            logger.error(f"Failed to submit receipt link for order {order_id} by user {user_id}: {error_message}")
    except Exception as e:
        logger.exception(f"Exception while submitting receipt link for order {order_id} by user {user_id}: {e}")
        await message.answer(
            "Произошла непредвиденная ошибка при отправке ссылки на чек. Пожалуйста, попробуйте позже.",
            reply_markup=back_to_main_menu_keyboard()
        )
    finally:
        await state.clear()

