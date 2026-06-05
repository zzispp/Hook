#!/usr/bin/env bash
set -euo pipefail

RELEASE_TAG="${RELEASE_TAG:?RELEASE_TAG is required}"
PRERELEASE="${PRERELEASE:?PRERELEASE is required}"
PLATFORMS=(linux-amd64 linux-arm64 macos-amd64 macos-arm64)
DOWNLOAD_DIR="$(mktemp -d)"

cleanup() {
  rm -rf "$DOWNLOAD_DIR"
}

asset_exists() {
  local release_json="$1"
  local asset_name="$2"
  jq -e --arg name "$asset_name" '.assets[] | select(.name == $name)' <<<"$release_json" >/dev/null
}

assert_release_metadata() {
  local release_json="$1"
  local tag_name
  local is_prerelease
  tag_name="$(jq -r '.tagName' <<<"$release_json")"
  is_prerelease="$(jq -r '.isPrerelease' <<<"$release_json")"

  if [[ "$tag_name" != "$RELEASE_TAG" ]]; then
    echo "Release tag mismatch: expected ${RELEASE_TAG}, got ${tag_name}" >&2
    exit 1
  fi
  if [[ "$is_prerelease" != "$PRERELEASE" ]]; then
    echo "Release prerelease mismatch: expected ${PRERELEASE}, got ${is_prerelease}" >&2
    exit 1
  fi
}

assert_required_assets() {
  local release_json="$1"
  require_asset "$release_json" "install.sh"
  require_asset "$release_json" "SHA256SUMS"
  for platform in "${PLATFORMS[@]}"; do
    require_asset "$release_json" "hook-${RELEASE_TAG}-${platform}.tar.gz"
  done
}

require_asset() {
  local release_json="$1"
  local asset_name="$2"
  if ! asset_exists "$release_json" "$asset_name"; then
    echo "Required release asset is missing: $asset_name" >&2
    exit 1
  fi
}

verify_checksums() {
  gh release download "$RELEASE_TAG" --dir "$DOWNLOAD_DIR" --clobber
  (cd "$DOWNLOAD_DIR" && sha256sum -c SHA256SUMS)
}

main() {
  local release_json
  release_json="$(gh release view "$RELEASE_TAG" --json tagName,isPrerelease,assets)"
  assert_release_metadata "$release_json"
  assert_required_assets "$release_json"
  verify_checksums
}

trap cleanup EXIT
main "$@"
