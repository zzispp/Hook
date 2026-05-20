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

is_pid_running() {
  local status

  status="$(ps -p "$1" -o stat= 2>/dev/null | tr -d '[:space:]')"
  [[ -n "$status" && "$status" != Z* ]]
}

wait_for_first_exit() {
  local pid

  while true; do
    for pid in "${PIDS[@]}"; do
      if ! is_pid_running "$pid"; then
        wait "$pid"
        return $?
      fi
    done
    sleep 1
  done
}

trap cleanup EXIT INT TERM

cd "$ROOT_DIR"

pnpm dev:backend &
PIDS+=("$!")

pnpm dev:frontend &
PIDS+=("$!")

wait_for_first_exit
