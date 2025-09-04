from aiogram import Router, F
from aiogram.filters import Command
from aiogram.types import Message, CallbackQuery
from aiogram.utils.markdown import hbold

from api import api_client
from keyboards import inline
from config import settings

router = Router()

@router.message(Command("start"))
async def start_handler(message: Message):
    try:
        await api_client.register_user(message.from_user.id)
        await message.answer(
            f"Добро пожаловать, {hbold(message.from_user.full_name)}!\n\n"
            f"Я - ваш личный помощник для покупок. Здесь вы можете:\n"
            f"- 🛍️ Смотреть каталог товаров\n"
            f"- 💰 Пополнять баланс\n"
            f"- 💳 Проверять свой счет\n\n"
            f"Выберите действие в меню ниже:",
            reply_markup=inline.main_menu(),
            parse_mode="HTML"
        )
    except Exception as e:
        await message.answer(f"Произошла ошибка: {e}")

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
