#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="${SCRIPT_DIR}/../../.."
source "${SCRIPT_DIR}/../lib/log.sh"

SCOPE="LINT"
MODE="${1:-}"

cd "$PROJECT_ROOT"

log_info "$SCOPE" "=== starting lint.sh ==="
log_info "$SCOPE" "working directory: $(pwd)"
log_info "$SCOPE" "mode: ${MODE:-standard}"

case "$MODE" in
    --strict)
        log_info "$SCOPE" "running in STRICT mode"
        log_info "$SCOPE" "enabled lints: pedantic, nursery"
        log_info "$SCOPE" "allowed: module_name_repetitions, must_use_candidate"

        log_step "$SCOPE" "executing: cargo clippy --workspace --all-targets --all-features"

        if cargo clippy --workspace --all-targets --all-features -- \
            -D warnings \
            -D clippy::pedantic \
            -D clippy::nursery \
            -A clippy::module_name_repetitions \
            -A clippy::must_use_candidate; then
            log_ok "$SCOPE" "no issues found"
            log_info "$SCOPE" "=== lint.sh finished successfully ==="
        else
            log_error "$SCOPE" "clippy found issues"
            log_info "$SCOPE" "=== lint.sh finished with errors ==="
            exit 1
        fi
        ;;
    "")
        log_info "$SCOPE" "running in STANDARD mode"
        log_info "$SCOPE" "warnings treated as errors (-D warnings)"

        log_step "$SCOPE" "executing: cargo clippy --workspace --all-targets"

        if cargo clippy --workspace --all-targets -- -D warnings; then
            log_ok "$SCOPE" "no issues found"
            log_info "$SCOPE" "=== lint.sh finished successfully ==="
        else
            log_error "$SCOPE" "clippy found issues"
            log_info "$SCOPE" "=== lint.sh finished with errors ==="
            exit 1
        fi
        ;;
    --help|-h)
        echo "Usage: lint.sh [--strict]"
        echo ""
        echo "  (none)   standard clippy warnings"
        echo "  --strict pedantic + nursery lints"
        exit 0
        ;;
    *)
        log_error "$SCOPE" "unknown flag: $MODE"
        echo ""
        echo "Usage: lint.sh [--strict]"
        echo ""
        echo "  (none)   standard clippy warnings"
        echo "  --strict pedantic + nursery lints"
        exit 1
        ;;
esac
