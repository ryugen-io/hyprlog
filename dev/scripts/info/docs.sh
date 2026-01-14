#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="${SCRIPT_DIR}/../../.."
source "${SCRIPT_DIR}/../lib/log.sh"

SCOPE="DOCS"
MODE="${1:-}"

cd "$PROJECT_ROOT"

log_info "$SCOPE" "=== starting docs.sh ==="
log_info "$SCOPE" "working directory: $(pwd)"
log_info "$SCOPE" "mode: ${MODE:-generate}"

case "$MODE" in
    --open)
        log_info "$SCOPE" "generating and opening documentation"
        log_step "$SCOPE" "executing: cargo doc --workspace --no-deps --open"
        echo ""
        if cargo doc --workspace --no-deps --open; then
            echo ""
            log_ok "$SCOPE" "documentation generated"
            log_ok "$SCOPE" "opened in default browser"
            log_info "$SCOPE" "=== docs.sh finished successfully ==="
        else
            echo ""
            log_error "$SCOPE" "documentation generation failed"
            log_info "$SCOPE" "=== docs.sh failed ==="
            exit 1
        fi
        ;;
    --help|-h)
        echo "Usage: docs.sh [--open]"
        echo ""
        echo "  (none)   generate documentation"
        echo "  --open   generate and open in browser"
        exit 0
        ;;
    "")
        log_info "$SCOPE" "generating documentation (no browser)"
        log_step "$SCOPE" "executing: cargo doc --workspace --no-deps"
        echo ""
        if cargo doc --workspace --no-deps; then
            echo ""
            log_ok "$SCOPE" "documentation generated"
            log_info "$SCOPE" "output: target/doc/hl_core/index.html"
            log_info "$SCOPE" "open with: just docs --open"
            log_info "$SCOPE" "=== docs.sh finished successfully ==="
        else
            echo ""
            log_error "$SCOPE" "documentation generation failed"
            log_info "$SCOPE" "=== docs.sh failed ==="
            exit 1
        fi
        ;;
    *)
        log_error "$SCOPE" "unknown flag: $MODE"
        echo ""
        echo "Usage: docs.sh [--open]"
        echo ""
        echo "  (none)   generate documentation"
        echo "  --open   generate and open in browser"
        exit 1
        ;;
esac
