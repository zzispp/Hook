#!/usr/bin/env bash
set -euo pipefail

IMAGE_NAME="${IMAGE_NAME:?IMAGE_NAME is required}"
REGISTRY_HOST="ghcr.io"
MAX_VERIFY_ATTEMPTS="${MAX_VERIFY_ATTEMPTS:-12}"
VERIFY_SLEEP_SECONDS="${VERIFY_SLEEP_SECONDS:-10}"

image_repository() {
  case "$IMAGE_NAME" in
    ghcr.io/*) printf '%s' "${IMAGE_NAME#ghcr.io/}" ;;
    *) echo "Only ghcr.io images are supported: $IMAGE_NAME" >&2; exit 1 ;;
  esac
}

expected_tags() {
  if [[ "${GITHUB_REF_TYPE}" == "branch" && "${GITHUB_REF_NAME}" == "main" ]]; then
    printf '%s\n' edge nightly
    return
  fi

  if [[ "${GITHUB_REF_TYPE}" == "tag" ]]; then
    printf '%s\n' "$GITHUB_REF_NAME" "${GITHUB_REF_NAME#v}"
    if [[ "$GITHUB_REF_NAME" != *-* ]]; then
      printf '%s\n' latest
    fi
  fi
}

fetch_public_token() {
  local repository="$1"
  curl -fsS "https://${REGISTRY_HOST}/token?service=${REGISTRY_HOST}&scope=repository:${repository}:pull" | jq -r '.token'
}

verify_public_tag() {
  local repository="$1"
  local tag="$2"
  local token="$3"
  local manifest_url="https://${REGISTRY_HOST}/v2/${repository}/manifests/${tag}"

  curl -fsS -H "Authorization: Bearer ${token}" \
    -H "Accept: application/vnd.oci.image.index.v1+json, application/vnd.docker.distribution.manifest.list.v2+json, application/vnd.oci.image.manifest.v1+json, application/vnd.docker.distribution.manifest.v2+json" \
    "$manifest_url" >/dev/null
}

verify_with_retry() {
  local repository="$1"
  local tag="$2"
  local token

  for attempt in $(seq 1 "$MAX_VERIFY_ATTEMPTS"); do
    if token="$(fetch_public_token "$repository")" && verify_public_tag "$repository" "$tag" "$token"; then
      echo "Verified public GHCR image tag: ${IMAGE_NAME}:${tag}"
      return
    fi
    echo "Waiting for public GHCR image tag (${attempt}/${MAX_VERIFY_ATTEMPTS}): ${IMAGE_NAME}:${tag}"
    sleep "$VERIFY_SLEEP_SECONDS"
  done

  echo "Public GHCR image tag is not available: ${IMAGE_NAME}:${tag}" >&2
  exit 1
}

main() {
  local repository
  local tag
  local tags=()
  repository="$(image_repository)"
  while IFS= read -r tag; do
    tags+=("$tag")
  done < <(expected_tags)

  if [[ "${#tags[@]}" -eq 0 ]]; then
    echo "No Docker tags are expected for ${GITHUB_REF_TYPE}:${GITHUB_REF_NAME}" >&2
    exit 1
  fi

  for tag in "${tags[@]}"; do
    verify_with_retry "$repository" "$tag"
  done
}

main "$@"
