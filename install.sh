#!/usr/bin/env bash
set -euo pipefail

REPO="${HOOK_REPO:-zzispp/Hook}"
VERSION="${HOOK_VERSION:-}"
INSTALL_DIR="${HOOK_INSTALL_DIR:-/opt/hook}"
CONFIG_DIR="${HOOK_CONFIG_DIR:-/etc/hook}"
BIN_DIR="${HOOK_BIN_DIR:-/usr/local/bin}"
TMP_DIR=""

usage() {
  cat <<'USAGE'
Install or update Hook from GitHub Release assets.

Usage:
  install.sh [--version vX.Y.Z] [--repo owner/name] [--install-dir /opt/hook]
             [--config-dir /etc/hook] [--bin-dir /usr/local/bin]

Environment variables:
  HOOK_VERSION      Release tag. Defaults to the latest GitHub Release.
  HOOK_REPO         GitHub repository. Defaults to zzispp/Hook.
  HOOK_INSTALL_DIR  Install root. Defaults to /opt/hook.
  HOOK_CONFIG_DIR   Config directory. Defaults to /etc/hook.
  HOOK_BIN_DIR      Symlink directory. Defaults to /usr/local/bin.
USAGE
}

cleanup() {
  if [[ -n "$TMP_DIR" && -d "$TMP_DIR" ]]; then
    rm -rf "$TMP_DIR"
  fi
}

parse_args() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --version)
        VERSION="${2:?--version requires a value}"
        shift 2
        ;;
      --repo)
        REPO="${2:?--repo requires a value}"
        shift 2
        ;;
      --install-dir)
        INSTALL_DIR="${2:?--install-dir requires a value}"
        shift 2
        ;;
      --config-dir)
        CONFIG_DIR="${2:?--config-dir requires a value}"
        shift 2
        ;;
      --bin-dir)
        BIN_DIR="${2:?--bin-dir requires a value}"
        shift 2
        ;;
      -h|--help)
        usage
        exit 0
        ;;
      *)
        echo "Unknown option: $1" >&2
        usage >&2
        exit 1
        ;;
    esac
  done
}

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "$1 is required" >&2
    exit 1
  fi
}

require_checksum_command() {
  if command -v sha256sum >/dev/null 2>&1; then
    return
  fi
  if command -v shasum >/dev/null 2>&1; then
    return
  fi
  echo "sha256sum or shasum is required" >&2
  exit 1
}

detect_os() {
  case "$(uname -s)" in
    Linux) printf 'linux' ;;
    Darwin) printf 'macos' ;;
    *)
      echo "Unsupported operating system: $(uname -s)" >&2
      exit 1
      ;;
  esac
}

detect_arch() {
  case "$(uname -m)" in
    x86_64|amd64) printf 'amd64' ;;
    arm64|aarch64) printf 'arm64' ;;
    *)
      echo "Unsupported CPU architecture: $(uname -m)" >&2
      exit 1
      ;;
  esac
}

resolve_version() {
  if [[ -n "$VERSION" ]]; then
    printf '%s' "$VERSION"
    return
  fi

  curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' \
    | head -n 1
}

download_asset() {
  local name="$1"
  local url="https://github.com/${REPO}/releases/download/${VERSION}/${name}"
  curl -fL --retry 3 --retry-delay 2 -o "${TMP_DIR}/${name}" "$url"
}

verify_checksum() {
  local archive="$1"
  local checksum_line=""
  checksum_line="$(grep "  ${archive}$" "${TMP_DIR}/SHA256SUMS" || true)"
  if [[ -z "$checksum_line" ]]; then
    echo "SHA256SUMS does not contain ${archive}" >&2
    exit 1
  fi

  if command -v sha256sum >/dev/null 2>&1; then
    (cd "$TMP_DIR" && printf '%s\n' "$checksum_line" | sha256sum -c -)
    return
  fi

  (cd "$TMP_DIR" && printf '%s\n' "$checksum_line" | shasum -a 256 -c -)
}

install_package() {
  local archive="$1"
  local package_name="$2"
  local release_dir="${INSTALL_DIR}/releases/${VERSION}"
  local extracted="${TMP_DIR}/extract/${package_name}"

  mkdir -p "${TMP_DIR}/extract" "${INSTALL_DIR}/releases"
  tar -xzf "${TMP_DIR}/${archive}" -C "${TMP_DIR}/extract"
  if [[ ! -d "$extracted" ]]; then
    echo "Release archive did not contain ${package_name}" >&2
    exit 1
  fi

  rm -rf "${release_dir}.tmp"
  cp -R "$extracted" "${release_dir}.tmp"
  rm -rf "$release_dir"
  mv "${release_dir}.tmp" "$release_dir"
  ln -sfn "$release_dir" "${INSTALL_DIR}/current"
}

install_config_example() {
  mkdir -p "$CONFIG_DIR"
  install -m 0644 "${INSTALL_DIR}/current/config/config.example.yaml" "${CONFIG_DIR}/config.example.yaml"
  if [[ ! -f "${CONFIG_DIR}/config.yaml" ]]; then
    install -m 0600 "${CONFIG_DIR}/config.example.yaml" "${CONFIG_DIR}/config.yaml"
  fi
}

install_binary_links() {
  mkdir -p "$BIN_DIR"
  ln -sfn "${INSTALL_DIR}/current/bin/hook_backend" "${BIN_DIR}/hook_backend"
  ln -sfn "${INSTALL_DIR}/current/bin/generate_password_hash" "${BIN_DIR}/generate_password_hash"
}

print_next_steps() {
  cat <<EOF
Hook ${VERSION} is installed in ${INSTALL_DIR}/current.

Before starting Hook, edit:
  ${CONFIG_DIR}/config.yaml

Generate an admin password hash with:
  ${INSTALL_DIR}/current/bin/generate_password_hash "your-password"

Then run:
  ${INSTALL_DIR}/current/bin/hook_backend --config ${CONFIG_DIR}/config.yaml migration up
  ${INSTALL_DIR}/current/bin/hook_backend --config ${CONFIG_DIR}/config.yaml
EOF
}

main() {
  parse_args "$@"
  require_command curl
  require_command tar
  require_command sed
  require_command grep
  require_checksum_command

  local os=""
  local arch=""
  local platform=""
  local archive=""
  local package_name=""

  os="$(detect_os)"
  arch="$(detect_arch)"
  platform="${os}-${arch}"
  VERSION="$(resolve_version)"
  if [[ -z "$VERSION" ]]; then
    echo "Could not resolve Hook release version" >&2
    exit 1
  fi

  archive="hook-${VERSION}-${platform}.tar.gz"
  package_name="hook-${VERSION}-${platform}"
  TMP_DIR="$(mktemp -d)"
  trap cleanup EXIT

  download_asset "$archive"
  download_asset SHA256SUMS
  verify_checksum "$archive"
  install_package "$archive" "$package_name"
  install_config_example
  install_binary_links
  print_next_steps
}

main "$@"
