# Stage 1: Build stage using Rust official image
FROM rust:alpine AS builder

WORKDIR /usr/src/astrea-sda-api
RUN apk add --no-cache build-base musl-dev pkgconfig openssl-dev openssl-libs-static curl

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY tests ./tests

RUN cargo build --release

# Stage 2: Minimal runtime container image
FROM alpine:3.20

WORKDIR /app
RUN apk add --no-cache ca-certificates tzdata

COPY --from=builder /usr/src/astrea-sda-api/target/release/astrea-sda-api /app/astrea-sda-api

ENV HOST=0.0.0.0
ENV PORT=3000
EXPOSE 3000

CMD ["/app/astrea-sda-api"]
