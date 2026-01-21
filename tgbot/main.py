import asyncio
import logging
import json

from aiogram import Bot, Dispatcher, BaseMiddleware
from aiogram.client.default import DefaultBotProperties
from aiogram.exceptions import TelegramForbiddenError
from aiogram.fsm.storage.redis import RedisStorage
from aiogram.fsm.context import FSMContext
from aiogram.fsm.state import State, StatesGroup
from aiogram.fsm.storage.base import StorageKey
from aiogram.types import TelegramObject, InlineKeyboardButton, InlineKeyboardMarkup, BufferedInputFile
from typing import Callable, Dict, Any, Awaitable
import redis.asyncio as redis

from keyboards.inline import back_to_main_menu_keyboard

from config import settings
from handlers import start, balance, catalog, buy, referral, my_bots, my_subscriptions, my_orders, payment, admin, my_payments, payment_receipt
from handlers.payment_receipt import PaymentReceiptStates
from api import APIClient
from logging_config import setup_logging
from middleware.block_check import BlockCheckMiddleware
from middleware.unblock_user import UnblockUserMiddleware

# Global flag to control bot operation
BOT_CAN_OPERATE = True

STATE_MAP = {
    "payment_awaiting_receipt_link": PaymentReceiptStates.awaiting_receipt_link,
}

class CanOperateMiddleware(BaseMiddleware):
    async def __call__(
        self,
        handler: Callable[[TelegramObject, Dict[str, Any]], Awaitable[Any]],
        event: TelegramObject,
        data: Dict[str, Any],
    ) -> Any:
        if not BOT_CAN_OPERATE:
            logging.warning("Store balance is low. Bot is disabled. Ignoring update.")
            return
        return await handler(event, data)

async def check_bot_status(api_client: APIClient):
    """Periodically checks with the backend if the bot is allowed to operate."""
    global BOT_CAN_OPERATE
    while True:
        try:
            response = await api_client.get_bot_status()
            can_operate = response.get("data", {}).get("can_operate", False)
            if can_operate != BOT_CAN_OPERATE:
                logging.info(f"Bot operational status changed to: {can_operate}")
                BOT_CAN_OPERATE = can_operate
        except Exception as e:
            logging.exception(f"Failed to check bot operational status: {e}. Assuming it cannot operate.")
            BOT_CAN_OPERATE = False
        await asyncio.sleep(30)


async def redis_listener(dp: Dispatcher, bot: Bot, redis_client: redis.Redis, bot_username: str, api_client: APIClient):
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
            if not telegram_id:
                logging.warning("Received message from Redis without telegram_id")
                continue

            # Action variables
            text = message_data.get('message')
            image_id = message_data.get('image_id')
            message_to_edit = message_data.get('message_to_edit')
            message_to_delete = message_data.get('message_to_delete')
            inline_keyboard_data = message_data.get('inline_keyboard')
            state_to_set = message_data.get('state_to_set')
            state_data = message_data.get('state_data')
            
            reply_markup = None
            if inline_keyboard_data:
                buttons = [
                    [InlineKeyboardButton(text=button['text'], callback_data=button['callback_data']) for button in row]
                    for row in inline_keyboard_data
                ]
                reply_markup = InlineKeyboardMarkup(inline_keyboard=buttons)

            try:
                # Handle FSM State if provided
                if state_to_set:
                    state_object = STATE_MAP.get(state_to_set)
                    if state_object:
                        fsm_context = FSMContext(storage=dp.storage, key=StorageKey(bot_id=bot.id, chat_id=telegram_id, user_id=telegram_id))
                        await fsm_context.set_state(state_object)
                        if state_data:
                            await fsm_context.set_data(state_data)
                        logging.info(f"Set FSM state for user {telegram_id} to {state_to_set} with data {state_data}")
                    else:
                        logging.error(f"Unknown state to set: {state_to_set}")

                # Priority 1: Delete a message
                if message_to_delete:
                    await bot.delete_message(chat_id=telegram_id, message_id=message_to_delete)
                    logging.info(f"Deleted message {message_to_delete} for user {telegram_id}")

                # Priority 2: Edit a message
                if message_to_edit:
                    await bot.edit_message_text(
                        chat_id=telegram_id,
                        message_id=message_to_edit,
                        text=text,
                        reply_markup=reply_markup
                    )
                    logging.info(f"Edited message {message_to_edit} for user {telegram_id}")
                
                # Priority 3: Send a new message (broadcast or simple notification)
                elif image_id:
                    image_path = f"/images/{image_id}"
                    image_data = await api_client.get_image(image_path)
                    if image_data:
                        input_file = BufferedInputFile(image_data, filename="image.png")
                        await bot.send_photo(
                            chat_id=telegram_id,
                            photo=input_file,
                            caption=text, # Use text as caption
                            reply_markup=reply_markup
                        )
                        logging.info(f"Sent photo to user {telegram_id}")
                    elif text: # Fallback to text if image fails
                        await bot.send_message(chat_id=telegram_id, text=text, reply_markup=reply_markup)
                        logging.warning(f"Sent text-only message to user {telegram_id} (image fetch failed)")
                elif text:
                    await bot.send_message(chat_id=telegram_id, text=text, reply_markup=reply_markup)
                    logging.info(f"Sent text message to user {telegram_id}")

            except TelegramForbiddenError:
                logging.warning(f"User {telegram_id} has blocked the bot. Marking as blocked.")
                await api_client.update_user_status(telegram_id, {"bot_is_blocked_by_user": True})
            except Exception as e:
                logging.error(f"Failed to process message for user {telegram_id}: {e}")

        except asyncio.CancelledError:
            logging.info("Redis listener task cancelled.")
            break
        except Exception as e:
            logging.exception("Error in Redis listener main loop")
            await asyncio.sleep(5)

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
    dp = Dispatcher(storage=storage, api_client=api_client, bot=bot)

    dp.update.middleware(UnblockUserMiddleware())
    dp.update.middleware(BlockCheckMiddleware())
    dp.update.middleware(CanOperateMiddleware()) # Register the new middleware

    dp.include_router(start.router)
    dp.include_router(balance.router)
    dp.include_router(catalog.router)
    dp.include_router(buy.router)
    dp.include_router(referral.router)
    dp.include_router(my_bots.router)
    dp.include_router(my_subscriptions.router)
    dp.include_router(my_orders.router)
    dp.include_router(payment.router)
    dp.include_router(admin.router)
    dp.include_router(my_payments.router)
    dp.include_router(payment_receipt.router)


    await bot.delete_webhook(drop_pending_updates=True)
    
    # Start background tasks
    listener_task = asyncio.create_task(redis_listener(dp, bot, redis_client, me.username, api_client))
    status_check_task = asyncio.create_task(check_bot_status(api_client))


    try:
        await dp.start_polling(bot)
    finally:
        logging.info("Stopping bot, cancelling background tasks...")
        listener_task.cancel()
        status_check_task.cancel()
        await asyncio.gather(listener_task, status_check_task, return_exceptions=True)


if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        logging.info("Bot stopped by user.")