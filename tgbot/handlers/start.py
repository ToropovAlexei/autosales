from aiogram import Router, F
from aiogram.filters import Command
from aiogram.types import Message, CallbackQuery, BufferedInputFile, InlineKeyboardButton, InputMediaPhoto
from aiogram.utils.markdown import hbold
from aiogram.fsm.context import FSMContext
from aiogram.types import InlineKeyboardMarkup

from api import api_client
from keyboards import inline
from config import settings
from states import CaptchaState
from captcha_helper import generate_captcha_and_options

router = Router()

def captcha_keyboard(options: list):
    buttons = []
    for option in options:
        buttons.append([InlineKeyboardButton(text=option, callback_data=f"captcha_{option}")])
    return InlineKeyboardMarkup(inline_keyboard=buttons)

@router.message(Command("start"))
async def start_handler(message: Message, state: FSMContext):
    try:
        response = await api_client.register_user(message.from_user.id)
        if response.get("success"):
            data = response["data"]
            if data["is_new"]:
                # New user, show captcha
                captcha_image, correct_answer, options = generate_captcha_and_options()
                await state.set_state(CaptchaState.waiting_for_answer)
                await state.update_data(correct_answer=correct_answer)
                
                await message.answer_photo(
                    photo=BufferedInputFile(captcha_image.getvalue(), "captcha.png"),
                    caption="Пожалуйста, решите капчу, чтобы продолжить:",
                    reply_markup=captcha_keyboard(options)
                )
            else:
                # Existing user, show main menu
                await message.answer(
                    f"С возвращением, {hbold(message.from_user.full_name)}!",
                    reply_markup=inline.main_menu(),
                    parse_mode="HTML"
                )
    except Exception as e:
        await message.answer(f"Произошла ошибка: {e}")

@router.callback_query(CaptchaState.waiting_for_answer, F.data.startswith("captcha_"))
async def captcha_answer_handler(callback_query: CallbackQuery, state: FSMContext):
    answer = callback_query.data.split("_")[1]
    data = await state.get_data()
    correct_answer = data.get("correct_answer")

    if answer == correct_answer:
        await callback_query.message.delete()
        await callback_query.message.answer(
            f"Добро пожаловать, {hbold(callback_query.from_user.full_name)}!\n\n"
            f"Я - ваш личный помощник для покупок. Здесь вы можете:\n"
            f"- 🛍️ Смотреть каталог товаров\n"
            f"- 💰 Пополнять баланс\n"
            f"- 💳 Проверять свой счет\n\n"
            f"Выберите действие в меню ниже:",
            reply_markup=inline.main_menu(),
            parse_mode="HTML"
        )
        await state.clear()
    else:
        await callback_query.answer("Неверный ответ, попробуйте еще раз.", show_alert=True)
        # Regenerate captcha
        captcha_image, correct_answer, options = generate_captcha_and_options()
        await state.update_data(correct_answer=correct_answer)
        await callback_query.message.edit_media(
            media=InputMediaPhoto(media=BufferedInputFile(captcha_image.getvalue(), "captcha.png"), caption="Пожалуйста, решите капчу, чтобы продолжить:"),
            reply_markup=captcha_keyboard(options)
        )

@router.callback_query(F.data == "main_menu")
async def main_menu_handler(callback_query: CallbackQuery):
    await callback_query.message.edit_text(
        "Главное меню",
        reply_markup=inline.main_menu()
    )

@router.callback_query(F.data == "support")
async def support_handler(callback_query: CallbackQuery):
    await callback_query.message.edit_text(
        f"Для связи с поддержкой, пожалуйста, напишите нам: {settings.support_url}",
        reply_markup=inline.main_menu()
    )