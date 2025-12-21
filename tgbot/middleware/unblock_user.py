from typing import Callable, Dict, Any, Awaitable
from aiogram import BaseMiddleware
from aiogram.types import TelegramObject
from api import APIClient
import logging

class UnblockUserMiddleware(BaseMiddleware):
    async def __call__(
        self,
        handler: Callable[[TelegramObject, Dict[str, Any]], Awaitable[Any]],
        event: TelegramObject,
        data: Dict[str, Any]
    ) -> Any:
        user = data.get('event_from_user')

        # If no user is associated with this event, do nothing.
        if not user:
            return await handler(event, data)

        user_id = user.id
        api_client: APIClient = data['api_client']

        try:
            user_response = await api_client.get_user(user_id)
            if user_response.get("success"):
                user_data = user_response.get("data", {})
                if user_data.get("bot_is_blocked_by_user"):
                    logging.info(f"User {user_id} has unblocked the bot. Resetting status.")
                    await api_client.update_user_status(user_id, {"bot_is_blocked_by_user": False})
        except Exception as e:
            # If the API call fails, we can log it but should proceed.
            # Failing to unblock is not critical for the current interaction.
            logging.error(f"Failed to check or update unblock status for user {user_id}: {e}")

        return await handler(event, data)
