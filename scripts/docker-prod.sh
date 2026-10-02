#!/usr/bin/env bash
set -e

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )/.." && pwd )"
cd "$DIR"

echo "=========================================================="
echo "🚀 Launching Full Production Multi-Container Docker Stack..."
echo "=========================================================="

# Ensure local RSA keypair exists if running locally
./scripts/setup-keys.sh

echo ""
echo "🐳 Spinning up PostgreSQL, Redis, Astrea API, and Caddy..."
docker compose up -d --build "$@"

echo ""
echo "=========================================================="
echo "✅ Production stack is running!"
echo "   ⚡ Caddy Zero-Config OSS Gateway :"
echo "      - HTTPS (Automatic TLS)       : https://sda.localtest.me:8443 (or https://localhost:8443)"
echo "      - Interactive Swagger UI      : https://sda.localtest.me:8443/ (or /swagger-ui/)"
echo "      - ReDoc API Reference         : https://sda.localtest.me:8443/docs"
echo "      - Direct API Endpoints        : https://sda.localtest.me:8443/v1/... (or /api/v1/...)"
echo "      - HTTP Local Gateway          : http://sda.localtest.me:8888 (or http://localhost:8888)"
echo "   ⚡ Direct Production API        : http://localhost:8880"
echo "   🗄️  PostgreSQL Database          : postgres:5432 (Docker DNS)"
echo "   💾 Redis L2 Cache               : redis:6379 (Docker DNS)"
echo "   (Zero accounts, zero signups, zero /etc/hosts modifications required)"
echo "=========================================================="


