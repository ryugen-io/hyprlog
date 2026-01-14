#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="${SCRIPT_DIR}/../../.."
source "${SCRIPT_DIR}/../lib/log.sh"
source "${SCRIPT_DIR}/../.env"

SCOPE="CLEAN"
MODE="${1:-}"

cd "$PROJECT_ROOT"

log_info "$SCOPE" "=== starting clean.sh ==="
log_info "$SCOPE" "working directory: $(pwd)"
log_info "$SCOPE" "mode: ${MODE:-standard}"

clean_target() {
    log_step "$SCOPE" "checking for target/ directory"
    if [[ -d "target" ]]; then
        local size
        size=$(du -sh target 2>/dev/null | cut -f1)
        log_info "$SCOPE" "found target/ (${size})"
        log_step "$SCOPE" "removing target/"
        rm -rf target
        log_ok "$SCOPE" "target/ removed (freed ${size})"
    else
        log_info "$SCOPE" "target/ not found, skipping"
    fi
}

clean_full() {
    log_info "$SCOPE" "full cleanup mode"
    log_info "$SCOPE" "will remove: target/, Cargo.lock, .tmp/, *.log"

    clean_target

    log_step "$SCOPE" "checking for Cargo.lock"
    if [[ -f "Cargo.lock" ]]; then
        log_info "$SCOPE" "found Cargo.lock"
        log_step "$SCOPE" "removing Cargo.lock"
        rm -f Cargo.lock
        log_ok "$SCOPE" "Cargo.lock removed"
    else
        log_info "$SCOPE" "Cargo.lock not found, skipping"
    fi

    log_step "$SCOPE" "checking for .tmp/"
    if [[ -d ".tmp" ]]; then
        local size
        size=$(du -sh .tmp 2>/dev/null | cut -f1)
        log_info "$SCOPE" "found .tmp/ (${size})"
        log_step "$SCOPE" "removing .tmp/"
        rm -rf .tmp
        log_ok "$SCOPE" ".tmp/ removed (freed ${size})"
    else
        log_info "$SCOPE" ".tmp/ not found, skipping"
    fi

    log_step "$SCOPE" "scanning for *.log files"
    local logs_found
    logs_found=$(find . -name "*.log" -not -path "./target/*" 2>/dev/null | wc -l)
    if [[ $logs_found -gt 0 ]]; then
        log_info "$SCOPE" "found ${logs_found} log file(s)"
        log_step "$SCOPE" "removing log files"
        find . -name "*.log" -not -path "./target/*" -delete 2>/dev/null || true
        log_ok "$SCOPE" "${logs_found} log file(s) removed"
    else
        log_info "$SCOPE" "no log files found, skipping"
    fi
}

clean_nuke() {
    log_warn "$SCOPE" "=== NUKE MODE ==="
    log_warn "$SCOPE" "this will clear the global cargo cache"
    log_warn "$SCOPE" "affects ALL rust projects on this machine"
    log_info "$SCOPE" "cache locations:"
    log_info "$SCOPE" "  ~/.cargo/registry/cache"
    log_info "$SCOPE" "  ~/.cargo/git/db"
    echo ""
    echo -n "Enter PIN to confirm: "
    read -r pin

    if [[ "$pin" != "$NUKE_PIN" ]]; then
        log_error "$SCOPE" "wrong PIN entered"
        log_info "$SCOPE" "=== clean.sh aborted ==="
        exit 1
    fi

    log_ok "$SCOPE" "PIN accepted"
    clean_full

    log_step "$SCOPE" "clearing cargo registry cache"
    if [[ -d ~/.cargo/registry/cache ]]; then
        local size
        size=$(du -sh ~/.cargo/registry/cache 2>/dev/null | cut -f1)
        rm -rf ~/.cargo/registry/cache
        log_ok "$SCOPE" "registry cache cleared (freed ${size})"
    else
        log_info "$SCOPE" "registry cache not found"
    fi

    log_step "$SCOPE" "clearing cargo git cache"
    if [[ -d ~/.cargo/git/db ]]; then
        local size
        size=$(du -sh ~/.cargo/git/db 2>/dev/null | cut -f1)
        rm -rf ~/.cargo/git/db
        log_ok "$SCOPE" "git cache cleared (freed ${size})"
    else
        log_info "$SCOPE" "git cache not found"
    fi

    log_warn "$SCOPE" "next cargo build will re-download all dependencies"
}

case "$MODE" in
    "")
        log_info "$SCOPE" "standard cleanup (target/ only)"
        clean_target
        ;;
    --full)
        clean_full
        ;;
    --nuke)
        clean_nuke
        ;;
    --help|-h)
        echo "Usage: clean.sh [--full|--nuke]"
        echo ""
        echo "  (none)   remove target/ only"
        echo "  --full   + Cargo.lock, .tmp/, *.log"
        echo "  --nuke   + cargo cache (requires PIN)"
        exit 0
        ;;
    *)
        log_error "$SCOPE" "unknown flag: $MODE"
        echo ""
        echo "Usage: clean.sh [--full|--nuke]"
        echo ""
        echo "  (none)   remove target/ only"
        echo "  --full   + Cargo.lock, .tmp/, *.log"
        echo "  --nuke   + cargo cache (requires PIN)"
        exit 1
        ;;
esac

log_ok "$SCOPE" "cleanup complete"
log_info "$SCOPE" "=== clean.sh finished successfully ==="
