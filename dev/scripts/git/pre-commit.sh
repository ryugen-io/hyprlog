#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="${SCRIPT_DIR}/../../.."
source "${SCRIPT_DIR}/../lib/log.sh"

SCOPE="PRE-COMMIT"

cd "$PROJECT_ROOT"

log_info "$SCOPE" "=== starting pre-commit.sh ==="
log_info "$SCOPE" "working directory: $(pwd)"
log_info "$SCOPE" "checks: format, lint, test"

ERRORS=0
TOTAL_CHECKS=3

# Format check
log_step "$SCOPE" "check 1/${TOTAL_CHECKS}: format"
log_info "$SCOPE" "executing: cargo fmt --all -- --check"

if cargo fmt --all -- --check >/dev/null 2>&1; then
    log_ok "$SCOPE" "format check passed"
else
    log_error "$SCOPE" "format check failed"
    log_info "$SCOPE" "fix with: just fmt"
    ((ERRORS++)) || true
fi

# Lint
log_step "$SCOPE" "check 2/${TOTAL_CHECKS}: clippy"
log_info "$SCOPE" "executing: cargo clippy --workspace --all-targets -- -D warnings"

if cargo clippy --workspace --all-targets -- -D warnings >/dev/null 2>&1; then
    log_ok "$SCOPE" "clippy check passed"
else
    log_error "$SCOPE" "clippy check failed"
    log_info "$SCOPE" "fix issues shown by: just lint"
    ((ERRORS++)) || true
fi

# Tests
log_step "$SCOPE" "check 3/${TOTAL_CHECKS}: tests"
log_info "$SCOPE" "executing: cargo test --workspace"

if cargo test --workspace --quiet 2>/dev/null; then
    log_ok "$SCOPE" "test check passed"
else
    log_error "$SCOPE" "test check failed"
    log_info "$SCOPE" "run tests with: just test"
    ((ERRORS++)) || true
fi

# Summary
echo ""
log_step "$SCOPE" "generating summary"
log_info "$SCOPE" "checks run: ${TOTAL_CHECKS}"
log_info "$SCOPE" "checks passed: $((TOTAL_CHECKS - ERRORS))"
log_info "$SCOPE" "checks failed: ${ERRORS}"

if [[ $ERRORS -eq 0 ]]; then
    log_ok "$SCOPE" "all checks passed"
    log_ok "$SCOPE" "ready to commit"
    log_info "$SCOPE" "=== pre-commit.sh finished successfully ==="
    exit 0
else
    log_error "$SCOPE" "${ERRORS} check(s) failed"
    log_warn "$SCOPE" "fix issues before committing"
    log_info "$SCOPE" "=== pre-commit.sh finished with errors ==="
    exit 1
fi
