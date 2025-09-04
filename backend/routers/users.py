from fastapi import APIRouter, Depends, status
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy import select
import traceback

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
            if not existing_user.is_deleted:
                response_data = {
                    "user": models.BotUser.model_validate(existing_user).model_dump(),
                    "is_new": False,
                    "has_passed_captcha": existing_user.has_passed_captcha
                }
                return success_response(response_data)
            else:
                # If user exists but is deleted, create a new one (effectively undelete and reset)
                existing_user.is_deleted = False
                existing_user.balance = 0 # Reset balance for "new" user
                existing_user.has_passed_captcha = False # Must pass captcha again
                await db.commit()
                await db.refresh(existing_user)
                response_data = {
                    "user": models.BotUser.model_validate(existing_user).model_dump(),
                    "is_new": True, # Treat as new for the bot's perspective
                    "has_passed_captcha": False
                }
                return success_response(response_data, status_code=status.HTTP_201_CREATED)

        db_user = db_models.BotUser(telegram_id=user.telegram_id, balance=0, has_passed_captcha=False)
        db.add(db_user)
        await db.commit()
        await db.refresh(db_user)
        response_data = {
            "user": models.BotUser.model_validate(db_user).model_dump(),
            "is_new": True,
            "has_passed_captcha": False
        }
        return success_response(response_data, status_code=status.HTTP_201_CREATED)
    except Exception as e:
        traceback.print_exc()
        return error_response(str(e), status_code=status.HTTP_500_INTERNAL_SERVER_ERROR)

@router.get("/{user_id}")
async def read_bot_user(
    user_id: int,
    db: AsyncSession = Depends(database.get_db),
    _ = Depends(security.verify_service_token)
):
    try:
        result = await db.execute(select(db_models.BotUser).filter(
            db_models.BotUser.id == user_id,
            db_models.BotUser.is_deleted == False
        ))
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
        result = await db.execute(select(db_models.BotUser).filter(
            db_models.BotUser.telegram_id == user_id,
            db_models.BotUser.is_deleted == False
        ))
        user = result.scalars().first()
        if user is None:
            return error_response("Bot user not found", status_code=status.HTTP_404_NOT_FOUND)
    except Exception as e:
        return error_response(str(e), status_code=status.HTTP_500_INTERNAL_SERVER_ERROR)

@router.put("/{user_id}/captcha-status")
async def update_user_captcha_status(
    user_id: int,
    captcha_status: dict, # This will contain {"has_passed_captcha": True/False}
    db: AsyncSession = Depends(database.get_db),
    _ = Depends(security.verify_service_token)
):
    try:
        result = await db.execute(select(db_models.BotUser).filter(db_models.BotUser.id == user_id))
        user = result.scalars().first()
        if user is None:
            return error_response("Bot user not found", status_code=status.HTTP_404_NOT_FOUND)
        
        user.has_passed_captcha = captcha_status.get("has_passed_captcha", False)
        await db.commit()
        await db.refresh(user)
        return success_response({"message": "Captcha status updated successfully"})
    except Exception as e:
        return error_response(str(e), status_code=status.HTTP_500_INTERNAL_SERVER_ERROR)