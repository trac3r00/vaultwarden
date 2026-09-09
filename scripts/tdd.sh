#!/usr/bin/env bash
# Recreated TDD runner for Vaultwarden.
# Matches CI's sqlite unit-test surface (see .github/workflows/build.yml).
set -euo pipefail
cd "$(dirname "$0")/.."

FILTER="${1:-}"
FEATURES="${TDD_FEATURES:-sqlite}"

if [[ -n "${FILTER}" ]]; then
  exec cargo test --features "${FEATURES}" --bins "${FILTER}" -- --nocapture
fi

exec cargo test --features "${FEATURES}" --bins -- --nocapture
