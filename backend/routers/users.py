

from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy import select

from models import models
from db import database, db_models
from security import security

router = APIRouter()

@router.post("/register", response_model=models.BotUser, status_code=201)
async def register_bot_user(
    user: models.BotUserCreate,
    db: AsyncSession = Depends(database.get_db),
    _ = Depends(security.verify_service_token)
):
    # Check if user already exists
    result = await db.execute(select(db_models.BotUser).filter(db_models.BotUser.telegram_id == user.telegram_id))
    existing_user = result.scalars().first()
    if existing_user:
        return existing_user

    db_user = db_models.BotUser(telegram_id=user.telegram_id, balance=0)
    db.add(db_user)
    await db.commit()
    await db.refresh(db_user)
    return db_user

@router.get("/{user_id}", response_model=models.BotUser)
async def read_bot_user(
    user_id: int,
    db: AsyncSession = Depends(database.get_db),
    _ = Depends(security.verify_service_token)
):
    result = await db.execute(select(db_models.BotUser).filter(db_models.BotUser.id == user_id))
    user = result.scalars().first()
    if user is None:
        raise HTTPException(status_code=404, detail="Bot user not found")
    return user

