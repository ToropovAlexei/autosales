from datetime import datetime, timedelta, timezone
from typing import Optional

from fastapi import Depends, HTTPException, status, Security
from fastapi.security import OAuth2PasswordBearer, APIKeyHeader
from jose import JWTError, jwt
from passlib.context import CryptContext
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy import select

from db import database, db_models
from models import models
from config import settings

# Configuration
oauth2_scheme = OAuth2PasswordBearer(tokenUrl="/api/auth/login")
api_key_header = APIKeyHeader(name="X-API-KEY", auto_error=False)

pwd_context = CryptContext(schemes=["bcrypt"], deprecated="auto")

# Password Hashing
def verify_password(plain_password, hashed_password):
    return pwd_context.verify(plain_password, hashed_password)

def get_password_hash(password):
    return pwd_context.hash(password)

# JWT Creation
def create_access_token(data: dict, expires_delta: Optional[timedelta] = None):
    to_encode = data.copy()
    if expires_delta:
        expire = datetime.now(timezone.utc) + expires_delta
    else:
        expire = datetime.now(timezone.utc) + timedelta(minutes=settings.ACCESS_TOKEN_EXPIRE_MINUTES)
    to_encode.update({"exp": expire})
    encoded_jwt = jwt.encode(to_encode, settings.SECRET_KEY, algorithm=settings.ALGORITHM)
    return encoded_jwt

# User lookup
async def get_user(db: AsyncSession, email: str):
    result = await db.execute(select(db_models.User).filter(db_models.User.email == email))
    return result.scalars().first()

# Dependency to get current user
async def get_current_active_user(token: str = Depends(oauth2_scheme), db: AsyncSession = Depends(database.get_db)) -> models.User:
    credentials_exception = HTTPException(
        status_code=status.HTTP_401_UNAUTHORIZED,
        detail="Could not validate credentials",
        headers={"WWW-Authenticate": "Bearer"},
    )
    try:
        payload = jwt.decode(token, settings.SECRET_KEY, algorithms=[settings.ALGORITHM])
        email: str = payload.get("sub")
        if email is None:
            raise credentials_exception
        token_data = models.TokenData(email=email)
    except JWTError:
        raise credentials_exception
    
    user = await get_user(db, email=token_data.email)
    if user is None:
        raise credentials_exception
    
    pydantic_user = models.User.from_orm(user)
    if not pydantic_user.is_active:
        raise HTTPException(status_code=400, detail="Inactive user")
    return pydantic_user

# Dependency for service token
async def verify_service_token(x_api_key: str = Security(api_key_header)):
    if x_api_key != settings.SERVICE_API_KEY:
        raise HTTPException(
            status_code=status.HTTP_403_FORBIDDEN,
            detail="Invalid service token",
        )