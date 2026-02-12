__all__ = [
    "get_service_info",
    "get_system_info",
    "get_runtime_info",
    "get_request_info",
    "get_endpoints_info",
]

import platform
import socket
import os
from datetime import datetime
from typing import Any

from fastapi import Request


def get_service_info() -> dict[str, str]:
    return {
        "name": "devops-info-service",
        "version": "1.0.0",
        "description": "DevOps course info service",
        "framework": "FastAPI",
    }


def get_os_release() -> str | None:
    try:
        with open("/etc/os-release") as f:
            for line in f:
                if not line.startswith("PRETTY_NAME"):
                    continue
                value = line.partition("=")[2].strip().strip('"').strip("'")
                return value
        return None
    except FileNotFoundError:
        return None


def get_system_info() -> dict[str, Any]:
    return {
        "hostname": socket.gethostname(),
        "platform": platform.system(),
        "platform_version": get_os_release(),
        "architecture": platform.machine(),
        "cpu_count": os.cpu_count(),
        "python_version": platform.python_version(),
    }


start_time = datetime.now()


def get_uptime() -> dict[str, Any]:
    delta = datetime.now() - start_time
    seconds = int(delta.total_seconds())
    hours = seconds // 3600
    minutes = (seconds % 3600) // 60
    return {"seconds": seconds, "human": f"{hours} hours, {minutes} minutes"}


def get_runtime_info() -> dict[str, Any]:
    uptime = get_uptime()
    now = datetime.now().astimezone()
    return {
        "uptime_seconds": uptime["seconds"],
        "uptime_human": uptime["human"],
        "current_time": now.isoformat(),
        "timezone": now.tzinfo.tzname(now)
        if now.tzinfo is not None
        else "Unknown",
    }


def get_request_info(request: Request) -> dict[str, Any]:
    return {
        "client_ip": request.client.host
        if request.client is not None
        else "Unknown",
        "user_agent": request.headers.get("User-Agent"),
        "method": request.method,
        "path": request.url.path,
    }


def get_endpoints_info() -> list[dict[str, Any]]:
    return [
        {"path": "/", "method": "GET", "description": "Service information"},
        {"path": "/health", "method": "GET", "description": "Health check"},
    ]
