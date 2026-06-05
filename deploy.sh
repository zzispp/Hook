#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ENV_FILE="$ROOT_DIR/.env"
COMPOSE_ARGS=()

set_compose_args() {
  COMPOSE_ARGS=(--env-file "$ENV_FILE")
  if [[ -n "${HOOK_COMPOSE_FILES:-}" ]]; then
    local old_ifs="$IFS"
    local file
    IFS=':'
    local files=($HOOK_COMPOSE_FILES)
    IFS="$old_ifs"

    for file in "${files[@]}"; do
      COMPOSE_ARGS+=(-f "$file")
    done
  fi
}

create_env_if_missing() {
  if [[ -f "$ENV_FILE" ]]; then
    return
  fi

  "$ROOT_DIR/scripts/create-docker-env.sh"
}

deploy_hook() {
  set_compose_args
  docker compose "${COMPOSE_ARGS[@]}" up -d --build
}

cd "$ROOT_DIR"
create_env_if_missing
deploy_hook

echo "Hook is deployed."
