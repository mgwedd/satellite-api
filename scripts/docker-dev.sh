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
echo "   - API Dev Endpoint      : http://localhost:8080"
echo "   - Nginx Proxy Endpoint  : http://localhost:8000"
echo "   - PostgreSQL Port       : localhost:5432"
echo "   - Redis Port            : localhost:6379"
echo "=========================================================="
echo ""

docker compose -f docker-compose.dev.yml up --build "$@"
