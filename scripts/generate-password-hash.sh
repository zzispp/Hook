#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 1 ]; then
  echo "usage: scripts/generate-password-hash.sh <password>" >&2
  exit 2
fi

cargo run -q -p user --bin generate_password_hash -- "$1"
