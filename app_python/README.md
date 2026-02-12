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

---

## Prerequisites

- **Python:** 3.11 or newer
- **Docker:** 25+
- **pip:** Latest version recommended

---

## Installation (Local)

```bash
python -m venv venv
source venv/bin/activate
pip install -r requirements.txt
```

---

## Running the Application (Local)

```bash
python app.py
```

---

## Docker Usage

### Build Image Locally
```bash
docker build -t devops-info-service .
```

### Run Container
```bash
docker run -p 5000:5000 devops-info-service
```

### Pull from Docker Hub
```bash
docker pull <dockerhub-username>/devops-info-service
docker run -p 5000:5000 <dockerhub-username>/devops-info-service
```

---

## Running tests

```bash
pip install -r requirements-dev.txt
pytest --cov=.
```

---

## API Endpoints

- `GET /` – Service and system information
- `GET /health` – Health check endpoint

---

## Configuration

| Variable | Default | Description |
|--------|---------|-------------|
| `HOST` | `0.0.0.0` | Interface to bind |
| `PORT` | `5000` | Listening port |
| `DEBUG` | `false` | Debug mode |

---

## License

Educational use only (DevOps course).
