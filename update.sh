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

require_env_file() {
  if [[ ! -f "$ENV_FILE" ]]; then
    echo ".env is missing. Run ./scripts/create-docker-env.sh first." >&2
    exit 1
  fi
}

update_source() {
  if [[ ! -d "$ROOT_DIR/.git" ]]; then
    return
  fi

  git -C "$ROOT_DIR" pull --ff-only
}

update_dependency_images() {
  set_compose_args
  docker compose "${COMPOSE_ARGS[@]}" pull postgres redis
}

rebuild_hook() {
  set_compose_args
  docker compose "${COMPOSE_ARGS[@]}" up -d --build
}

require_env_file
cd "$ROOT_DIR"
update_source
update_dependency_images
rebuild_hook

echo "Hook is updated. Docker named volumes were not removed."
