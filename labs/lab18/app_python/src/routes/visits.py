__all__ = ["router"]

from fastapi import APIRouter


router = APIRouter()


@router.get("/visits")
async def visits() -> int:
    with open('/data/visits', 'a+') as read:
        read.seek(0)
        n = int(read.read() or '0')

    return n
