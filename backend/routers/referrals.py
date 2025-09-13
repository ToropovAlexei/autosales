from typing import List
from fastapi import APIRouter, Depends, status
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy import select, func
from sqlalchemy.sql.functions import coalesce

from core.responses import success_response, error_response
from models import models
from db import database, db_models
from security import security

router = APIRouter()

@router.post("")
async def create_referral_bot(bot: models.ReferralBotCreate, db: AsyncSession = Depends(database.get_db), _ = Depends(security.verify_service_token)):
    user_result = await db.execute(select(db_models.BotUser).filter(db_models.BotUser.telegram_id == bot.owner_id))
    owner = user_result.scalars().first()
    if not owner:
        return error_response("Referral owner (user) not found.", status_code=status.HTTP_404_NOT_FOUND)

    seller_result = await db.execute(select(db_models.User).filter(db_models.User.id == bot.seller_id))
    if not seller_result.scalars().first():
        return error_response("Seller not found.", status_code=status.HTTP_404_NOT_FOUND)

    existing_bot_result = await db.execute(select(db_models.ReferralBot).filter(db_models.ReferralBot.bot_token == bot.bot_token))
    if existing_bot_result.scalars().first():
        return error_response("Bot with this token already exists.", status_code=status.HTTP_400_BAD_REQUEST)

    db_bot = db_models.ReferralBot(
        owner_id=owner.id,
        seller_id=bot.seller_id,
        bot_token=bot.bot_token
    )
    db.add(db_bot)
    await db.commit()
    await db.refresh(db_bot)
    return success_response(models.ReferralBot.model_validate(db_bot).model_dump())


@router.get("")
async def read_referral_bots(
    skip: int = 0,
    limit: int = 100,
    db: AsyncSession = Depends(database.get_db),
    _ = Depends(security.verify_service_token),
):
    result = await db.execute(select(db_models.ReferralBot).offset(skip).limit(limit))
    bots = result.scalars().all()
    return success_response(bots)

@router.get("/admin-list", response_model=List[models.ReferralBotAdminInfo])
async def read_referral_bots_admin(
    db: AsyncSession = Depends(database.get_db),
    current_user: models.User = Depends(security.get_current_active_user),
):
    if current_user.role not in [models.UserRole.admin, models.UserRole.seller]:
        return error_response("Not enough permissions", status_code=status.HTTP_403_FORBIDDEN)

    stmt = (
        select(
            db_models.ReferralBot.id,
            db_models.ReferralBot.is_active,
            db_models.ReferralBot.created_at,
            db_models.BotUser.telegram_id.label("owner_telegram_id"),
            coalesce(func.sum(db_models.RefTransaction.amount), 0).label("turnover"),
            coalesce(func.sum(db_models.RefTransaction.ref_share), 0).label("accruals")
        )
        .select_from(db_models.ReferralBot)
        .join(db_models.BotUser, db_models.ReferralBot.owner_id == db_models.BotUser.id)
        .outerjoin(db_models.RefTransaction, db_models.ReferralBot.owner_id == db_models.RefTransaction.ref_owner_id)
        .where(db_models.ReferralBot.seller_id == current_user.id)
        .group_by(db_models.ReferralBot.id, db_models.BotUser.telegram_id)
    )

    result = await db.execute(stmt)
    bots_data = result.mappings().all()
    
    return success_response(bots_data)

@router.put("/{bot_id}")
async def toggle_referral_bot_status(
    bot_id: int,
    db: AsyncSession = Depends(database.get_db),
    current_user: models.User = Depends(security.get_current_active_user),
):
    if current_user.role not in [models.UserRole.admin, models.UserRole.seller]:
        return error_response("Not enough permissions", status_code=status.HTTP_403_FORBIDDEN)

    result = await db.execute(select(db_models.ReferralBot).filter(db_models.ReferralBot.id == bot_id))
    bot = result.scalars().first()

    if not bot:
        return error_response("Referral bot not found", status_code=status.HTTP_404_NOT_FOUND)
    
    if bot.seller_id != current_user.id:
        return error_response("You are not the owner of this referral bot", status_code=status.HTTP_403_FORBIDDEN)

    bot.is_active = not bot.is_active
    await db.commit()
    await db.refresh(bot)
    
    return success_response(models.ReferralBot.model_validate(bot).model_dump())