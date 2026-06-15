#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${BASE_URL:-http://127.0.0.1:5555}"
ADMIN_BEARER="${ADMIN_BEARER:-}"
DB_URL="${DB_URL:-postgres://postgres:123456@127.0.0.1:5433/postgres}"
GROUP_CODE="${GROUP_CODE:-}"
MODEL_NAME="${MODEL_NAME:-}"
API_FORMAT="${API_FORMAT:-}"
WINDOW="${WINDOW:-24h}"
INCLUDE_EXCLUDED="${INCLUDE_EXCLUDED:-true}"
SKIP_DB_CHECKS="${SKIP_DB_CHECKS:-0}"
ARTIFACT_DIR="${ARTIFACT_DIR:-/tmp/routing-learning-$(date +%Y%m%d-%H%M%S)}"

profiles=(
  balanced
  first_byte
  high_tps
  cost_optimal
  high_availability
  cache_affinity_plus
  fixed_priority_plus
  custom
)

if [[ "${1:-}" == "--help" ]]; then
  cat <<'EOF'
Usage:
  ADMIN_BEARER=... scripts/verify-routing-learning-all.sh

Optional env:
  BASE_URL=http://127.0.0.1:5555
  DB_URL=postgres://postgres:123456@127.0.0.1:5433/postgres
  GROUP_CODE=default
  MODEL_NAME=gpt-5.5
  API_FORMAT=openai:chat
  WINDOW=24h
  INCLUDE_EXCLUDED=true
  SKIP_DB_CHECKS=0
  ARTIFACT_DIR=/tmp/routing-learning-...
EOF
  exit 0
fi

if [[ -z "$ADMIN_BEARER" ]]; then
  echo "ADMIN_BEARER is required" >&2
  exit 1
fi

mkdir -p "$ARTIFACT_DIR"
echo "artifact_dir: $ARTIFACT_DIR"

for profile in "${profiles[@]}"; do
  echo
  echo "===== ${profile} ====="
  PROFILE_ID="$profile" \
  BASE_URL="$BASE_URL" \
  ADMIN_BEARER="$ADMIN_BEARER" \
  DB_URL="$DB_URL" \
  GROUP_CODE="$GROUP_CODE" \
  MODEL_NAME="$MODEL_NAME" \
  API_FORMAT="$API_FORMAT" \
  WINDOW="$WINDOW" \
  INCLUDE_EXCLUDED="$INCLUDE_EXCLUDED" \
  SKIP_DB_CHECKS="$SKIP_DB_CHECKS" \
    bash scripts/verify-routing-learning.sh | tee "$ARTIFACT_DIR/${profile}.txt"
done

echo
echo "completed: $ARTIFACT_DIR"
