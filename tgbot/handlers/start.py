from aiogram import Router, F
import logging
from aiogram.filters import Command
from aiogram.types import Message, CallbackQuery, BufferedInputFile, InlineKeyboardButton, InputMediaPhoto
from aiogram.utils.markdown import hbold
from aiogram.fsm.context import FSMContext
from aiogram.types import InlineKeyboardMarkup
import contextlib

from api import APIClient
from keyboards import inline
from config import settings
from states import CaptchaState
from captcha_helper import generate_captcha_and_options

from aiogram.exceptions import TelegramBadRequest

router = Router()

def captcha_keyboard(options: list):
    buttons = []
    for option in options:
        buttons.append([InlineKeyboardButton(text=option, callback_data=f"captcha_{option}")])
    return InlineKeyboardMarkup(inline_keyboard=buttons)

async def update_pinned_message(message: Message):
    if not settings.fallback_bot_username:
        return

    try:
        chat = await message.bot.get_chat(message.chat.id)
        new_text = f"🤖 Наш резервный бот: @{settings.fallback_bot_username}"

        if chat.pinned_message and chat.pinned_message.text == new_text:
            return

        with contextlib.suppress(Exception):
            await message.bot.unpin_all_chat_messages(message.chat.id)
        
        sent_message = await message.answer(new_text)
        with contextlib.suppress(Exception):
            await sent_message.pin(disable_notification=True)

    except Exception as e:
        print(f"Error updating pinned message: {e}")

@router.message(Command("start"))
async def start_handler(message: Message, state: FSMContext, api_client: APIClient):
    try:
        args = message.text.split()
        if len(args) > 1:
            try:
                referral_bot_id = int(args[1])
                await state.update_data(referral_bot_id=referral_bot_id)
            except (ValueError, IndexError):
                pass  # Ignore if the payload is not a valid integer

        response = await api_client.register_user(message.from_user.id)
        if response.get("success"):
            user_data = response["data"]

            if user_data.get("is_blocked"):
                await message.answer("Ваш аккаунт заблокирован.")
                return

            if not user_data.get("has_passed_captcha"):
                captcha_image, correct_answer, options = generate_captcha_and_options()
                await state.set_state(CaptchaState.waiting_for_answer)
                await state.update_data(correct_answer=correct_answer, user_id=user_data["id"], telegram_id=message.from_user.id)
                
                await message.answer_photo(
                    photo=BufferedInputFile(captcha_image.getvalue(), "captcha.png"),
                    caption="Пожалуйста, решите капчу, чтобы продолжить:",
                    reply_markup=captcha_keyboard(options)
                )
            else:
                await update_pinned_message(message)
                seller_info_response = await api_client.get_public_settings()
                referral_program_enabled = seller_info_response.get("data", {}).get("referral_program_enabled", False) == 'true'

                await message.answer(
                    f"С возвращением, {hbold(message.from_user.full_name)}!",
                    reply_markup=inline.main_menu(
                        referral_program_enabled=referral_program_enabled,
                        bot_type=settings.bot_type
                    ),
                    parse_mode="HTML"
                )
    except Exception:
        logging.exception("An error occurred in start_handler")
        await message.answer("Произошла непредвиденная ошибка. Попробуйте позже.")

@router.callback_query(CaptchaState.waiting_for_answer, F.data.startswith("captcha_"))
async def captcha_answer_handler(callback_query: CallbackQuery, state: FSMContext, api_client: APIClient):
    answer = callback_query.data.split("_")[1]
    data = await state.get_data()
    correct_answer = data.get("correct_answer")
    user_id = data.get("user_id")
    telegram_id = data.get("telegram_id")

    if user_id is None or telegram_id is None:
        await callback_query.answer("Произошла ошибка. Пожалуйста, попробуйте начать заново (/start).", show_alert=True)
        await state.clear()
        return

    if answer == correct_answer:
        try:
            update_response = await api_client.update_user_captcha_status(telegram_id, True)
            if not update_response.get("success"):
                await callback_query.answer(f"Ошибка при обновлении статуса капчи: {update_response.get('error')}", show_alert=True)
                return
        except Exception as e:
            await callback_query.answer(f"Ошибка при отправке запроса на обновление статуса капчи: {e}", show_alert=True)
            return

        await callback_query.message.delete()
        await update_pinned_message(callback_query.message)
        seller_info_response = await api_client.get_public_settings()
        referral_program_enabled = seller_info_response.get("data", {}).get("referral_program_enabled", False)

        await callback_query.message.answer(
            f"Добро пожаловать, {hbold(callback_query.from_user.full_name)}!\n\n"
            f"Я - ваш личный помощник для покупок. Здесь вы можете:\n"
            f"- 🛍️ Смотреть каталог товаров\n"
            f"- 💰 Пополнять баланс\n"
            f"- 💳 Проверять свой счет\n\n"
            f"Выберите действие в меню ниже:",
            reply_markup=inline.main_menu(
                referral_program_enabled=referral_program_enabled,
                bot_type=settings.bot_type
            ),
            parse_mode="HTML"
        )
        await state.clear()
    else:
        await callback_query.answer("Неверный ответ, попробуйте еще раз.", show_alert=True)
        captcha_image, correct_answer, options = generate_captcha_and_options()
        await state.update_data(correct_answer=correct_answer)
        await callback_query.message.edit_media(
            media=InputMediaPhoto(media=BufferedInputFile(captcha_image.getvalue(), "captcha.png"), caption="Пожалуйста, решите капчу, чтобы продолжить:"),
            reply_markup=captcha_keyboard(options)
        )

@router.callback_query(F.data == "main_menu")
async def main_menu_handler(callback_query: CallbackQuery, api_client: APIClient):
    seller_info_response = await api_client.get_public_settings()
    referral_program_enabled = seller_info_response.get("data", {}).get("referral_program_enabled", False)
    reply_markup = inline.main_menu(
        referral_program_enabled=referral_program_enabled,
        bot_type=settings.bot_type
    )
    try:
        await callback_query.message.edit_text(
            "Главное меню",
            reply_markup=reply_markup
        )
    except TelegramBadRequest:
        await callback_query.message.delete()
        await callback_query.message.answer(
            "Главное меню",
            reply_markup=reply_markup
        )

@router.callback_query(F.data == "support")
async def support_handler(callback_query: CallbackQuery, api_client: APIClient):
    seller_info_response = await api_client.get_public_settings()
    referral_program_enabled = seller_info_response.get("data", {}).get("referral_program_enabled", False)
    await callback_query.message.edit_text(
        f"Для связи с поддержкой, пожалуйста, напишите нам: {settings.support_url}",
        reply_markup=inline.main_menu(
            referral_program_enabled=referral_program_enabled,
            bot_type=settings.bot_type
        )
    )