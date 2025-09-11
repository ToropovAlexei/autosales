from typing import List
from fastapi import APIRouter, Depends, status
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy import select
from core.responses import success_response, error_response

from models import models
from db import database, db_models
from security import security

router = APIRouter()

@router.post("", response_model=models.ReferralBot)
async def create_referral_bot(bot: models.ReferralBotCreate, db: AsyncSession = Depends(database.get_db), _ = Depends(security.verify_service_token)):
    
    db_bot = db_models.ReferralBot(**bot.dict())
    db.add(db_bot)
    await db.commit()
    await db.refresh(db_bot)
    return db_bot


@router.get("", response_model=List[models.ReferralBot])
async def read_referral_bots(
    skip: int = 0,
    limit: int = 100,
    db: AsyncSession = Depends(database.get_db),
    current_user: models.User = Depends(security.get_current_active_user),
):
    if current_user.role != models.UserRole.admin and current_user.role != models.UserRole.seller:
        return error_response("Not enough permissions", status_code=status.HTTP_403_FORBIDDEN)
    
    result = await db.execute(select(db_models.ReferralBot).filter(db_models.ReferralBot.seller_id == current_user.id).offset(skip).limit(limit))
    bots = result.scalars().all()
    return success_response(bots)