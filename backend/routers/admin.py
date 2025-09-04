from typing import List
from fastapi import APIRouter, Depends, status
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy import select

from models import models
from db import database, db_models
from security import security
from core.responses import success_response, error_response

router = APIRouter()

@router.get("/bot-users")
async def read_bot_users(
    db: AsyncSession = Depends(database.get_db),
    current_user: models.User = Depends(security.get_current_admin_user)
):
    try:
        result = await db.execute(select(db_models.BotUser))
        bot_users = result.scalars().all()
        return success_response([models.BotUser.model_validate(u).model_dump() for u in bot_users])
    except Exception as e:
        return error_response(str(e), status_code=status.HTTP_500_INTERNAL_SERVER_ERROR)