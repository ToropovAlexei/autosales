
from fastapi import APIRouter, Depends, HTTPException
from pydantic import BaseModel

from models import models
from db import database
from security import security

router = APIRouter()

class Deposit(BaseModel):
    user_id: int
    amount: float

@router.get("/users/{user_id}/balance", response_model=float)
async def get_balance(
    user_id: int,
    _ = Depends(security.verify_service_token)
):
    user = database.DB["bot_users"].get(user_id)
    if user is None:
        raise HTTPException(status_code=404, detail="Bot user not found")
    return user["balance"]

@router.post("/deposit")
async def deposit_balance(
    deposit: Deposit,
    _ = Depends(security.verify_service_token)
):
    user = database.DB["bot_users"].get(deposit.user_id)
    if user is None:
        raise HTTPException(status_code=404, detail="Bot user not found")
    
    user["balance"] += deposit.amount
    return {"message": "Balance updated successfully", "new_balance": user["balance"]}

@router.post("/webhook")
async def payment_webhook(payload: dict):
    # In a real application, this would be a secured endpoint
    # that verifies the webhook signature from the payment provider.
    print("Received webhook:", payload)
    # Here you would parse the payload and update the user's balance.
    return {"status": "received"}
