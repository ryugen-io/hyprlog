#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="${SCRIPT_DIR}/../../.."
source "${SCRIPT_DIR}/../lib/log.sh"

SCOPE="FMT"
MODE="${1:-}"

cd "$PROJECT_ROOT"

log_info "$SCOPE" "=== starting fmt.sh ==="
log_info "$SCOPE" "working directory: $(pwd)"
log_info "$SCOPE" "mode: ${MODE:-format}"

case "$MODE" in
    --check)
        log_info "$SCOPE" "running in CHECK mode (no modifications)"
        log_step "$SCOPE" "executing: cargo fmt --all -- --check"

        if cargo fmt --all -- --check; then
            log_ok "$SCOPE" "all files are properly formatted"
            log_info "$SCOPE" "=== fmt.sh finished successfully ==="
        else
            log_error "$SCOPE" "format issues detected"
            log_info "$SCOPE" "run 'just fmt' to auto-fix"
            log_info "$SCOPE" "=== fmt.sh finished with errors ==="
            exit 1
        fi
        ;;
    "")
        log_info "$SCOPE" "running in FORMAT mode (will modify files)"
        log_step "$SCOPE" "executing: cargo fmt --all"

        if cargo fmt --all; then
            log_ok "$SCOPE" "all files formatted successfully"
            log_info "$SCOPE" "=== fmt.sh finished successfully ==="
        else
            log_error "$SCOPE" "formatting failed"
            log_info "$SCOPE" "=== fmt.sh finished with errors ==="
            exit 1
        fi
        ;;
    --help|-h)
        echo "Usage: fmt.sh [--check]"
        echo ""
        echo "  (none)   format all code"
        echo "  --check  check without modifying"
        exit 0
        ;;
    *)
        log_error "$SCOPE" "unknown flag: $MODE"
        echo ""
        echo "Usage: fmt.sh [--check]"
        echo ""
        echo "  (none)   format all code"
        echo "  --check  check without modifying"
        exit 1
        ;;
esac
