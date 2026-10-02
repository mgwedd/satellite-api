#!/usr/bin/env bash
set -e

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )/.." && pwd )"
cd "$DIR"

echo "=========================================================="
echo "🔥 Starting Containerized Hot-Reloading Local Dev Stack..."
echo "=========================================================="

# Ensure local RSA keypair exists
./scripts/setup-keys.sh

echo ""
echo "🚀 Launching Docker Compose Dev Stack (cargo-watch + Postgres + Redis + Nginx)..."
echo "   - Host hot-reload mount : . -> /app"
echo "   - Gateway (Nginx Proxy) : http://localhost:8888"
echo "   - API Dev Direct        : http://localhost:8880"
echo "   - PostgreSQL Port       : localhost:5433"
echo "   - Redis Port            : localhost:6380"
echo "=========================================================="
echo ""

docker compose -f docker-compose.dev.yml up --build "$@"
