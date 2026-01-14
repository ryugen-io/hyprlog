#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="${SCRIPT_DIR}/../../.."
source "${SCRIPT_DIR}/../lib/log.sh"

SCOPE="TEST"

cd "$PROJECT_ROOT"

log_info "$SCOPE" "=== starting quick.sh ==="
log_info "$SCOPE" "working directory: $(pwd)"

log_step "$SCOPE" "checking for cargo-nextest"
if ! command -v cargo-nextest &>/dev/null; then
    log_warn "$SCOPE" "cargo-nextest not installed"
    log_info "$SCOPE" "install with: cargo install cargo-nextest"
    log_info "$SCOPE" "falling back to cargo test"

    log_step "$SCOPE" "executing: cargo test --workspace"
    echo ""
    if cargo test --workspace; then
        echo ""
        log_ok "$SCOPE" "all tests passed"
        log_info "$SCOPE" "=== quick.sh finished successfully ==="
    else
        echo ""
        log_error "$SCOPE" "tests failed"
        log_info "$SCOPE" "=== quick.sh failed ==="
        exit 1
    fi
    exit 0
fi

log_ok "$SCOPE" "cargo-nextest found"
log_step "$SCOPE" "executing: cargo nextest run --workspace"
echo ""

if cargo nextest run --workspace; then
    echo ""
    log_ok "$SCOPE" "all tests passed"
    log_info "$SCOPE" "=== quick.sh finished successfully ==="
else
    echo ""
    log_error "$SCOPE" "tests failed"
    log_info "$SCOPE" "=== quick.sh failed ==="
    exit 1
fi
