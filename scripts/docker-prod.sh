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
echo "🐳 Spinning up PostgreSQL, Redis, Astrea API, Tailscale, and Nginx..."
docker compose up -d --build "$@"

echo ""
echo "=========================================================="
echo "✅ Production stack is running!"
echo "   🌐 Main Gateway (Tailscale HTTPS) :"
echo "      - Interactive Swagger UI/ReDoc: https://sda/ (or https://sda/docs)"
echo "      - Direct API Endpoints        : https://sda/v1/... or https://sda/api/v1/..."
echo "      - Team / Custom Domain Alias  : https://sda.dev.astrealabs.com/api/v1/..."
echo "   🛠️  DIY Nginx Gateway (Local Only) :"
echo "      - HTTPS Local                 : https://localhost:8443/sda/api/v1"
echo "      - HTTP Local                  : http://localhost:8888"
echo "   ⚡ Direct Production API        : http://localhost:8880"
echo "   🗄️  PostgreSQL Database          : postgres:5432 (Docker DNS)"
echo "   💾 Redis L2 Cache               : redis:6379 (Docker DNS)"
echo "=========================================================="

