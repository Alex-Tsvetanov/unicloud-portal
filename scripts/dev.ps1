# Runs the portal locally against the PostgreSQL in Docker (Windows). Same settings as scripts/dev.sh.
# Uses the test-only signing key and demo data. Never use these values anywhere else.
Set-Location (Join-Path $PSScriptRoot "..")
$env:APP_ENV = "development"
if (-not $env:DATABASE_URL) { $env:DATABASE_URL = "postgres://postgres:dev@127.0.0.1:55432/unicloud" }
$env:APP_SIGNING_KEY_PEM = Get-Content -Raw "tests/fixtures/test-only-signing-key.pem"
$env:APP_INSECURE_COOKIE = "1"
if (-not $env:APP_DEMO_PASSWORD) { $env:APP_DEMO_PASSWORD = "demo-pass-123" }
if (-not $env:PORT) { $env:PORT = "8090" }
cargo run --quiet
