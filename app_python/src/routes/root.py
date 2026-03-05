__all__ = ["router"]


from typing import Any
from fastapi import Request, APIRouter

from src.statistics import *


router = APIRouter()


@router.get("/")
async def root(request: Request) -> dict[str, Any]:
    return {
        "service": get_service_info(),
        "system": get_system_info(),
        "runtime": get_runtime_info(),
        "request": get_request_info(request),
        "endpoints": get_endpoints_info(),
    }
