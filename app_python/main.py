import os
import logging
import time

from fastapi import FastAPI, Request
import uvicorn

from src.routes.root import router as root_router
from src.routes.health import router as health_router

host = os.getenv("HOST", "0.0.0.0")
port = int(os.getenv("PORT", 5000))
debug = os.getenv("DEBUG", "False").lower() == "true"

app = FastAPI(debug=debug)

logger = logging.getLogger("app")
logger.setLevel(logging.INFO)

@app.middleware("http")
async def log_requests(request: Request, call_next):
    start = time.time()

    response = await call_next(request)

    duration = time.time() - start

    logger.info(
        "request",
        extra={
            "method": request.method,
            "path": request.url.path,
            "status_code": response.status_code,
            "client_ip": request.client.host,
            "duration": duration
        }
    )

    return response


app.include_router(root_router)
app.include_router(health_router)


if __name__ == "__main__":
    uvicorn.run(app, host=host, port=port)
