#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="${SCRIPT_DIR}/../../.."
source "${SCRIPT_DIR}/../lib/log.sh"

SCOPE="FUZZ"
DURATION="${1:-30}"

cd "$PROJECT_ROOT"

log_info "$SCOPE" "=== starting fuzz.sh ==="
log_info "$SCOPE" "working directory: $(pwd)"
log_info "$SCOPE" "duration per target: ${DURATION}s"

log_step "$SCOPE" "checking for cargo-fuzz"
if ! command -v cargo-fuzz &>/dev/null; then
    log_error "$SCOPE" "cargo-fuzz not installed"
    log_info "$SCOPE" "install with: cargo install cargo-fuzz"
    log_info "$SCOPE" "=== fuzz.sh failed ==="
    exit 1
fi
log_ok "$SCOPE" "cargo-fuzz found"

TARGETS=(
    fuzz_event_parse
    fuzz_style_parse
    fuzz_highlight
    fuzz_format_template
    fuzz_color_hex
    fuzz_config_sources
)

FAILED=0

for target in "${TARGETS[@]}"; do
    log_step "$SCOPE" "fuzzing: $target (${DURATION}s)"
    echo ""
    if cargo fuzz run "$target" -- -max_total_time="$DURATION"; then
        echo ""
        log_ok "$SCOPE" "$target: passed"
    else
        echo ""
        log_error "$SCOPE" "$target: FAILED (check fuzz/artifacts/$target/)"
        FAILED=$((FAILED + 1))
    fi
done

echo ""
if [ "$FAILED" -eq 0 ]; then
    log_ok "$SCOPE" "all ${#TARGETS[@]} fuzz targets passed"
    log_info "$SCOPE" "=== fuzz.sh finished successfully ==="
else
    log_error "$SCOPE" "$FAILED/${#TARGETS[@]} fuzz targets failed"
    log_info "$SCOPE" "=== fuzz.sh failed ==="
    exit 1
fi
