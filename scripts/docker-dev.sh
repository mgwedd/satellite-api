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
echo "🚀 Launching Docker Compose Dev Stack (cargo-watch + Postgres + Redis + Tailscale + Nginx)..."
echo "   🌐 Main Gateway (Tailscale HTTPS) :"
echo "      - Interactive Swagger UI/ReDoc: https://sda/ (or https://sda/docs)"
echo "      - Direct API Endpoints        : https://sda/v1/... or https://sda/api/v1/..."
echo "      - Team / Custom Domain Alias  : https://sda.dev.astrealabs.com/api/v1/..."
echo "   🛠️  DIY Nginx Gateway (Local Only) :"
echo "      - HTTPS Local                 : https://localhost:8443/sda/api/v1"
echo "      - HTTP Local                  : http://localhost:8888"
echo "   ⚡ Direct Dev API               : http://localhost:8880"
echo "   🗄️  PostgreSQL                   : localhost:5433"
echo "   💾 Redis                        : localhost:6380"
echo "=========================================================="
echo ""

docker compose -f docker-compose.dev.yml up --build "$@"
