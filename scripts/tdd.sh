#!/usr/bin/env bash
# Recreated TDD runner for Vaultwarden.
# Matches CI's sqlite unit-test surface (see .github/workflows/build.yml).
set -euo pipefail
cd "$(dirname "$0")/.."

FILTER="${1:-}"
FEATURES="${TDD_FEATURES:-sqlite}"

if [[ -z "${FILTER}" ]]; then
  exec cargo test --features "${FEATURES}" --bins -- --nocapture
fi

# A filter that matches nothing makes cargo exit 0 with "0 passed", so sum the
# executed tests across every test binary and fail if the total is zero.
LOG="$(mktemp)"
trap 'rm -f "${LOG}"' EXIT

set +e
cargo test --features "${FEATURES}" --bins "${FILTER}" -- --nocapture | tee "${LOG}"
STATUS="${PIPESTATUS[0]}"
set -e

if [[ "${STATUS}" -ne 0 ]]; then
  exit "${STATUS}"
fi

EXECUTED="$(awk '/^test result: / {
  for (i = 2; i <= NF; i++) if ($i ~ /^(passed|failed);$/) n += $(i - 1)
} END { print n + 0 }' "${LOG}")"

if [[ "${EXECUTED}" -eq 0 ]]; then
  echo "tdd.sh: filter '${FILTER}' ran no tests (it matched none, or only #[ignore]d tests)" >&2
  exit 1
fi
