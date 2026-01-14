#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="${SCRIPT_DIR}/../../.."
source "${SCRIPT_DIR}/../lib/log.sh"

SCOPE="AUDIT"
ERRORS=0
SKIPPED=0

cd "$PROJECT_ROOT"

log_info "$SCOPE" "=== starting audit.sh ==="
log_info "$SCOPE" "working directory: $(pwd)"
log_info "$SCOPE" "checks: unused dependencies, security vulnerabilities"

# Unused dependencies
log_step "$SCOPE" "check 1/2: unused dependencies"
log_info "$SCOPE" "tool: cargo-machete"

if cargo machete --version &>/dev/null; then
    log_ok "$SCOPE" "cargo-machete found"
    log_step "$SCOPE" "executing: cargo machete"

    if cargo machete 2>&1; then
        log_ok "$SCOPE" "no unused dependencies detected"
    else
        log_warn "$SCOPE" "unused dependencies found"
        ((ERRORS++)) || true
    fi
else
    log_warn "$SCOPE" "cargo-machete not installed, skipping"
    log_info "$SCOPE" "install with: cargo install cargo-machete"
    ((SKIPPED++)) || true
fi

echo ""

# Security audit
log_step "$SCOPE" "check 2/2: security vulnerabilities"
log_info "$SCOPE" "tool: cargo-audit"
log_info "$SCOPE" "database: RustSec Advisory Database"

if cargo audit --version &>/dev/null; then
    log_ok "$SCOPE" "cargo-audit found"
    log_step "$SCOPE" "executing: cargo audit"

    if cargo audit 2>&1; then
        log_ok "$SCOPE" "no known vulnerabilities detected"
    else
        log_error "$SCOPE" "security vulnerabilities detected!"
        log_warn "$SCOPE" "review above output and update affected dependencies"
        ((ERRORS++)) || true
    fi
else
    log_warn "$SCOPE" "cargo-audit not installed, skipping"
    log_info "$SCOPE" "install with: cargo install cargo-audit"
    ((SKIPPED++)) || true
fi

echo ""

# Summary
log_step "$SCOPE" "generating summary"
log_info "$SCOPE" "checks run: $((2 - SKIPPED))/2"
log_info "$SCOPE" "checks skipped: ${SKIPPED}"
log_info "$SCOPE" "issues found: ${ERRORS}"

if [[ $ERRORS -eq 0 ]]; then
    log_ok "$SCOPE" "all checks passed"
    log_info "$SCOPE" "=== audit.sh finished successfully ==="
else
    log_error "$SCOPE" "${ERRORS} issue(s) require attention"
    log_info "$SCOPE" "=== audit.sh finished with errors ==="
    exit 1
fi
