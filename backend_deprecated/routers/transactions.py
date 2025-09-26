from typing import List

from fastapi import APIRouter, Depends
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy import select

from models import models
from db import database, db_models
from security import security
from core.responses import success_response, error_response

router = APIRouter()

@router.get("", response_model=List[models.Transaction])
async def read_transactions(db: AsyncSession = Depends(database.get_db), current_user: models.User = Depends(security.get_current_active_user)):
    try:
        result = await db.execute(select(db_models.Transaction).order_by(db_models.Transaction.created_at.desc()))
        transactions = result.scalars().all()
        return success_response([models.Transaction.model_validate(t).model_dump() for t in transactions])
    except Exception as e:
        return error_response(str(e))
