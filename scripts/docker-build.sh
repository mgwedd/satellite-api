#!/usr/bin/env bash
set -e

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )/.." && pwd )"
cd "$DIR"

TAG="${1:-astrea-sda-api:latest}"

echo "=========================================================="
echo "🐳 Building Production Multi-Stage Docker Container: $TAG"
echo "=========================================================="
echo "ℹ️ Note: Automated test verification executes inside build container."
echo "ℹ️ Production container contains NO local dev keys, debug code, or test files."
echo ""

docker build -t "$TAG" -f Dockerfile .

echo ""
echo "=========================================================="
echo "✅ Production container built successfully: $TAG"
echo "=========================================================="
