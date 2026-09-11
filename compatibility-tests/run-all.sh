#!/usr/bin/env bash
#
# k6 runner script — orchestrates the full compatibility test suite.
#
# Usage:
#   ./compatibility-tests/run-all.sh              # Run all tests with JSON output
#   K6_OUTPUT_DIR=./results/compatibility ./compatibility-tests/run-all.sh  # Custom output directory
#
# Prerequisites:
#   - Docker Compose stack running (app + postgres)
#   - Seed data loaded (bun run scripts/seed/index.ts)
#   - k6 installed (or use docker run grafana/k6)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

APP_BASE_URL="${APP_BASE_URL:-http://localhost:80}"
K6_OUTPUT_DIR="${K6_OUTPUT_DIR:-${PROJECT_ROOT}/results/compatibility}"
BOOTSTRAP_KEY="${BOOTSTRAP_KEY:-replace-me-with-long-random-bootstrap-secret}"
HEALTH_ENDPOINT="${APP_BASE_URL}/api/v1/about"
HEALTH_TIMEOUT=30

echo "============================================"
echo "  Utopia Compatibility Verification Suite"
echo "============================================"
echo ""
echo "  APP_BASE_URL:  ${APP_BASE_URL}"
echo "  OUTPUT_DIR:    ${K6_OUTPUT_DIR}"
echo ""

# ---------------------------------------------------------------------------
# Step 1: Wait for application health check
# ---------------------------------------------------------------------------
echo "[1/4] Waiting for application health check..."
elapsed=0
while [ $elapsed -lt $HEALTH_TIMEOUT ]; do
  if curl -sf "${HEALTH_ENDPOINT}" > /dev/null 2>&1; then
    echo "  ✓ Application is healthy"
    break
  fi
  sleep 1
  elapsed=$((elapsed + 1))
done

if [ $elapsed -ge $HEALTH_TIMEOUT ]; then
  echo "  ✗ Application did not become healthy within ${HEALTH_TIMEOUT}s"
  exit 1
fi

# ---------------------------------------------------------------------------
# Step 2: Load seed data
# ---------------------------------------------------------------------------
echo "[2/4] Loading seed data..."
cd "${PROJECT_ROOT}"
if command -v bun &> /dev/null; then
  DATABASE_URL="${DATABASE_URL:-postgres://utopia:utopia@localhost:5432/utopia}" \
    bun run scripts/seed/index.ts
else
  echo "  ⚠ bun not found, skipping seed (ensure seed data is loaded manually)"
fi

# ---------------------------------------------------------------------------
# Step 3: Create output directory
# ---------------------------------------------------------------------------
mkdir -p "${K6_OUTPUT_DIR}"

# ---------------------------------------------------------------------------
# Step 4: Run k6 tests
# ---------------------------------------------------------------------------
echo "[3/4] Running k6 test suite..."

export APP_BASE_URL
export BOOTSTRAP_KEY

# Run auth tests
echo "  → Running auth tests..."
k6 run \
  --compatibility-mode=extended \
  --env APP_BASE_URL="${APP_BASE_URL}" \
  --env BOOTSTRAP_KEY="${BOOTSTRAP_KEY}" \
  --out "json=${K6_OUTPUT_DIR}/auth-results.json" \
  "${SCRIPT_DIR}/auth.ts" || true

# Run accounts tests
echo "  → Running accounts tests..."
k6 run \
  --compatibility-mode=extended \
  --env APP_BASE_URL="${APP_BASE_URL}" \
  --env BOOTSTRAP_KEY="${BOOTSTRAP_KEY}" \
  --out "json=${K6_OUTPUT_DIR}/accounts-results.json" \
  "${SCRIPT_DIR}/accounts.ts" || true

# Run transactions tests
echo "  → Running transactions tests..."
k6 run \
  --compatibility-mode=extended \
  --env APP_BASE_URL="${APP_BASE_URL}" \
  --env BOOTSTRAP_KEY="${BOOTSTRAP_KEY}" \
  --out "json=${K6_OUTPUT_DIR}/transactions-results.json" \
  "${SCRIPT_DIR}/transactions.ts" || true

# ---------------------------------------------------------------------------
# Step 5: Summary
# ---------------------------------------------------------------------------
echo "[4/4] Test suite complete."
echo ""
echo "  Results: ${K6_OUTPUT_DIR}/"
echo "    - auth-results.json"
echo "    - accounts-results.json"
echo "    - transactions-results.json"
echo ""
echo "============================================"
