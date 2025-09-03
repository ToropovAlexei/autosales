
from typing import List

from fastapi import APIRouter, Depends, HTTPException

from models import models
from db import database
from security import security

router = APIRouter()

@router.get("", response_model=List[models.Category])
async def read_categories():
    return list(database.DB["categories"].values())

@router.post("", response_model=models.Category, status_code=201)
async def create_category(
    category: models.CategoryCreate,
    current_user: models.User = Depends(security.get_current_active_user)
):
    new_id = database.get_next_id("categories")
    new_category = models.Category(id=new_id, name=category.name).model_dump()
    database.DB["categories"][new_id] = new_category
    return new_category

@router.get("/{category_id}", response_model=models.Category)
async def read_category(category_id: int):
    category = database.DB["categories"].get(category_id)
    if category is None:
        raise HTTPException(status_code=404, detail="Category not found")
    return category

@router.put("/{category_id}", response_model=models.Category)
async def update_category(
    category_id: int,
    category_update: models.CategoryCreate,
    current_user: models.User = Depends(security.get_current_active_user)
):
    if category_id not in database.DB["categories"]:
        raise HTTPException(status_code=404, detail="Category not found")
    
    updated_category = models.Category(id=category_id, name=category_update.name).model_dump()
    database.DB["categories"][category_id] = updated_category
    return updated_category

@router.delete("/{category_id}", status_code=204)
async def delete_category(
    category_id: int,
    current_user: models.User = Depends(security.get_current_active_user)
):
    if category_id not in database.DB["categories"]:
        raise HTTPException(status_code=404, detail="Category not found")
    
    del database.DB["categories"][category_id]
    return
