from fastapi import APIRouter, Depends, status
from pydantic import BaseModel
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy import select
import traceback

from models import models
from db import database, db_models
from security import security
from core.responses import success_response, error_response

router = APIRouter()

class Deposit(BaseModel):
    user_id: int
    amount: float

class WebhookPayload(BaseModel):
    telegram_id: int
    amount: float

@router.post("/deposit")
async def deposit_balance(
    deposit: Deposit,
    db: AsyncSession = Depends(database.get_db),
    _ = Depends(security.verify_service_token)
):
    try:
        result = await db.execute(select(db_models.BotUser).filter(
            db_models.BotUser.telegram_id == deposit.user_id,
            db_models.BotUser.is_deleted == False
        ))
        user = result.scalars().first()
        if user is None:
            return error_response("Bot user not found", status_code=status.HTTP_404_NOT_FOUND)
        
        user.balance += deposit.amount
        await db.commit()
        await db.refresh(user)
        return success_response({"message": "Balance updated successfully", "new_balance": user.balance})
    except Exception as e:
        return error_response(str(e), status_code=status.HTTP_500_INTERNAL_SERVER_ERROR)

@router.post("/webhook")
async def payment_webhook(payload: WebhookPayload, db: AsyncSession = Depends(database.get_db)):
    # In a real application, this would be a secured endpoint
    # that verifies the webhook signature from the payment provider.
    try:
        print("Received webhook:", payload)
        result = await db.execute(select(db_models.BotUser).filter(
            db_models.BotUser.telegram_id == payload.telegram_id,
            db_models.BotUser.is_deleted == False
        ))
        user = result.scalars().first()
        if user is None:
            return error_response("Bot user not found", status_code=status.HTTP_404_NOT_FOUND)
        
        user.balance += payload.amount
        await db.commit()
        await db.refresh(user)
        return success_response({"status": "success", "message": "Balance updated via webhook"})
    except Exception as e:
        traceback.print_exc()
        return error_response(str(e), status_code=status.HTTP_500_INTERNAL_SERVER_ERROR)