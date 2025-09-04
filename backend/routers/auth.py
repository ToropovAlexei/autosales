from datetime import timedelta

from fastapi import APIRouter, Depends, status
from fastapi.security import OAuth2PasswordRequestForm
from sqlalchemy.ext.asyncio import AsyncSession

from config import settings
from models import models
from security import security
from db import database
from core.responses import success_response, error_response

router = APIRouter()

@router.post("/login")
async def login_for_access_token(form_data: OAuth2PasswordRequestForm = Depends(), db: AsyncSession = Depends(database.get_db)):
    user = await security.get_user(db, form_data.username)
    if not user or not security.verify_password(form_data.password, user.hashed_password):
        return error_response(
            "Incorrect username or password",
            status_code=status.HTTP_401_UNAUTHORIZED
        )
    access_token_expires = timedelta(minutes=settings.ACCESS_TOKEN_EXPIRE_MINUTES)
    access_token = security.create_access_token(
        data={"sub": user.email, "role": user.role}, expires_delta=access_token_expires
    )
    token_data = models.Token(access_token=access_token, token_type="bearer")
    return success_response(token_data.model_dump())