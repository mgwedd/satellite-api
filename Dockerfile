# Stage 1: Build stage using Rust official image
FROM rust:1.80-alpine as builder

WORKDIR /usr/src/satellite-api
RUN apk add --no-crate-build-pkg build-base musl-dev pkgconfig openssl-dev

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY tests ./tests

RUN cargo build --release

# Stage 2: Minimal runtime container image
FROM alpine:3.20

WORKDIR /app
RUN apk add --no-cache ca-certificates tzdata

COPY --from=builder /usr/src/satellite-api/target/release/satellite-api /app/satellite-api

ENV HOST=0.0.0.0
ENV PORT=3000
EXPOSE 3000

CMD ["/app/satellite-api"]
