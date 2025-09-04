import asyncio
import logging

from aiogram import Bot, Dispatcher
from aiogram.fsm.storage.redis import RedisStorage
from redis.asyncio import Redis

from config import settings
from handlers import start, balance, catalog, buy

async def main():
    logging.basicConfig(level=logging.INFO)

    redis = Redis(host=settings.redis_host, port=settings.redis_port)
    storage = RedisStorage(redis)

    bot = Bot(token=settings.bot_token)
    dp = Dispatcher(storage=storage)

    dp.include_router(start.router)
    dp.include_router(balance.router)
    dp.include_router(catalog.router)
    dp.include_router(buy.router)

    await bot.delete_webhook(drop_pending_updates=True)
    await dp.start_polling(bot)

if __name__ == "__main__":
    asyncio.run(main())
