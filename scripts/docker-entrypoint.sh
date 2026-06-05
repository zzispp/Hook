#!/usr/bin/env sh
set -eu

CONFIG_PATH="${HOOK_CONFIG_PATH:-/app/config.yaml}"
SERVER_HOST="${HOOK_SERVER_HOST:-0.0.0.0}"
SERVER_PORT="${HOOK_SERVER_PORT:-5555}"
LOG_LEVEL="${HOOK_LOG_LEVEL:-info}"

required_env() {
  name="$1"
  value="$(printenv "$name" || true)"
  if [ -z "$value" ]; then
    echo "$name is required" >&2
    exit 1
  fi
}

yaml_quote() {
  escaped="$(printf '%s' "$1" | sed "s/'/''/g")"
  printf "'%s'" "$escaped"
}

decode_admin_password() {
  if ! decoded="$(printf '%s' "$HOOK_ADMIN_PASSWORD_B64" | base64 -d)"; then
    echo "HOOK_ADMIN_PASSWORD_B64 must be valid base64" >&2
    exit 1
  fi

  printf '%s' "$decoded"
}

write_core_config() {
  cat > "$CONFIG_PATH" <<EOF
server:
  host: $(yaml_quote "$SERVER_HOST")
  port: $SERVER_PORT

database:
  url: $(yaml_quote "$HOOK_DATABASE_URL")
  scheme: "postgres"
  host: "postgres"
  port: 5432
  username: "hook"
  password:
  name: "hook"

jwt:
  secret: $(yaml_quote "$HOOK_JWT_SECRET")
  access_token_ttl_seconds: 86400
  refresh_token_ttl_seconds: 604800

admin:
  id: "00000000-0000-7000-8000-000000000000"
  username: $(yaml_quote "$HOOK_ADMIN_USERNAME")
  email: $(yaml_quote "$HOOK_ADMIN_EMAIL")
  role: "admin"
  is_active: true
  password_hash: $(yaml_quote "$admin_password_hash")
  wallet:
    id: "00000000-0000-7000-8000-000000000001"
    status: "active"
    limit_mode: "unlimited"
EOF
}

append_public_auth_paths() {
  cat >> "$CONFIG_PATH" <<EOF
auth:
  whitelist:
    - methods: ["GET"]
      path_pattern: "/health"
    - methods: ["GET"]
      path_pattern: "/api/i18n/resources"
    - methods: ["GET"]
      path_pattern: "/api/site-info"
    - methods: ["GET"]
      path_pattern: "/api/auth/config"
    - methods: ["POST"]
      path_pattern: "/api/auth/registration-email-code"
    - methods: ["POST"]
      path_pattern: "/api/auth/sign-in"
    - methods: ["POST"]
      path_pattern: "/api/auth/sign-up"
    - methods: ["POST"]
      path_pattern: "/api/auth/refresh"
    - methods: ["GET"]
      path_pattern: "/api/auth/oauth/{provider}/start"
    - methods: ["GET"]
      path_pattern: "/api/auth/oauth/{provider}/callback"
EOF
}

append_auth_action_paths() {
  cat >> "$CONFIG_PATH" <<EOF
    - methods: ["POST"]
      path_pattern: "/api/auth/oauth/{provider}/bind-existing"
    - methods: ["POST"]
      path_pattern: "/api/auth/wallet/nonce"
    - methods: ["POST"]
      path_pattern: "/api/auth/wallet/sign-in"
    - methods: ["POST"]
      path_pattern: "/api/auth/wallet/email-code"
    - methods: ["POST"]
      path_pattern: "/api/auth/wallet/complete"
    - methods: ["POST"]
      path_pattern: "/api/auth/password-reset/request"
    - methods: ["POST"]
      path_pattern: "/api/auth/password-reset/confirm"
    - methods: ["GET"]
      path_pattern: "/api/captcha/config"
    - methods: ["POST"]
      path_pattern: "/api/captcha/challenge"
    - methods: ["POST"]
      path_pattern: "/api/captcha/redeem"
EOF
}

append_proxy_paths() {
  cat >> "$CONFIG_PATH" <<EOF
    - methods: ["GET", "POST"]
      path_pattern: "/v1/*"
    - methods: ["POST"]
      path_pattern: "/v1beta/*"
EOF
}

append_authenticated_paths() {
  cat >> "$CONFIG_PATH" <<EOF
  authenticated:
    - methods: ["GET"]
      path_pattern: "/api/auth/me"
    - methods: ["GET"]
      path_pattern: "/api/navbar"
    - methods: ["GET"]
      path_pattern: "/api/notifications"
    - methods: ["PATCH"]
      path_pattern: "/api/notifications/read-all"
    - methods: ["PATCH"]
      path_pattern: "/api/notifications/{source_type}/{source_id}/read"
    - methods: ["DELETE"]
      path_pattern: "/api/notifications/{source_type}/{source_id}"
EOF
}

append_runtime_config() {
  cat >> "$CONFIG_PATH" <<EOF
security:
  provider_key_secret: $(yaml_quote "$HOOK_PROVIDER_KEY_SECRET")

redis:
  url: $(yaml_quote "$HOOK_REDIS_URL")
  scheme: "redis"
  host: "redis"
  port: 6379
  username:
  password:
  database:
  protocol: "resp3"
  key_prefix: "hook"
  scheduling_snapshot_ttl_seconds: 3600

tracing:
  log_level: $(yaml_quote "$LOG_LEVEL")
EOF
}

write_config() {
  admin_password="$(decode_admin_password)"
  admin_password_hash="$(generate_password_hash "$admin_password")"
  mkdir -p "$(dirname "$CONFIG_PATH")"
  write_core_config
  append_public_auth_paths
  append_auth_action_paths
  append_proxy_paths
  append_authenticated_paths
  append_runtime_config
}

required_env HOOK_DATABASE_URL
required_env HOOK_REDIS_URL
required_env HOOK_JWT_SECRET
required_env HOOK_PROVIDER_KEY_SECRET
required_env HOOK_ADMIN_USERNAME
required_env HOOK_ADMIN_EMAIL
required_env HOOK_ADMIN_PASSWORD_B64

write_config

if [ "$#" -eq 0 ]; then
  hook_backend --config "$CONFIG_PATH" migration up
  exec hook_backend --config "$CONFIG_PATH"
fi

exec hook_backend --config "$CONFIG_PATH" "$@"
