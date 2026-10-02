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
echo "🚀 Launching Docker Compose Dev Stack (cargo-watch + Postgres + Redis + Caddy)..."
echo "   ⚡ Caddy Zero-Config OSS Gateway :"
echo "      - HTTPS (Automatic TLS)       : https://sda.localtest.me:8443 (or https://localhost:8443)"
echo "      - Interactive Swagger UI      : https://sda.localtest.me:8443/ (or /swagger-ui/)"
echo "      - ReDoc API Reference         : https://sda.localtest.me:8443/docs"
echo "      - Direct API Endpoints        : https://sda.localtest.me:8443/v1/... (or /api/v1/...)"
echo "      - HTTP Local Gateway          : http://sda.localtest.me:8888 (or http://localhost:8888)"
echo "   ⚡ Direct Dev API Container      : http://localhost:8880"
echo "   🗄️  PostgreSQL                    : localhost:5433"
echo "   💾 Redis                         : localhost:6380"
echo "   (Zero accounts, zero signups, zero /etc/hosts modifications required)"
echo "=========================================================="
echo ""

docker compose -f docker-compose.dev.yml up --build "$@"
