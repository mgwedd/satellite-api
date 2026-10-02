#!/usr/bin/env bash
# ==============================================================================
# Astrea SDA API - Tailscale OSS Gateway Helper Script
# Batteries-included CLI for managing Tailscale across dev, dev-hot, and prod
# ==============================================================================
set -e

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )/.." && pwd )"
cd "$DIR"

# Detect active Tailscale container across environments
find_active_container() {
  for name in astrea-dev-tailscale astrea-dev-hot-tailscale astrea-tailscale; do
    if docker ps --format '{{.Names}}' | grep -q "^${name}$"; then
      echo "$name"
      return 0
    fi
  done
  return 1
}

cmd_urls() {
  echo "=========================================================="
  echo "🌐 Astrea SDA API - Tailscale Gateway Endpoints"
  echo "=========================================================="
  echo "   - Interactive Swagger UI: https://sda/"
  echo "   - ReDoc Interactive Docs: https://sda/docs"
  echo "   - Direct API Endpoints  : https://sda/v1/... (or https://sda/api/v1/...)"
  echo "   - Team Domain Alias     : https://sda.dev.astrealabs.com/api/v1/..."
  echo ""
  echo "🛠️  DIY Nginx Local Fallback:"
  echo "   - Local HTTPS           : https://localhost:8443/sda/api/v1"
  echo "   - Local HTTP            : http://localhost:8888"
  echo "=========================================================="
}

cmd_status() {
  local container
  if ! container=$(find_active_container); then
    echo "⚠️  No Tailscale container is currently running."
    echo "   Start a stack first:"
    echo "     - make dev-hot  (Host app with sub-second hot reload)"
    echo "     - make dev      (Containerized cargo-watch dev stack)"
    echo "     - make prod-run (Local production stack)"
    exit 1
  fi

  echo "🛰️  Checking Tailscale status in container: $container..."
  echo "----------------------------------------------------------"
  docker exec "$container" tailscale status
  echo "----------------------------------------------------------"
  cmd_urls
}

cmd_login() {
  local container
  if ! container=$(find_active_container); then
    echo "⚠️  No Tailscale container is currently running."
    echo "   Start a stack first with 'make dev', 'make dev-hot', or 'make prod-run'."
    exit 1
  fi

  echo "🔑 Searching logs for Tailscale interactive login link in $container..."
  local login_line
  login_line=$(docker logs "$container" 2>&1 | grep -A 2 -B 1 "To authenticate, visit:" | tail -n 3 || true)

  if [ -n "$login_line" ]; then
    echo ""
    echo "$login_line"
    echo ""
    echo "💡 Open the URL above in your browser to approve this node on your tailnet."
  else
    echo "Checking tailscale login inside container..."
    docker exec -it "$container" tailscale login || true
  fi
}

cmd_ping() {
  echo "📡 Testing HTTPS connectivity to https://sda/ ..."
  if curl -k -s -o /dev/null -w "%{http_code}" --connect-timeout 3 https://sda/ &> /dev/null; then
    local status
    status=$(curl -k -s -o /dev/null -w "%{http_code}" https://sda/)
    echo "✅ Success! Received HTTP $status from https://sda/"
  else
    echo "⚠️  Could not reach https://sda/."
    echo "   1. Ensure your Tailscale daemon is running on this machine."
    echo "   2. Ensure the container has joined your tailnet ('./scripts/tailscale.sh status')."
    echo "   3. If authentication is needed, run: './scripts/tailscale.sh login'."
  fi
}

cmd_help() {
  echo "Usage: ./scripts/tailscale.sh [command]"
  echo ""
  echo "Commands:"
  echo "  status   Show Tailscale connection and node status in the active container"
  echo "  login    Display one-time interactive browser login link if TS_AUTHKEY is unset"
  echo "  ping     Test HTTPS connectivity to https://sda/"
  echo "  urls     Display all configured URLs and gateway endpoints"
  echo "  help     Show this help menu"
}

case "${1:-status}" in
  status) cmd_status ;;
  login)  cmd_login ;;
  ping)   cmd_ping ;;
  urls)   cmd_urls ;;
  help|--help|-h) cmd_help ;;
  *)
    echo "Unknown command: $1"
    cmd_help
    exit 1
    ;;
esac
