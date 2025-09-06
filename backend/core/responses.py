from typing import TypeVar, Generic, Optional, Any
from pydantic import BaseModel, Field
from fastapi.responses import JSONResponse
from fastapi import status
from fastapi.encoders import jsonable_encoder

T = TypeVar('T')

class ApiResponse(BaseModel, Generic[T]):
    success: bool
    data: Optional[T] = None
    error: Optional[str] = None

def success_response(data: Any, status_code: int = status.HTTP_200_OK) -> JSONResponse:
    return JSONResponse(
        status_code=status_code,
        content={
            "success": True,
            "data": jsonable_encoder(data),
            "error": None,
        },
    )

def error_response(error: str, status_code: int = status.HTTP_400_BAD_REQUEST) -> JSONResponse:
    return JSONResponse(
        status_code=status_code,
        content={
            "success": False,
            "data": None,
            "error": error,
        },
    )
