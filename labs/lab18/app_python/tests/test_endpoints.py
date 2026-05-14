from httpx import Response
from fastapi.testclient import TestClient

from main import app

client = TestClient(app)


def test_root_structure():
    response: Response = client.get("/")
    json: dict = response.json()
    FIELDS = {"service", "system", "runtime", "request", "endpoints"}
    assert json.keys() == FIELDS
    # rest details must be checked via type system


def test_healthcheck():
    response: Response = client.get("/health")
    json: dict = response.json()
    FIELDS = {"status", "timestamp", "uptime_seconds"}
    assert json.keys() == FIELDS
    assert json["status"] == "healthy"
    # rest details must be checked via type system
