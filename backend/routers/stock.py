from typing import List

from fastapi import APIRouter, Depends
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy import select

from models import models
from db import database, db_models
from security import security
from core.responses import success_response, error_response

router = APIRouter()

@router.get("/movements", response_model=List[models.StockMovement])
async def read_stock_movements(db: AsyncSession = Depends(database.get_db), current_user: models.User = Depends(security.get_current_active_user)):
    try:
        result = await db.execute(select(db_models.StockMovement).order_by(db_models.StockMovement.created_at.desc()))
        movements = result.scalars().all()
        return success_response([models.StockMovement.model_validate(m).model_dump() for m in movements])
    except Exception as e:
        return error_response(str(e))
