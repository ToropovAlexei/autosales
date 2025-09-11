import asyncio
import logging

from aiogram import Bot, Dispatcher
from aiogram.fsm.storage.redis import RedisStorage
from redis.asyncio import Redis

from config import settings
from handlers import start, balance, catalog, buy, referral
from api import api_client

async def run_bot(token: str, storage: RedisStorage):
    bot = Bot(token=token)
    dp = Dispatcher(storage=storage)

    dp.include_router(start.router)
    dp.include_router(balance.router)
    dp.include_router(catalog.router)
    dp.include_router(buy.router)
    dp.include_router(referral.router)

    await bot.delete_webhook(drop_pending_updates=True)
    await dp.start_polling(bot)

async def main():
    logging.basicConfig(level=logging.INFO)

    redis = Redis(host=settings.redis_host, port=settings.redis_port)
    storage = RedisStorage(redis)

    # Fetch referral bots
    try:
        response = await api_client.get_referral_bots()
        if response.get("status") == "success":
            referral_bots = response.get("data", [])
            all_tokens = [bot["bot_token"] for bot in referral_bots]
        else:
            logging.error(f"Failed to fetch referral bots: {response.get('message')}")
            all_tokens = []
    except Exception as e:
        logging.error(f"Error fetching referral bots: {e}")
        all_tokens = []

    # Add the main bot token
    if settings.bot_token:
        all_tokens.append(settings.bot_token)
    
    if not all_tokens:
        logging.error("No bot tokens found. Exiting.")
        return

    tasks = [run_bot(token, storage) for token in all_tokens]
    await asyncio.gather(*tasks)

if __name__ == "__main__":
    asyncio.run(main())
