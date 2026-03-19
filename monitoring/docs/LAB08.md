# LAB08 --- Metrics & Monitoring with Prometheus

## 1. Architecture

Application exposes /metrics → Prometheus scrapes → Grafana visualizes.

## 2. Application Instrumentation

Implemented: - Counter: http_requests_total - Gauge:
http_requests_in_progress - Histogram: http_request_duration_seconds

Labels used: method, endpoint, status.

## 3. Prometheus Configuration

-   Scrape interval: 15s
-   Targets:
    -   prometheus:9090
    -   app-python:8000
    -   loki:3100
    -   grafana:3000
-   Retention: 15d / 10GB

## 4. Dashboard

Panels: - Request Rate → rate(http_requests_total\[5m\]) - Error Rate →
rate(http_requests_total{status=\~"5.."}\[5m\]) - p95 latency →
histogram_quantile - Heatmap → request duration buckets - Active
requests → gauge - Status distribution → sum by(status)

## 5. PromQL Examples

-   rate(http_requests_total\[5m\])
-   sum(rate(http_requests_total\[5m\]))
-   histogram_quantile(0.95,
    rate(http_request_duration_seconds_bucket\[5m\]))
-   up == 0
-   sum by(status)(rate(http_requests_total\[5m\]))

## 6. Production Setup

-   Health checks enabled
-   Resource limits configured
-   Persistent volumes used
-   Retention configured

## 7. Testing

-   curl /metrics works
-   Prometheus targets UP
-   Grafana dashboards show live data

![logs](./metrics_logs.png)

## 8. Challenges

-   Metrics not appearing → fixed endpoint path
-   Wrong labels → normalized endpoints
-   Prometheus DOWN → fixed network/service name
