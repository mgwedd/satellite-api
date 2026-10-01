#!/usr/bin/env bash
set -e

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )/.." && pwd )"
cd "$DIR"

git config core.hooksPath .githooks

chmod +x .githooks/pre-commit .githooks/pre-push 2>/dev/null || true
chmod +x .cargo-husky/hooks/pre-commit .cargo-husky/hooks/pre-push 2>/dev/null || true
