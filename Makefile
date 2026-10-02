# ==============================================================================
# Astrea Space Domain Awareness (SDA) API - Developer Makefile
# ==============================================================================

# User-overridable variables for token generation
USER ?= admin_user
ROLE ?= admin

.PHONY: help build dev-hot dev prod check fmt lint test test-watch keys jwt ui openapi sdk sdk-all sdk-ts sdk-typescript sdk-py sdk-python sdk-go sdk-java sdk-rust hooks install install-dev logs stop clean build-dev-hot build-dev build-prod

help:
	@echo "=============================================================================="
	@echo "🛰️ Astrea SDA API Developer Makefile Options:"
	@echo "=============================================================================="
	@echo "BUILD & CONTAINER TARGETS:"
	@echo "  make dev-hot      (or make build dev-hot) - Docker DNS/Nginx stack with local host app hot-reload"
	@echo "  make dev          (or make build dev)     - Fully containerized local dev stack (cargo-watch in Docker)"
	@echo "  make prod         (or make build prod)    - Build optimized production container image (with build tests)"
	@echo "  make install                              - Install binary locally into ~/.cargo/bin"
	@echo "  make install-dev                          - Full developer setup (cargo-watch, fern, keys, hooks, binary)"
	@echo ""
	@echo "CODE QUALITY & TESTING:"
	@echo "  make check                                - Run fast offline compilation check"
	@echo "  make fmt                                  - Format code using cargo fmt"
	@echo "  make lint                                 - Run cargo clippy lints with strict warning checks"
	@echo "  make test                                 - Run full workspace offline test suite"
	@echo "  make test-watch                           - Continuously run test suite on file changes"
	@echo ""
	@echo "KEYS & AUTHENTICATION:"
	@echo "  make keys                                 - Generate local RSA 2048 keypair & dev TLS certs"
	@echo "  make jwt [USER=...] [ROLE=...]            - Generate signed RS256 Bearer JWT test token"
	@echo ""
	@echo "API DOCS & CLIENT SDK GENERATION:"
	@echo "  make ui                                   - Launch interactive OpenAPI UI documentation selector"
	@echo "  make openapi                              - Verify OpenAPI 3.0 schema contract stability"
	@echo "  make sdk                                  - Generate all SDKs (TypeScript, Python, Go, Java, Rust)"
	@echo "  make sdk-ts / make sdk-typescript         - Generate TypeScript / Node.js SDK"
	@echo "  make sdk-python / make sdk-py             - Generate Python SDK"
	@echo "  make sdk-go                               - Generate Go SDK"
	@echo "  make sdk-java                             - Generate Java SDK"
	@echo "  make sdk-rust                             - Generate Rust SDK"
	@echo ""
	@echo "DOCKER OPERATIONS & HOUSEKEEPING:"
	@echo "  make hooks                                - Install git pre-commit quality hooks"
	@echo "  make logs                                 - Stream Docker container logs"
	@echo "  make stop                                 - Stop all running Docker containers"
	@echo "  make clean                                - Stop Docker stacks and clean build artifacts"
	@echo "=============================================================================="

# Default 'make build' builds production image unless sub-target (dev-hot / dev / prod) is specified
build:
	@if [ "$(filter-out build,$(MAKECMDGOALS))" = "" ]; then \
		./scripts/docker-build.sh; \
	fi

dev-hot:
	@./scripts/docker-dev-hot.sh

dev:
	@./scripts/docker-dev.sh

prod:
	@./scripts/docker-build.sh

install:
	@echo "Installing astrea-sda-api binary locally..."
	cargo install --path .
	@echo "✅ Installed astrea-sda-api binary into ~/.cargo/bin/astrea-sda-api"

install-dev: keys hooks install
	@echo "=========================================================="
	@echo "🛠️ Installing Local Developer Tooling & Dependencies..."
	@echo "=========================================================="
	@if ! command -v cargo-watch &> /dev/null; then \
		echo "📦 Installing cargo-watch for hot reloading..."; \
		cargo install cargo-watch; \
	else \
		echo "✅ cargo-watch is already installed"; \
	fi
	@if ! command -v fern &> /dev/null && command -v npm &> /dev/null; then \
		echo "📦 Installing fern-api CLI for SDK generation..."; \
		npm install -g fern-api || true; \
	fi
	@echo "=========================================================="
	@echo "✅ Developer environment fully initialized!"
	@echo "=========================================================="

build-dev-hot: dev-hot
build-dev: dev
build-prod: prod

# Code Quality & Testing Targets
check:
	cargo check --offline

fmt:
	cargo fmt

lint:
	cargo clippy --all-targets --all-features --offline -- -D warnings

test:
	cargo test --offline

test-watch:
	cargo watch -x 'test --offline'

# Keys & Auth Targets
keys:
	@./scripts/setup-keys.sh

jwt: keys
	@./scripts/make-jwt.sh $(USER) $(ROLE)

# API Explorer & SDK Targets
ui:
	@./scripts/dev-ui.sh

openapi:
	cargo test --test openapi_contract_tests

sdk: sdk-all

sdk-all:
	@if command -v fern &> /dev/null; then \
		fern generate --group local; \
	else \
		npx fern generate --group local; \
	fi

sdk-ts: sdk-typescript
sdk-typescript:
	@if command -v fern &> /dev/null; then \
		fern generate --group ts; \
	else \
		npx fern generate --group ts; \
	fi

sdk-py: sdk-python
sdk-python:
	@if command -v fern &> /dev/null; then \
		fern generate --group python; \
	else \
		npx fern generate --group python; \
	fi

sdk-go:
	@if command -v fern &> /dev/null; then \
		fern generate --group go; \
	else \
		npx fern generate --group go; \
	fi

sdk-java:
	@if command -v fern &> /dev/null; then \
		fern generate --group java; \
	else \
		npx fern generate --group java; \
	fi

sdk-rust:
	@if command -v fern &> /dev/null; then \
		fern generate --group rust; \
	else \
		npx fern generate --group rust; \
	fi

# Operations & Housekeeping Targets
hooks:
	@./scripts/setup-hooks.sh

logs:
	docker compose logs -f

stop:
	-docker compose down
	-docker compose -f docker-compose.dev.yml down
	-docker compose -f docker-compose.dev-hot.yml down

clean: stop
	cargo clean
