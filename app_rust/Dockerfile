FROM rust:alpine3.23 AS builder

WORKDIR /build
COPY . .
RUN cargo build --release

FROM alpine:3.23 AS main

RUN addgroup groupapp && adduser -S -G groupapp userapp
USER userapp

WORKDIR /app
COPY --from=builder /build/target/release/devops-info-service /app/service

EXPOSE 8080
CMD ["./service"]
