#!/usr/bin/env bash
set -euo pipefail

ENV_FILE="${HOOK_CI_ENV_FILE:-.env.ci}"
COMPOSE_PROJECT_NAME="hook-ci-smoke-${GITHUB_RUN_ID:-local}-${GITHUB_RUN_ATTEMPT:-1}"
OVERRIDE_FILE="$(mktemp)"

load_env_file() {
  if [[ ! -f "$ENV_FILE" ]]; then
    echo "CI Docker env file is missing: $ENV_FILE" >&2
    exit 1
  fi

  set -a
  # shellcheck disable=SC1090
  . "$ENV_FILE"
  set +a
}

write_override_file() {
  cat > "$OVERRIDE_FILE" <<EOF
services:
  hook:
    pull_policy: never
volumes:
  hook-postgres:
    name: ${COMPOSE_PROJECT_NAME}-postgres
  hook-redis:
    name: ${COMPOSE_PROJECT_NAME}-redis
EOF
}

compose() {
  COMPOSE_PROJECT_NAME="$COMPOSE_PROJECT_NAME" docker compose --env-file "$ENV_FILE" -f docker-compose.prebuilt.yml -f "$OVERRIDE_FILE" "$@"
}

cleanup() {
  compose down -v --remove-orphans
  rm -f "$OVERRIDE_FILE"
}

wait_for_health() {
  local health_url="http://127.0.0.1:${HOOK_PORT:-15555}/health"
  for attempt in $(seq 1 60); do
    if curl -fsS "$health_url" >/tmp/hook-ci-health.json; then
      cat /tmp/hook-ci-health.json
      return
    fi
    echo "Waiting for Hook health endpoint (${attempt}/60): $health_url"
    sleep 2
  done

  compose ps
  compose logs hook
  echo "Hook health endpoint did not become ready: $health_url" >&2
  exit 1
}

main() {
  load_env_file
  write_override_file
  trap cleanup EXIT

  compose up -d
  wait_for_health
}

main "$@"
