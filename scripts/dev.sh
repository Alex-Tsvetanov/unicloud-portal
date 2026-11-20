#!/usr/bin/env bash
# Runs the portal locally against the PostgreSQL in Docker (see the README's "Run it locally").
# Uses the test-only signing key and demo data. Never use these values anywhere else.
set -euo pipefail
cd "$(dirname "$0")/.."
export APP_ENV=development
export DATABASE_URL="${DATABASE_URL:-postgres://postgres:dev@127.0.0.1:55432/unicloud}"
export APP_SIGNING_KEY_PEM="$(cat tests/fixtures/test-only-signing-key.pem)"
export APP_INSECURE_COOKIE=1
export APP_DEMO_PASSWORD="${APP_DEMO_PASSWORD:-demo-pass-123}"
export PORT="${PORT:-8090}"
export PATH="$HOME/.cargo/bin:$PATH"
exec cargo run --quiet
