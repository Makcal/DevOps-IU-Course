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

app.include_router(root_router)
app.include_router(health_router)


LOGGING_CONFIG = {
    "version": 1,
    "disable_existing_loggers": False,
    "formatters": {
        "json": {
            "()": "pythonjsonlogger.jsonlogger.JsonFormatter",
            "fmt": "%(asctime)s %(levelname)s %(name)s %(message)s"
        }
    },
    "handlers": {
        "default": {
            "class": "logging.StreamHandler",
            "formatter": "json",
            "level": "INFO"
        }
    },
    "root": {
        "handlers": ["default"],
        "level": "INFO"
    }
}


if __name__ == "__main__":
    uvicorn.run(app, host=host, port=port, log_config=LOGGING_CONFIG)
