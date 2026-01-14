#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="${SCRIPT_DIR}/../../.."
source "${SCRIPT_DIR}/../lib/log.sh"

SCOPE="TODO"

cd "$PROJECT_ROOT"

log_info "$SCOPE" "=== starting todo.sh ==="
log_debug "$SCOPE" "script location: ${SCRIPT_DIR}/todo.sh"
log_debug "$SCOPE" "working directory: $(pwd)"

PATTERNS="TODO|FIXME|HACK|XXX|BUG|OPTIMIZE"
log_info "$SCOPE" "patterns to scan: ${PATTERNS}"

EXCLUDE_DIRS="dev/scripts, .tmp"
log_info "$SCOPE" "excluding: ${EXCLUDE_DIRS}"

log_step "$SCOPE" "detecting search tool"
# Use grep - more portable, aliases don't work in scripts
log_info "$SCOPE" "using grep"

log_step "$SCOPE" "scanning .rs files for annotations"
results=$(grep -rn -E "$PATTERNS" --include="*.rs" --exclude-dir="dev" --exclude-dir=".tmp" . 2>/dev/null || true)

if [[ -z "$results" ]]; then
    log_ok "$SCOPE" "scan complete: no annotations found"
    log_info "$SCOPE" "=== todo.sh finished successfully ==="
    exit 0
fi

count=$(echo "$results" | wc -l)
log_info "$SCOPE" "scan complete: found ${count} annotation(s)"

log_step "$SCOPE" "categorizing results"
echo ""
echo "$results" | while IFS= read -r line; do
    if [[ "$line" == *"TODO"* ]]; then
        echo -e "\033[38;2;139;233;253m[TODO]    $line\033[0m"
    elif [[ "$line" == *"FIXME"* ]]; then
        echo -e "\033[38;2;255;85;85m[FIXME]   $line\033[0m"
    elif [[ "$line" == *"HACK"* ]]; then
        echo -e "\033[38;2;241;250;140m[HACK]    $line\033[0m"
    elif [[ "$line" == *"XXX"* ]]; then
        echo -e "\033[38;2;241;250;140m[XXX]     $line\033[0m"
    elif [[ "$line" == *"BUG"* ]]; then
        echo -e "\033[38;2;255;85;85m[BUG]     $line\033[0m"
    elif [[ "$line" == *"OPTIMIZE"* ]]; then
        echo -e "\033[38;2;189;147;249m[OPTIM]   $line\033[0m"
    else
        echo "[OTHER]   $line"
    fi
done
echo ""

log_warn "$SCOPE" "total: ${count} annotation(s) require attention"
log_info "$SCOPE" "=== todo.sh finished successfully ==="
