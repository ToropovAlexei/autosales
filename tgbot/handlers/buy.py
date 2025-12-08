from aiogram import Router, F, Bot
from aiogram.fsm.context import FSMContext
from aiogram.types import CallbackQuery, InlineKeyboardMarkup, BufferedInputFile
from aiogram.utils.markdown import hbold
import logging

from api import APIClient
from keyboards.inline import back_to_main_menu_keyboard, insufficient_balance_keyboard

router = Router()

async def _safe_edit_message(callback_query: CallbackQuery, text: str, reply_markup: InlineKeyboardMarkup = None):
    """
    Edits message text or caption, whichever is present, to avoid
    TelegramBadRequest when a message is a photo.
    """
    if callback_query.message and callback_query.message.photo:
        await callback_query.message.edit_caption(caption=text, reply_markup=reply_markup)
    elif callback_query.message:
        await callback_query.message.edit_text(text, reply_markup=reply_markup)

async def process_buy_result(callback_query: CallbackQuery, result: dict, bot: Bot, api_client: APIClient):
    if result.get("success"):
        data = result.get("data")
        if not isinstance(data, dict):
            logging.error(f"API returned success but data is not a dict: {data}")
            await _safe_edit_message(callback_query, "Произошла ошибка при обработке ответа сервера.")
            return

        new_balance = data.get("balance")
        product_name = data.get("product_name")
        product_price = data.get("product_price")
        fulfilled_content = data.get("fulfilled_content")
        image_url = data.get("image_url")
        fulfilled_image_url = data.get("fulfilled_image_url")

        logging.info(f"Fulfilled Image URL from API: {fulfilled_image_url}")

        if new_balance is not None and product_name and product_price is not None:
            success_message = (
                f"✅ Поздравляем! Вы успешно купили товар {hbold(product_name)} за {hbold(f'{product_price} ₽')}.\n\n"
                f"💳 Ваш новый баланс: {hbold(f'{new_balance} ₽')}"
            )

            if fulfilled_content:
                success_message += f"\n\n{hbold('Ваш товар:')}\n<pre>{fulfilled_content}</pre>"

            # Delete the old message with the 'buy' button
            await callback_query.message.delete()

            image_path_to_send = None
            if fulfilled_image_url:
                image_path_to_send = fulfilled_image_url
            elif image_url:
                image_path_to_send = image_url
            
            if image_path_to_send:
                image_bytes = await api_client.get_image(image_path_to_send)
                if image_bytes:
                    await bot.send_photo(
                        chat_id=callback_query.from_user.id,
                        photo=BufferedInputFile(image_bytes, filename="image.png"),
                        caption=success_message,
                        parse_mode="HTML",
                        reply_markup=back_to_main_menu_keyboard()
                    )
                else: # Fallback to text if image download fails
                     await bot.send_message(
                        chat_id=callback_query.from_user.id,
                        text=success_message,
                        parse_mode="HTML",
                        reply_markup=back_to_main_menu_keyboard()
                    )
            else:
                await bot.send_message(
                    chat_id=callback_query.from_user.id,
                    text=success_message,
                    parse_mode="HTML",
                    reply_markup=back_to_main_menu_keyboard()
                )
        else:
            logging.error(f"Missing keys in successful buy response data: {data}")
            await _safe_edit_message(callback_query, "Произошла ошибка при обработке покупки.")
    else:
        error = result.get("error", "Произошла неизвестная ошибка.")
        if error == "Insufficient Balance":
            error_message = "😔 Недостаточно средств на балансе для совершения покупки. Пожалуйста, пополните баланс."
            await _safe_edit_message(callback_query, error_message, reply_markup=insufficient_balance_keyboard())
        elif error == "Product out of stock":
            error_message = "😔 К сожалению, этот товар закончился."
            await _safe_edit_message(callback_query, error_message)
        else:
            error_message = "Произошла непредвиденная ошибка. Попробуйте позже."
            await _safe_edit_message(callback_query, error_message)

@router.callback_query(F.data.startswith("buy_"))
async def buy_handler(callback_query: CallbackQuery, state: FSMContext, api_client: APIClient, bot: Bot):
    try:
        parts = callback_query.data.split('_')
        telegram_id = callback_query.from_user.id

        data = await state.get_data()
        referral_bot_id = data.get("referral_bot_id")

        _, product_id_str = parts
        product_id = int(product_id_str)
        result = await api_client.buy_product(telegram_id, product_id, referral_bot_id=referral_bot_id)
        
        await process_buy_result(callback_query, result, bot, api_client)

    except Exception as e:
        logging.exception("An unexpected error occurred in buy_handler")
        await _safe_edit_message(callback_query, "Произошла непредвиденная ошибка. Попробуйте позже.")
    finally:
        await callback_query.answer()
