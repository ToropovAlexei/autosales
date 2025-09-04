from aiogram import Router
from aiogram.filters import Command
from aiogram.types import Message

from api import api_client
from keyboards import inline

router = Router()

@router.message(Command("start"))
async def start_handler(message: Message):
    try:
        await api_client.register_user(message.from_user.id)
        await message.answer("Добро пожаловать!", reply_markup=inline.main_menu())
    except Exception as e:
        await message.answer(f"Произошла ошибка: {e}")
