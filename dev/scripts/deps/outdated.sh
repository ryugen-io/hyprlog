#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="${SCRIPT_DIR}/../../.."
source "${SCRIPT_DIR}/../lib/log.sh"

SCOPE="OUTDATED"

cd "$PROJECT_ROOT"

log_info "$SCOPE" "=== starting outdated.sh ==="
log_info "$SCOPE" "working directory: $(pwd)"

log_step "$SCOPE" "checking for cargo-outdated"
if ! cargo outdated --version &>/dev/null; then
    log_warn "$SCOPE" "cargo-outdated not installed"
    log_info "$SCOPE" "install with: cargo install cargo-outdated"
    log_info "$SCOPE" "falling back to cargo update --dry-run"

    log_step "$SCOPE" "executing: cargo update --dry-run"
    output=$(cargo update --dry-run 2>&1 || true)

    if echo "$output" | grep -qE "Updating|Adding"; then
        echo ""
        echo "$output" | grep -E "Updating|Adding"
        echo ""
        log_warn "$SCOPE" "updates available (see above)"
    else
        log_ok "$SCOPE" "all dependencies up to date"
    fi

    log_info "$SCOPE" "=== outdated.sh finished ==="
    exit 0
fi

log_ok "$SCOPE" "cargo-outdated found"
log_step "$SCOPE" "executing: cargo outdated --workspace"

output=$(cargo outdated --workspace 2>&1)

if echo "$output" | grep -q "All dependencies are up to date"; then
    log_ok "$SCOPE" "all dependencies are up to date"
else
    echo ""
    echo "$output"
    echo ""
    log_warn "$SCOPE" "outdated dependencies found (see above)"
    log_info "$SCOPE" "update with: cargo update"
fi

log_info "$SCOPE" "=== outdated.sh finished successfully ==="
