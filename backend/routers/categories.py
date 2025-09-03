

from typing import List

from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy import select

from models import models
from db import database, db_models
from security import security

router = APIRouter()

@router.get("", response_model=List[models.Category])
async def read_categories(db: AsyncSession = Depends(database.get_db)):
    result = await db.execute(select(db_models.Category))
    categories = result.scalars().all()
    return categories

@router.post("", response_model=models.Category, status_code=201)
async def create_category(
    category: models.CategoryCreate,
    db: AsyncSession = Depends(database.get_db),
    current_user: models.User = Depends(security.get_current_active_user)
):
    db_category = db_models.Category(name=category.name)
    db.add(db_category)
    await db.commit()
    await db.refresh(db_category)
    return db_category

@router.get("/{category_id}", response_model=models.Category)
async def read_category(category_id: int, db: AsyncSession = Depends(database.get_db)):
    result = await db.execute(select(db_models.Category).filter(db_models.Category.id == category_id))
    category = result.scalars().first()
    if category is None:
        raise HTTPException(status_code=404, detail="Category not found")
    return category

@router.put("/{category_id}", response_model=models.Category)
async def update_category(
    category_id: int,
    category_update: models.CategoryCreate,
    db: AsyncSession = Depends(database.get_db),
    current_user: models.User = Depends(security.get_current_active_user)
):
    result = await db.execute(select(db_models.Category).filter(db_models.Category.id == category_id))
    db_category = result.scalars().first()
    if db_category is None:
        raise HTTPException(status_code=404, detail="Category not found")
    
    db_category.name = category_update.name
    await db.commit()
    await db.refresh(db_category)
    return db_category

@router.delete("/{category_id}", status_code=204)
async def delete_category(
    category_id: int,
    db: AsyncSession = Depends(database.get_db),
    current_user: models.User = Depends(security.get_current_active_user)
):
    result = await db.execute(select(db_models.Category).filter(db_models.Category.id == category_id))
    db_category = result.scalars().first()
    if db_category is None:
        raise HTTPException(status_code=404, detail="Category not found")

    await db.delete(db_category)
    await db.commit()
    return

