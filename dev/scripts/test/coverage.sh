#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="${SCRIPT_DIR}/../../.."
source "${SCRIPT_DIR}/../lib/log.sh"

SCOPE="COVERAGE"

cd "$PROJECT_ROOT"

log_info "$SCOPE" "=== starting coverage.sh ==="
log_info "$SCOPE" "working directory: $(pwd)"
log_info "$SCOPE" "tool: cargo-llvm-cov"

log_step "$SCOPE" "checking for cargo-llvm-cov"
if ! command -v cargo-llvm-cov &>/dev/null; then
    log_error "$SCOPE" "cargo-llvm-cov not installed"
    log_info "$SCOPE" "install with: cargo install cargo-llvm-cov"
    log_info "$SCOPE" "=== coverage.sh failed ==="
    exit 1
fi
log_ok "$SCOPE" "cargo-llvm-cov found"

log_step "$SCOPE" "running tests with instrumentation"
log_info "$SCOPE" "executing: cargo llvm-cov --workspace --html"
echo ""

if cargo llvm-cov --workspace --html; then
    echo ""
    log_ok "$SCOPE" "coverage analysis complete"
    log_info "$SCOPE" "report: target/llvm-cov/html/index.html"
    log_info "$SCOPE" "open in browser to view detailed coverage"
    log_info "$SCOPE" "=== coverage.sh finished successfully ==="
else
    echo ""
    log_error "$SCOPE" "coverage analysis failed"
    log_info "$SCOPE" "=== coverage.sh failed ==="
    exit 1
fi
