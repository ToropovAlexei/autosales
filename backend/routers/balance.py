
from fastapi import APIRouter, Depends, HTTPException
from pydantic import BaseModel
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy import select

from models import models
from db import database, db_models
from security import security

router = APIRouter()

class Deposit(BaseModel):
    user_id: int
    amount: float


@router.post("/deposit")
async def deposit_balance(
    deposit: Deposit,
    db: AsyncSession = Depends(database.get_db),
    _ = Depends(security.verify_service_token)
):
    result = await db.execute(select(db_models.BotUser).filter(db_models.BotUser.telegram_id == deposit.user_id))
    user = result.scalars().first()
    if user is None:
        raise HTTPException(status_code=404, detail="Bot user not found")
    
    user.balance += deposit.amount
    await db.commit()
    await db.refresh(user)
    return {"message": "Balance updated successfully", "new_balance": user.balance}

@router.post("/webhook")
async def payment_webhook(payload: dict):
    # In a real application, this would be a secured endpoint
    # that verifies the webhook signature from the payment provider.
    print("Received webhook:", payload)
    # Here you would parse the payload and update the user's balance.
    return {"status": "received"}
