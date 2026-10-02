#!/usr/bin/env bash
set -e

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )/.." && pwd )"
cd "$DIR"

echo "=========================================================="
echo "🔥 Starting Local Host Hot-Reload Stack (dev-hot)..."
echo "=========================================================="

# Ensure local RSA keypair and TLS certificates exist
./scripts/setup-keys.sh

echo ""
echo "🐳 Launching Docker services (Postgres + Redis + Caddy)..."
docker compose -f docker-compose.dev-hot.yml up -d

echo ""
echo "=========================================================="
echo "✅ Docker Dev Stack active (Postgres + Redis + Caddy):"
echo "   ⚡ Caddy Zero-Config OSS Gateway :"
echo "      - HTTPS (Automatic TLS)       : https://sda.localtest.me:8443 (or https://localhost:8443)"
echo "      - Interactive Swagger UI      : https://sda.localtest.me:8443/ (or /swagger-ui/)"
echo "      - ReDoc API Reference         : https://sda.localtest.me:8443/docs"
echo "      - Direct API Endpoints        : https://sda.localtest.me:8443/v1/... (or /api/v1/...)"
echo "      - HTTP Local Gateway          : http://sda.localtest.me:8888 (or http://localhost:8888)"
echo "   🗄️  PostgreSQL                    : localhost:5432"
echo "   💾 Redis                         : localhost:6379"
echo "   (Sub-second hot-reload via host cargo-watch, zero /etc/hosts edits required)"
echo "=========================================================="
echo "🚀 Starting host process with cargo watch (press Ctrl+C to stop)..."

echo ""

export POSTGRES_URI="postgres://astrea:astreadbpass@127.0.0.1:5432/astrea_sda"
export REDIS_URL="redis://127.0.0.1:6379"
export HOST="0.0.0.0"
export PORT="8080"
export ENABLE_DISCOVERY_PIPELINE="true"

if command -v cargo-watch &> /dev/null; then
  cargo watch -x run
else
  cargo run
fi
