#!/usr/bin/env bash
# Passkey / FIDO2 / related-origins TDD target.
set -euo pipefail
cd "$(dirname "$0")/.."
./scripts/tdd.sh cipher_login
./scripts/tdd.sh feature_flag_tests
./scripts/tdd.sh web_well_known
