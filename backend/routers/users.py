
from fastapi import APIRouter, Depends, HTTPException

from models import models
from db import database
from security import security

router = APIRouter()

@router.post("/register", response_model=models.BotUser, status_code=201)
async def register_bot_user(
    user: models.BotUserCreate,
    _ = Depends(security.verify_service_token)
):
    # Check if user already exists
    for existing_user in database.DB["bot_users"].values():
        if existing_user["telegram_id"] == user.telegram_id:
            return existing_user

    new_id = database.get_next_id("bot_users")
    new_user = models.BotUser(
        id=new_id, 
        telegram_id=user.telegram_id,
        balance=0
    ).model_dump()
    database.DB["bot_users"][new_id] = new_user
    return new_user

@router.get("/{user_id}", response_model=models.BotUser)
async def read_bot_user(
    user_id: int,
    _ = Depends(security.verify_service_token)
):
    user = database.DB["bot_users"].get(user_id)
    if user is None:
        raise HTTPException(status_code=404, detail="Bot user not found")
    return user
