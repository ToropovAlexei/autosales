from fastapi import APIRouter, Depends, status
from pydantic import BaseModel
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy import select
import datetime

from models import models
from db import database, db_models
from security import security
from core.responses import success_response, error_response

router = APIRouter()

class DepositRequest(BaseModel):
    user_id: int # This is the telegram_id
    amount: float

# THIS IS A TEST ENDPOINT AND SHOULD BE REMOVED IN PRODUCTION
# In a real application, you would have a proper payment provider integration
# and a webhook to confirm the payment.
@router.post("/deposit")
async def deposit_balance(
    deposit: DepositRequest,
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
        
        transaction = db_models.Transaction(
            user_id=user.id,
            type=models.TransactionType.DEPOSIT,
            amount=deposit.amount,
            description="Test deposit",
            created_at=datetime.datetime.utcnow()
        )
        db.add(transaction)
        await db.commit()
        return success_response({"message": "Balance updated successfully"})
    except Exception as e:
        return error_response(str(e), status_code=status.HTTP_500_INTERNAL_SERVER_ERROR)

class WebhookPayload(BaseModel):
    user_id: int # This is the telegram_id
    amount: float

@router.post("/webhook")
async def payment_webhook(payload: WebhookPayload, db: AsyncSession = Depends(database.get_db)):
    # In a real application, this would be a secured endpoint
    # that verifies the webhook signature from the payment provider.
    try:
        result = await db.execute(select(db_models.BotUser).filter(
            db_models.BotUser.telegram_id == payload.user_id,
            db_models.BotUser.is_deleted == False
        ))
        user = result.scalars().first()
        if user is None:
            # In a real app, you might want to create the user if they don't exist
            # or handle this case differently.
            return error_response("Bot user not found", status_code=status.HTTP_404_NOT_FOUND)
        
        transaction = db_models.Transaction(
            user_id=user.id,
            type=models.TransactionType.DEPOSIT,
            amount=payload.amount,
            description="Deposit via webhook",
            created_at=datetime.datetime.utcnow()
        )
        db.add(transaction)
        await db.commit()
        return success_response({"message": "Webhook received and balance updated"})
    except Exception as e:
        return error_response(str(e), status_code=status.HTTP_500_INTERNAL_SERVER_ERROR)
