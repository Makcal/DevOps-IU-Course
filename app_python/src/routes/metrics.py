__all__ = ["router"]

from fastapi import APIRouter
from fastapi.responses import PlainTextResponse
from prometheus_client import generate_latest


router = APIRouter()


@router.get("/metrics", response_class=PlainTextResponse)
async def metrics_endpoint():
    return generate_latest()
