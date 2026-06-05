#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ENV_FILE="$ROOT_DIR/.env"
DEFAULT_ADMIN_USERNAME="admin"
DEFAULT_ADMIN_EMAIL="admin@example.com"

if [[ -e "$ENV_FILE" ]]; then
  echo ".env already exists: $ENV_FILE" >&2
  exit 1
fi

random_secret() {
  local length="${1:-48}"
  local secret=""

  while [[ "${#secret}" -lt "$length" ]]; do
    local remaining=$((length - ${#secret}))
    secret+=$(LC_ALL=C tr -dc 'A-Za-z0-9' < /dev/urandom | head -c "$remaining" || true)
  done

  printf '%s' "$secret"
}

base64_value() {
  printf '%s' "$1" | base64 | tr -d '\n'
}

prompt_with_default() {
  local label="$1"
  local default_value="$2"
  local value=""

  read -r -p "$label [$default_value]: " value
  if [[ -z "$value" ]]; then
    value="$default_value"
  fi

  printf '%s' "$value"
}

prompt_required_password() {
  local password=""
  local confirmation=""

  while true; do
    read -r -s -p "Admin password: " password
    printf '\n' >&2
    read -r -s -p "Confirm admin password: " confirmation
    printf '\n' >&2

    if [[ -z "$password" ]]; then
      echo "Admin password cannot be empty." >&2
      continue
    fi

    if [[ "$password" != "$confirmation" ]]; then
      echo "Admin passwords do not match." >&2
      continue
    fi

    printf '%s' "$password"
    return
  done
}

validate_required() {
  local name="$1"
  local value="$2"

  if [[ -z "$value" ]]; then
    echo "$name cannot be empty." >&2
    exit 1
  fi
}

validate_env_scalar() {
  local name="$1"
  local value="$2"

  if [[ ! "$value" =~ ^[A-Za-z0-9._%+@-]+$ ]]; then
    echo "$name can only contain letters, numbers, dot, underscore, percent, plus, at sign, or hyphen." >&2
    exit 1
  fi
}

validate_email() {
  local value="$1"

  if [[ "$value" != *@* ]]; then
    echo "Admin email must contain @." >&2
    exit 1
  fi
}

echo "Initialize Hook Docker Compose deployment."
HOOK_ADMIN_USERNAME="$(prompt_with_default "Admin username" "$DEFAULT_ADMIN_USERNAME")"
HOOK_ADMIN_EMAIL="$(prompt_with_default "Admin email" "$DEFAULT_ADMIN_EMAIL")"
HOOK_ADMIN_PASSWORD="$(prompt_required_password)"

validate_required "Admin username" "$HOOK_ADMIN_USERNAME"
validate_required "Admin email" "$HOOK_ADMIN_EMAIL"
validate_env_scalar "Admin username" "$HOOK_ADMIN_USERNAME"
validate_env_scalar "Admin email" "$HOOK_ADMIN_EMAIL"
validate_email "$HOOK_ADMIN_EMAIL"

POSTGRES_PASSWORD="$(random_secret 32)"
HOOK_JWT_SECRET="$(random_secret 64)"
HOOK_PROVIDER_KEY_SECRET="$(random_secret 64)"
HOOK_ADMIN_PASSWORD_B64="$(base64_value "$HOOK_ADMIN_PASSWORD")"

umask 077
cat > "$ENV_FILE" <<EOF
POSTGRES_DB=hook
POSTGRES_USER=hook
POSTGRES_PASSWORD=$POSTGRES_PASSWORD

HOOK_PORT=5555
HOOK_LOG_LEVEL=info
HOOK_ADMIN_USERNAME=$HOOK_ADMIN_USERNAME
HOOK_ADMIN_EMAIL=$HOOK_ADMIN_EMAIL
HOOK_ADMIN_PASSWORD_B64=$HOOK_ADMIN_PASSWORD_B64
HOOK_JWT_SECRET=$HOOK_JWT_SECRET
HOOK_PROVIDER_KEY_SECRET=$HOOK_PROVIDER_KEY_SECRET
EOF

echo "Created $ENV_FILE"
echo "Admin username: $HOOK_ADMIN_USERNAME"
echo "Admin email: $HOOK_ADMIN_EMAIL"
