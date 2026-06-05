#!/usr/bin/env bash
set -euo pipefail

CONFIG_PATH="${HOOK_CI_CONFIG_PATH:-config/ci.yaml}"
DATABASE_URL="${HOOK_CI_DATABASE_URL:-postgres://hook:hook_ci_password@127.0.0.1:5432/hook_ci}"
BASELINE_VERSION="m20260605_000001_initial_stable_baseline"
SENTINEL_ID="00000000-0000-7000-8000-00000000c101"
SENTINEL_CODE="ci-preserve"

run_migration_up() {
  cargo run -p hook_backend -- --config "$CONFIG_PATH" migration up
}

query_scalar() {
  local sql="$1"
  psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -Atc "$sql"
}

assert_scalar() {
  local label="$1"
  local sql="$2"
  local expected="$3"
  local actual
  actual="$(query_scalar "$sql")"
  if [[ "$actual" != "$expected" ]]; then
    echo "${label}: expected ${expected}, got ${actual}" >&2
    exit 1
  fi
}

insert_sentinel_user_group() {
  psql "$DATABASE_URL" -v ON_ERROR_STOP=1 <<SQL
insert into user_groups (id, code, name, description, is_active, is_system, sort_order, created_at, updated_at)
values ('${SENTINEL_ID}', '${SENTINEL_CODE}', 'CI Preserve', 'migration preservation sentinel', true, false, 999999, now(), now())
on conflict (code) do update
set name = excluded.name,
    updated_at = now();
SQL
}

main() {
  test -f "$CONFIG_PATH"

  run_migration_up
  assert_scalar "baseline migration marker" "select count(*) from seaql_migrations where version = '${BASELINE_VERSION}'" "1"
  assert_scalar "default user group seed" "select count(*) from user_groups where code = 'default' and is_system = true" "1"
  assert_scalar "system settings seed" "select count(*) from system_settings where id = 'global'" "1"

  insert_sentinel_user_group
  assert_scalar "sentinel before second migration" "select count(*) from user_groups where code = '${SENTINEL_CODE}'" "1"

  run_migration_up
  assert_scalar "sentinel after second migration" "select count(*) from user_groups where code = '${SENTINEL_CODE}'" "1"
}

main "$@"
