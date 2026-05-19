#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PIDS=()

cleanup() {
  for pid in "${PIDS[@]}"; do
    if kill -0 "$pid" 2>/dev/null; then
      kill "$pid" 2>/dev/null || true
    fi
  done
}

trap cleanup EXIT INT TERM

cd "$ROOT_DIR"

pnpm dev:backend &
PIDS+=("$!")

pnpm dev:frontend &
PIDS+=("$!")

wait -n "${PIDS[@]}"
