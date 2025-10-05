import asyncio
import logging
import json

from aiogram import Bot, Dispatcher
from aiogram.client.default import DefaultBotProperties
from aiogram.fsm.storage.redis import RedisStorage
import redis.asyncio as redis

from config import settings
from handlers import start, balance, catalog, buy, referral, my_bots, my_subscriptions, my_orders
from api import APIClient
from logging_config import setup_logging
from middleware.block_check import BlockCheckMiddleware

async def redis_listener(bot: Bot, redis_client: redis.Redis, bot_username: str):
    pubsub = redis_client.pubsub()
    channel = f"bot-notifications:{bot_username}"
    await pubsub.subscribe(channel)
    logging.info(f"Subscribed to Redis channel: {channel}")

    while True:
        try:
            message = await pubsub.get_message(ignore_subscribe_messages=True, timeout=60)
            if message is None:
                continue

            logging.info(f"Received message from Redis: {message}")
            message_data = json.loads(message['data'])
            telegram_id = message_data.get('telegram_id')
            text = message_data.get('message')
            message_to_edit = message_data.get('message_to_edit')

            if telegram_id and text:
                if message_to_edit:
                    try:
                        await bot.edit_message_text(
                            chat_id=telegram_id, 
                            message_id=message_to_edit, 
                            text=text, 
                            reply_markup=None
                        )
                        logging.info(f"Edited message {message_to_edit} for user {telegram_id}")
                    except Exception as e:
                        logging.warning(f"Could not edit message {message_to_edit}, sending new one. Error: {e}")
                        await bot.send_message(chat_id=telegram_id, text=text)
                else:
                    await bot.send_message(chat_id=telegram_id, text=text)
                    logging.info(f"Sent notification to user {telegram_id}")

        except asyncio.CancelledError:
            logging.info("Redis listener task cancelled.")
            break
        except Exception as e:
            logging.exception("Error in Redis listener")
            await asyncio.sleep(5) # Avoid spamming logs on persistent errors

async def main():
    setup_logging()
    logging.info(f"Bot starting. Type: {settings.bot_type}")

    if not settings.bot_token:
        logging.error("BOT_TOKEN environment variable not set. Exiting.")
        return

    redis_client = redis.Redis(host=settings.redis_host, port=settings.redis_port, decode_responses=True)
    storage = RedisStorage(redis_client)

    bot = Bot(token=settings.bot_token, default=DefaultBotProperties(parse_mode="HTML"))
    me = await bot.get_me()
    api_client = APIClient(me.username)
    dp = Dispatcher(storage=storage, api_client=api_client)

    dp.update.middleware(BlockCheckMiddleware())

    dp.include_router(start.router)
    dp.include_router(balance.router)
    dp.include_router(catalog.router)
    dp.include_router(buy.router)
    dp.include_router(referral.router)
    dp.include_router(my_bots.router)
    dp.include_router(my_subscriptions.router)
    dp.include_router(my_orders.router)

    await bot.delete_webhook(drop_pending_updates=True)
    
    listener_task = asyncio.create_task(redis_listener(bot, redis_client, me.username))

    try:
        await dp.start_polling(bot)
    finally:
        logging.info("Stopping bot, cancelling listener task...")
        listener_task.cancel()
        await listener_task

if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        logging.info("Bot stopped by user.")
