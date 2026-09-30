#!/usr/bin/env bash
set -e

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )/.." && pwd )"
cd "$DIR"

echo "============================================================"
echo "🛰️  Satellite API - Interactive OpenAPI UI Launcher"
echo "============================================================"
echo ""
echo "Available Interactive UI Modes:"
echo "  1) Axum Live API + Swagger UI & Redoc"
echo "     Runs cargo run. Interactive Swagger UI: http://localhost:8080/"
echo "     Redoc documentation view: http://localhost:8080/docs"
echo ""
echo "  2) Fern Interactive Docs Dev Portal"
echo "     Runs 'fern docs dev' for live multi-language SDK playground."
echo ""
echo "  3) Static HTML UI Preview"
echo "     Serves api-docs/index.html on http://localhost:8000"
echo "============================================================"

MODE="${1:-1}"

case "$MODE" in
  1|"server"|"swagger")
    echo "🚀 Launching Axum Satellite API Server..."
    export PATH="$HOME/.cargo/bin:$PATH"
    cargo run
    ;;
  2|"fern")
    echo "🌿 Launching Fern Docs Dev Portal..."
    fern docs dev
    ;;
  3|"static"|"html")
    echo "🌐 Launching local HTTP server for api-docs/index.html..."
    python3 -m http.server 8000 --directory api-docs
    ;;
  *)
    echo "Usage: ./scripts/dev-ui.sh [1|2|3|server|fern|static]"
    exit 1
    ;;
esac
