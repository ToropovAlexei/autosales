import asyncio
import logging

from aiogram import Bot, Dispatcher
from aiogram.fsm.storage.redis import RedisStorage
from redis.asyncio import Redis

from config import settings
from handlers import start, balance, catalog, buy, referral, my_bots, my_subscriptions, my_orders
from api import api_client
from logging_config import setup_logging

async def main():
    setup_logging()
    logging.info(f"Bot starting. Type: {settings.bot_type}")

    if not settings.bot_token:
        logging.error("BOT_TOKEN environment variable not set. Exiting.")
        return

    redis = Redis(host=settings.redis_host, port=settings.redis_port)
    storage = RedisStorage(redis)

    bot = Bot(token=settings.bot_token)
    dp = Dispatcher(storage=storage)

    dp.include_router(start.router)
    dp.include_router(balance.router)
    dp.include_router(catalog.router)
    dp.include_router(buy.router)
    dp.include_router(referral.router)
    dp.include_router(my_bots.router)
    dp.include_router(my_subscriptions.router)
    dp.include_router(my_orders.router)

    await bot.delete_webhook(drop_pending_updates=True)
    await dp.start_polling(bot)

if __name__ == "__main__":
    asyncio.run(main())
