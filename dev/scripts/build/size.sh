#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="${SCRIPT_DIR}/../../.."
source "${SCRIPT_DIR}/../lib/log.sh"

SCOPE="SIZE"

cd "$PROJECT_ROOT"

log_info "$SCOPE" "=== starting size.sh ==="
log_info "$SCOPE" "working directory: $(pwd)"

RELEASE_DIR="target/release"
BINARIES=(hyprlog)

log_step "$SCOPE" "checking for release build"
if [[ ! -d "$RELEASE_DIR" ]]; then
    log_warn "$SCOPE" "no release build found at ${RELEASE_DIR}"
    log_step "$SCOPE" "building release binaries"
    log_info "$SCOPE" "executing: cargo build --release"
    cargo build --release --quiet
    log_ok "$SCOPE" "release build complete"
else
    log_ok "$SCOPE" "release build found"
fi

log_step "$SCOPE" "calculating binary sizes"
echo ""
printf "%-20s %10s\n" "Binary" "Size"
printf "%-20s %10s\n" "------" "----"

found=0
for bin in "${BINARIES[@]}"; do
    bin_path="${RELEASE_DIR}/${bin}"
    log_info "$SCOPE" "checking: ${bin_path}"
    if [[ -f "$bin_path" ]]; then
        size=$(du -h "$bin_path" | cut -f1)
        printf "%-20s %10s\n" "$bin" "$size"
        ((found++)) || true
    else
        log_warn "$SCOPE" "${bin} not found"
    fi
done

log_step "$SCOPE" "checking for library artifacts"
if [[ -f "${RELEASE_DIR}/libhl_core.rlib" ]]; then
    size=$(du -h "${RELEASE_DIR}/libhl_core.rlib" | cut -f1)
    printf "%-20s %10s\n" "libhl_core.rlib" "$size"
    log_ok "$SCOPE" "library artifact found"
else
    log_info "$SCOPE" "no library artifact found (libhl_core.rlib)"
fi

echo ""
log_ok "$SCOPE" "found ${found} binary/binaries"
log_info "$SCOPE" "=== size.sh finished successfully ==="
