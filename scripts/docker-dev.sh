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
echo "   - HTTPS Custom Domain   : https://astrealabs.local.com/sda/api/v1 (or https://localhost:8443/sda/api/v1)"
echo "   - HTTP Gateway          : http://localhost:8888"
echo "   - Direct API            : http://localhost:8880"
echo "   - PostgreSQL            : localhost:5433"
echo "   - Redis                 : localhost:6380"
echo "   (Tip: Add '127.0.0.1 astrealabs.local.com' to /etc/hosts for custom domain local testing)"
echo "=========================================================="
echo ""

docker compose -f docker-compose.dev.yml up --build "$@"
