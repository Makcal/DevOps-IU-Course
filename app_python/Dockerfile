FROM python:3.13-slim AS builder

WORKDIR /build
COPY requirements.txt .

RUN ["pip", "install", "-r", "requirements.txt", "--user", "--no-cache-dir"]

# =========================
FROM python:3.13-slim AS main

WORKDIR /app
RUN groupadd groupapp && useradd -r -m -g groupapp userapp

COPY --from=builder /root/.local /home/userapp/.local
COPY . .

RUN mkdir /data && chown userapp:groupapp /data

USER userapp
EXPOSE 5000
CMD ["python", "main.py"]
