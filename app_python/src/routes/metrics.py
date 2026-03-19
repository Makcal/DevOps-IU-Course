__all__ = ["router"]

from fastapi import APIRouter
from prometheus_client import generate_latest


router = APIRouter()


@router.get("/metrics")
async def metrics_endpoint():
    return generate_latest()
