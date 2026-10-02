# ==============================================================================
# Astrea SDA API - Build & Container Management Makefile
# ==============================================================================

.PHONY: help build dev-hot dev prod test clean build-dev-hot build-dev build-prod

help:
	@echo "Astrea SDA API Build & Container Targets:"
	@echo "  make dev-hot    (or make build dev-hot) - Docker DNS/Nginx stack with local host app hot-reload"
	@echo "  make dev        (or make build dev)     - Fully containerized local dev stack (cargo-watch in Docker)"
	@echo "  make prod       (or make build prod)    - Build optimized production container image (with build tests)"
	@echo "  make test                               - Run full workspace offline test suite"
	@echo "  make clean                              - Stop all Docker containers and clean build cache"

# Default 'make build' builds production image unless followed by target (dev-hot / dev / prod)
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

build-dev-hot: dev-hot
build-dev: dev
build-prod: prod

test:
	cargo test --offline

clean:
	-docker compose down -v
	-docker compose -f docker-compose.dev.yml down -v
	-docker compose -f docker-compose.dev-hot.yml down -v
	cargo clean
