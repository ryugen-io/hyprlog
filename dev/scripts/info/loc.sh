#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="${SCRIPT_DIR}/../../.."
source "${SCRIPT_DIR}/../lib/log.sh"

SCOPE="LOC"

cd "$PROJECT_ROOT"

log_info "$SCOPE" "=== starting loc.sh ==="
log_info "$SCOPE" "working directory: $(pwd)"
log_info "$SCOPE" "excluding: target, .git, .tmp"

log_step "$SCOPE" "detecting line counting tool"

if command -v tokei &>/dev/null; then
    log_ok "$SCOPE" "tokei found"
    log_step "$SCOPE" "executing: tokei --exclude target --exclude .git --exclude .tmp"
    echo ""
    tokei --exclude target --exclude .git --exclude .tmp
    echo ""
    log_ok "$SCOPE" "line count complete"
    log_info "$SCOPE" "=== loc.sh finished successfully ==="
elif command -v cloc &>/dev/null; then
    log_ok "$SCOPE" "cloc found (tokei fallback)"
    log_step "$SCOPE" "executing: cloc --exclude-dir=target,.git,.tmp ."
    echo ""
    cloc --exclude-dir=target,.git,.tmp .
    echo ""
    log_ok "$SCOPE" "line count complete"
    log_info "$SCOPE" "=== loc.sh finished successfully ==="
else
    log_error "$SCOPE" "no line counting tool found"
    log_info "$SCOPE" "install with: cargo install tokei"
    log_info "$SCOPE" "alternative: apt install cloc"
    log_info "$SCOPE" "=== loc.sh failed ==="
    exit 1
fi
