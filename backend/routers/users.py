from fastapi import APIRouter, Depends, status
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy import select, func, update
from typing import List
import traceback

from models import models
from db import database, db_models
from security import security
from core.responses import success_response, error_response

router = APIRouter()

@router.get("/me")
async def read_users_me(current_user: models.User = Depends(security.get_current_active_user)):
    return success_response(models.User.model_validate(current_user).model_dump())

@router.get("/seller-settings")
async def get_seller_settings(
    db: AsyncSession = Depends(database.get_db),
    _ = Depends(security.verify_service_token)
):
    try:
        # Assuming the seller is the admin user
        result = await db.execute(select(db_models.User).filter(db_models.User.role == models.UserRole.admin))
        seller = result.scalars().first()
        if seller is None:
            # Fallback to the first user if no admin found
            result = await db.execute(select(db_models.User))
            seller = result.scalars().first()
            if seller is None:
                return error_response("Seller not found", status_code=status.HTTP_404_NOT_FOUND)

        return success_response({
            "id": seller.id,
            "referral_program_enabled": seller.referral_program_enabled,
            "referral_percentage": seller.referral_percentage
        })
    except Exception as e:
        traceback.print_exc()
        return error_response(str(e), status_code=status.HTTP_500_INTERNAL_SERVER_ERROR)

@router.put("/me/referral-settings")
async def update_referral_settings(
    settings: models.ReferralSettings,
    current_user: models.User = Depends(security.get_current_active_user),
    db: AsyncSession = Depends(database.get_db)
):
    try:
        if current_user.role not in [models.UserRole.admin, models.UserRole.seller]:
            return error_response("Not enough permissions", status_code=status.HTTP_403_FORBIDDEN)
        if settings.referral_percentage < 0 or settings.referral_percentage > 100:
            return error_response("Referral percentage must be between 0 and 100", status_code=status.HTTP_400_BAD_REQUEST)

        stmt = (
            update(db_models.User)
            .where(db_models.User.id == current_user.id)
            .values(
                referral_program_enabled=settings.referral_program_enabled,
                referral_percentage=settings.referral_percentage
            )
        )
        await db.execute(stmt)
        await db.commit()
        return success_response({"message": "Referral settings updated successfully"})
    except Exception as e:
        traceback.print_exc()
        return error_response(str(e), status_code=status.HTTP_500_INTERNAL_SERVER_ERROR)

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
                existing_user.has_passed_captcha = False # Must pass captcha again
                await db.commit()
                await db.refresh(existing_user)
                response_data = {
                    "user": models.BotUser.model_validate(existing_user).model_dump(),
                    "is_new": True, # Treat as new for the bot's perspective
                    "has_passed_captcha": False
                }
                return success_response(response_data, status_code=status.HTTP_201_CREATED)

        db_user = db_models.BotUser(telegram_id=user.telegram_id, has_passed_captcha=False)
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
            db_models.BotUser.is_deleted.is_(False)
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
            db_models.BotUser.is_deleted.is_(False)
        ))
        user = result.scalars().first()
        if user is None:
            return error_response("Bot user not found", status_code=status.HTTP_404_NOT_FOUND)

        balance_result = await db.execute(
            select(func.sum(db_models.Transaction.amount)).filter(db_models.Transaction.user_id == user.id)
        )
        balance = balance_result.scalar_one_or_none() or 0
        return success_response({"balance": balance})
    except Exception as e:
        return error_response(str(e), status_code=status.HTTP_500_INTERNAL_SERVER_ERROR)

@router.get("/{user_id}/transactions", response_model=List[models.Transaction])
async def get_transactions(
    user_id: int,
    db: AsyncSession = Depends(database.get_db),
    _ = Depends(security.verify_service_token)
):
    try:
        result = await db.execute(select(db_models.BotUser).filter(
            db_models.BotUser.telegram_id == user_id,
            db_models.BotUser.is_deleted.is_(False)
        ))
        user = result.scalars().first()
        if user is None:
            return error_response("Bot user not found", status_code=status.HTTP_404_NOT_FOUND)

        transactions_result = await db.execute(
            select(db_models.Transaction).filter(db_models.Transaction.user_id == user.id).order_by(db_models.Transaction.created_at.desc())
        )
        transactions = transactions_result.scalars().all()
        return success_response(transactions)
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