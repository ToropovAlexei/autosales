
from datetime import datetime, timedelta, timezone
from typing import Optional

from fastapi import Depends, HTTPException, status, Security
from fastapi.security import OAuth2PasswordBearer, APIKeyHeader
from jose import JWTError, jwt
from passlib.context import CryptContext

from db.database import DB
from models import models

# Configuration
SECRET_KEY = "a_very_secret_key_that_should_be_in_env_vars"
ALGORITHM = "HS256"
ACCESS_TOKEN_EXPIRE_MINUTES = 30
SERVICE_API_KEY = "a_very_secret_service_key"

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
        expire = datetime.now(timezone.utc) + timedelta(minutes=15)
    to_encode.update({"exp": expire})
    encoded_jwt = jwt.encode(to_encode, SECRET_KEY, algorithm=ALGORITHM)
    return encoded_jwt

# User lookup
def get_user(email: str):
    for user in DB["users"].values():
        if user["email"] == email:
            return user
    return None

# Dependency to get current user
from datetime import datetime, timedelta, timezone
from typing import Optional

from fastapi import Depends, HTTPException, status, Security
from fastapi.security import OAuth2PasswordBearer, APIKeyHeader
from jose import JWTError, jwt
from passlib.context import CryptContext

from db.database import DB
from models import models

# Configuration
SECRET_KEY = "a_very_secret_key_that_should_be_in_env_vars"
ALGORITHM = "HS256"
ACCESS_TOKEN_EXPIRE_MINUTES = 30
SERVICE_API_KEY = "a_very_secret_service_key"

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
        expire = datetime.now(timezone.utc) + timedelta(minutes=15)
    to_encode.update({"exp": expire})
    encoded_jwt = jwt.encode(to_encode, SECRET_KEY, algorithm=ALGORITHM)
    return encoded_jwt

# User lookup
def get_user(email: str):
    for user in DB["users"].values():
        if user["email"] == email:
            return user
    return None

# Dependency to get current user
async def get_current_active_user(token: str = Depends(oauth2_scheme)) -> models.User:
    credentials_exception = HTTPException(
        status_code=status.HTTP_401_UNAUTHORIZED,
        detail="Could not validate credentials",
        headers={"WWW-Authenticate": "Bearer"},
    )
    try:
        payload = jwt.decode(token, SECRET_KEY, algorithms=[ALGORITHM])
        email: str = payload.get("sub")
        if email is None:
            raise credentials_exception
        token_data = models.TokenData(email=email)
    except JWTError:
        raise credentials_exception
    
    user_data = get_user(email=token_data.email)
    if user_data is None:
        raise credentials_exception
    
    user = models.User(**user_data)
    if not user.is_active:
        raise HTTPException(status_code=400, detail="Inactive user")
    return user

# Dependency for service token
async def verify_service_token(x_api_key: str = Security(api_key_header)):
    if x_api_key != SERVICE_API_KEY:
        raise HTTPException(
            status_code=status.HTTP_403_FORBIDDEN,
            detail="Invalid service token",
        )


# Dependency for service token
async def verify_service_token(x_api_key: str = Security(api_key_header)):
    if x_api_key != SERVICE_API_KEY:
        raise HTTPException(
            status_code=status.HTTP_403_FORBIDDEN,
            detail="Invalid service token",
        )
