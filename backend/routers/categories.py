from typing import List

from fastapi import APIRouter, Depends, status
from fastapi.responses import JSONResponse
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy import select

from models import models
from db import database, db_models
from security import security
from core.responses import success_response, error_response

router = APIRouter()

@router.get("")
async def read_categories(db: AsyncSession = Depends(database.get_db)):
    try:
        result = await db.execute(select(db_models.Category))
        categories = result.scalars().all()
        return success_response([models.Category.model_validate(c).model_dump() for c in categories])
    except Exception as e:
        return error_response(str(e), status_code=status.HTTP_500_INTERNAL_SERVER_ERROR)

@router.post("", status_code=status.HTTP_201_CREATED)
async def create_category(
    category: models.CategoryCreate,
    db: AsyncSession = Depends(database.get_db),
    current_user: models.User = Depends(security.get_current_active_user)
):
    try:
        db_category = db_models.Category(name=category.name)
        db.add(db_category)
        await db.commit()
        await db.refresh(db_category)
        return success_response(models.Category.model_validate(db_category).model_dump(), status_code=status.HTTP_201_CREATED)
    except Exception as e:
        return error_response(str(e), status_code=status.HTTP_500_INTERNAL_SERVER_ERROR)

@router.get("/{category_id}")
async def read_category(category_id: int, db: AsyncSession = Depends(database.get_db)):
    try:
        result = await db.execute(select(db_models.Category).filter(db_models.Category.id == category_id))
        category = result.scalars().first()
        if category is None:
            return error_response("Category not found", status_code=status.HTTP_404_NOT_FOUND)
        return success_response(models.Category.model_validate(category).model_dump())
    except Exception as e:
        return error_response(str(e), status_code=status.HTTP_500_INTERNAL_SERVER_ERROR)

@router.put("/{category_id}")
async def update_category(
    category_id: int,
    category_update: models.CategoryCreate,
    db: AsyncSession = Depends(database.get_db),
    current_user: models.User = Depends(security.get_current_active_user)
):
    try:
        result = await db.execute(select(db_models.Category).filter(db_models.Category.id == category_id))
        db_category = result.scalars().first()
        if db_category is None:
            return error_response("Category not found", status_code=status.HTTP_404_NOT_FOUND)
        
        db_category.name = category_update.name
        await db.commit()
        await db.refresh(db_category)
        return success_response(models.Category.model_validate(db_category).model_dump())
    except Exception as e:
        return error_response(str(e), status_code=status.HTTP_500_INTERNAL_SERVER_ERROR)

@router.delete("/{category_id}", status_code=status.HTTP_204_NO_CONTENT)
async def delete_category(
    category_id: int,
    db: AsyncSession = Depends(database.get_db),
    current_user: models.User = Depends(security.get_current_active_user)
):
    try:
        result = await db.execute(select(db_models.Category).filter(db_models.Category.id == category_id))
        db_category = result.scalars().first()
        if db_category is None:
            return error_response("Category not found", status_code=status.HTTP_404_NOT_FOUND)

        await db.delete(db_category)
        await db.commit()
        return JSONResponse(status_code=status.HTTP_204_NO_CONTENT)
    except Exception as e:
        return error_response(str(e), status_code=status.HTTP_500_INTERNAL_SERVER_ERROR)