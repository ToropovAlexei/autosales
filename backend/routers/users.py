from fastapi import APIRouter, Depends, status
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy import select

from models import models
from db import database, db_models
from security import security
from core.responses import success_response, error_response

router = APIRouter()

@router.get("/me")
async def read_users_me(current_user: models.User = Depends(security.get_current_active_user)):
    return success_response(models.User.model_validate(current_user).model_dump())

@router.post("/register", status_code=status.HTTP_200_OK)
async def register_bot_user(
    user: models.BotUserCreate,
    db: AsyncSession = Depends(database.get_db),
    _ = Depends(security.verify_service_token)
):
    try:
        # Check if user already exists
        result = await db.execute(select(db_models.BotUser).filter(db_models.BotUser.telegram_id == user.telegram_id))
        existing_user = result.scalars().first()
        if existing_user:
            response_data = {
                "user": models.BotUser.model_validate(existing_user).model_dump(),
                "is_new": False
            }
            return success_response(response_data)

        db_user = db_models.BotUser(telegram_id=user.telegram_id, balance=0)
        db.add(db_user)
        await db.commit()
        await db.refresh(db_user)
        response_data = {
            "user": models.BotUser.model_validate(db_user).model_dump(),
            "is_new": True
        }
        return success_response(response_data, status_code=status.HTTP_201_CREATED)
    except Exception as e:
        return error_response(str(e), status_code=status.HTTP_500_INTERNAL_SERVER_ERROR)

@router.get("/{user_id}")
async def read_bot_user(
    user_id: int,
    db: AsyncSession = Depends(database.get_db),
    _ = Depends(security.verify_service_token)
):
    try:
        result = await db.execute(select(db_models.BotUser).filter(db_models.BotUser.id == user_id))
        user = result.scalars().first()
        if user is None:
            return error_response("Bot user not found", status_code=status.HTTP_404_NOT_FOUND)
        return success_response(models.BotUser.model_validate(user).model_dump())
    except Exception as e:
        return error_response(str(e), status_code=status.HTTP_500_INTERNAL_SERVER_ERROR)


@router.get("/{user_id}/balance")
async def get_balance(
    user_id: int,
    db: AsyncSession = Depends(database.get_db),
    _ = Depends(security.verify_service_token)
):
    try:
        result = await db.execute(select(db_models.BotUser).filter(db_models.BotUser.telegram_id == user_id))
        user = result.scalars().first()
        if user is None:
            return error_response("Bot user not found", status_code=status.HTTP_404_NOT_FOUND)
        return success_response({"balance": user.balance})
    except Exception as e:
        return error_response(str(e), status_code=status.HTTP_500_INTERNAL_SERVER_ERROR)