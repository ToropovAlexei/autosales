from typing import Callable, Dict, Any, Awaitable
from aiogram import BaseMiddleware
from aiogram.types import TelegramObject
from api import APIClient

class BlockCheckMiddleware(BaseMiddleware):
    async def __call__(
        self,
        handler: Callable[[TelegramObject, Dict[str, Any]], Awaitable[Any]],
        event: TelegramObject,
        data: Dict[str, Any]
    ) -> Any:
        # We only want to check users, not other events
        if not hasattr(event, 'from_user'):
            return await handler(event, data)

        user_id = event.from_user.id
        api_client: APIClient = data['api_client']

        try:
            user_response = await api_client.get_user(user_id)
            if user_response.get("success"):
                user_data = user_response.get("data", {})
                if user_data.get("is_blocked"):
                    # User is blocked, do not process the update
                    return
        except Exception:
            # If API is down or there's an error, let's fail open and process the update
            # Or you could fail closed by returning here
            pass

        return await handler(event, data)
