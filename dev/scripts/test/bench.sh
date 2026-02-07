#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="${SCRIPT_DIR}/../../.."
source "${SCRIPT_DIR}/../lib/log.sh"

SCOPE="BENCH"

cd "$PROJECT_ROOT"

log_info "$SCOPE" "=== starting bench.sh ==="
log_info "$SCOPE" "working directory: $(pwd)"

log_step "$SCOPE" "executing: cargo bench $*"
echo ""

if cargo bench "$@"; then
    echo ""
    log_ok "$SCOPE" "benchmarks complete"
    log_info "$SCOPE" "report: target/criterion/report/index.html"
    log_info "$SCOPE" "=== bench.sh finished successfully ==="
else
    echo ""
    log_error "$SCOPE" "benchmarks failed"
    log_info "$SCOPE" "=== bench.sh failed ==="
    exit 1
fi
