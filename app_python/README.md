# DevOps Info Service

A lightweight web service that exposes information about itself and the system it is running on.  
This service is the foundation for a larger DevOps monitoring and observability tool that will evolve throughout the course.

---

## Overview

The **DevOps Info Service** is a Python-based web application that provides:

- Service metadata (name, version, framework)
- Host and system information
- Runtime and uptime details
- Incoming request metadata
- A health check endpoint for monitoring and orchestration tools

The service exposes a simple HTTP API and is designed with production-ready best practices in mind, including configuration via environment variables, structured JSON responses, logging, and error handling.

---

## Prerequisites

- **Python:** 3.11 or newer  
- **pip:** Latest version recommended  
- **Virtual environment:** `venv` (recommended)

---

## Installation

Clone the repository, `cd` to `app_python` and set up a virtual environment:

```bash
python -m venv venv
source venv/bin/activate
pip install -r requirements.txt
```

---

## Running the Application

Start the service with default settings:

```bash
python app.py
```

Run with custom configuration using environment variables:

```bash
PORT=8080 python app.py
HOST=127.0.0.1 PORT=3000 python app.py
DEBUG=true python app.py
```

By default, the service listens on `0.0.0.0:5000`.

---

## API Endpoints

### `GET /` — Service & System Information

Returns detailed information about the service, host system, runtime, request, and available endpoints.

### `GET /health` — Health Check

Simple health endpoint intended for monitoring systems and Kubernetes probes.

**Example response:**

```json
{
  "status": "healthy",
  "timestamp": "2026-01-07T14:30:00.000Z",
  "uptime_seconds": 3600
}
```

Returns HTTP `200` when the service is healthy.

---

## Configuration

The application can be configured using environment variables:

| Variable | Default | Description |
|--------|---------|-------------|
| `HOST` | `0.0.0.0` | Interface to bind the server |
| `PORT` | `5000` | Port to listen on |
| `DEBUG` | `false` | Enable debug mode |

---

## Development Notes

- Follows PEP 8 Python style guidelines
- Uses structured JSON responses
- Centralized uptime tracking
- Designed for future containerization, CI/CD, and monitoring

---

## License

This project is intended for educational use as part of the DevOps course.
