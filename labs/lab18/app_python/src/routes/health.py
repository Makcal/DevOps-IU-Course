__all__ = ["router"]

from typing import Any
from fastapi import APIRouter

from src.statistics import get_runtime_info


router = APIRouter()


@router.get("/health")
async def health() -> dict[str, Any]:
    runtime_info = get_runtime_info()
    return {
        "status": "healthy",
        "timestamp": runtime_info["current_time"],
        "uptime_seconds": runtime_info["uptime_seconds"],
    }
