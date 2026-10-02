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
echo "🐳 Launching Docker DNS services (Postgres + Redis + Nginx)..."
docker compose -f docker-compose.dev-hot.yml up -d

echo ""
echo "=========================================================="
echo "✅ Docker Dev Stack active (Postgres + Redis + Tailscale + Nginx):"
echo "   🌐 Main Gateway (Tailscale HTTPS) :"
echo "      - Interactive Swagger UI/ReDoc: https://sda/ (or https://sda/docs)"
echo "      - Direct API Endpoints        : https://sda/v1/... or https://sda/api/v1/..."
echo "      - Team / Custom Domain Alias  : https://sda.dev.astrealabs.com/api/v1/..."
echo "   🛠️  DIY Nginx Gateway (Local Only) :"
echo "      - HTTPS Local                 : https://localhost:8443/sda/api/v1"
echo "      - HTTP Local                  : http://localhost:8888"
echo "   🗄️  PostgreSQL                   : localhost:5432"
echo "   💾 Redis                        : localhost:6379"
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
