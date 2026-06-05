#!/usr/bin/env bash
set -euo pipefail

VERSION="${HOOK_RELEASE_VERSION:?HOOK_RELEASE_VERSION is required}"
PLATFORM="${HOOK_RELEASE_PLATFORM:?HOOK_RELEASE_PLATFORM is required}"
TARGET="${HOOK_RELEASE_TARGET:?HOOK_RELEASE_TARGET is required}"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PACKAGE_NAME="hook-${VERSION}-${PLATFORM}"
PACKAGE_DIR="${ROOT_DIR}/dist/${PACKAGE_NAME}"
TARGET_DIR="${ROOT_DIR}/target/${TARGET}/release"

require_file() {
  local path="$1"
  if [[ ! -f "$path" ]]; then
    echo "Required release file is missing: $path" >&2
    exit 1
  fi
}

copy_binary() {
  local name="$1"
  local source="${TARGET_DIR}/${name}"
  require_file "$source"
  install -m 0755 "$source" "${PACKAGE_DIR}/bin/${name}"
}

prepare_package_dir() {
  rm -rf "$PACKAGE_DIR"
  mkdir -p "${PACKAGE_DIR}/bin" "${PACKAGE_DIR}/config"
}

copy_package_files() {
  copy_binary hook_backend
  copy_binary generate_password_hash
  install -m 0644 "${ROOT_DIR}/packaging/config.example.yaml" "${PACKAGE_DIR}/config/config.example.yaml"
  install -m 0644 "${ROOT_DIR}/README_RELEASE.md" "${PACKAGE_DIR}/README_RELEASE.md"
  install -m 0644 "${ROOT_DIR}/LICENSE" "${PACKAGE_DIR}/LICENSE"
}

create_archive() {
  mkdir -p "${ROOT_DIR}/dist"
  tar -C "${ROOT_DIR}/dist" -czf "${ROOT_DIR}/dist/${PACKAGE_NAME}.tar.gz" "$PACKAGE_NAME"
}

main() {
  require_file "${ROOT_DIR}/packaging/config.example.yaml"
  require_file "${ROOT_DIR}/README_RELEASE.md"
  require_file "${ROOT_DIR}/LICENSE"
  prepare_package_dir
  copy_package_files
  create_archive
}

main "$@"
