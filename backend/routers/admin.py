from typing import List
from fastapi import APIRouter, Depends, status
from fastapi.responses import JSONResponse
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy import select, func
import traceback

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
        result = await db.execute(select(db_models.BotUser).filter(db_models.BotUser.is_deleted == False))
        bot_users = result.scalars().all()
        for user in bot_users:
            balance_result = await db.execute(
                select(func.sum(db_models.Transaction.amount)).filter(db_models.Transaction.user_id == user.id)
            )
            user.balance = balance_result.scalar_one_or_none() or 0
        return success_response([models.BotUser.model_validate(u).model_dump() for u in bot_users])
    except Exception as e:
        return error_response(str(e), status_code=status.HTTP_500_INTERNAL_SERVER_ERROR)

@router.delete("/bot-users/{user_id}", status_code=status.HTTP_204_NO_CONTENT)
async def delete_bot_user(
    user_id: int,
    db: AsyncSession = Depends(database.get_db),
    current_user: models.User = Depends(security.get_current_admin_user)
):
    try:
        result = await db.execute(select(db_models.BotUser).filter(db_models.BotUser.id == user_id))
        bot_user = result.scalars().first()
        if bot_user is None:
            return error_response("Bot user not found", status_code=status.HTTP_404_NOT_FOUND)

        bot_user.is_deleted = True
        await db.commit()
        await db.refresh(bot_user)
        return JSONResponse(status_code=status.HTTP_204_NO_CONTENT, content="")
    except Exception as e:
        traceback.print_exc()
        return error_response(str(e), status_code=status.HTTP_500_INTERNAL_SERVER_ERROR)