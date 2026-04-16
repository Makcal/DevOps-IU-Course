__all__ = ["router"]

from typing import Any
from fastapi import Request, APIRouter

from src.statistics import (
    get_service_info,
    get_system_info,
    get_runtime_info,
    get_request_info,
    get_endpoints_info,
)


router = APIRouter()


@router.get("/")
async def root(request: Request) -> dict[str, Any]:
    with open('/data/visits', 'a+') as read:
        read.seek(0)
        n = int(read.read() or '0')
    n += 1
    with open('/data/visits', 'w') as write:
        write.write(str(n))

    return {
        "service": get_service_info(),
        "system": get_system_info(),
        "runtime": get_runtime_info(),
        "request": get_request_info(request),
        "endpoints": get_endpoints_info(),
    }
