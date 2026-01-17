#!/usr/bin/env bash
# =============================================================================
# hyprlog dev scripts - logging library
# Source this file in other scripts for consistent kitchn-style output
#
# Usage:
#   source "$(dirname "$0")/../lib/log.sh"
#   log_info "SCOPE" "message"
#   log_ok "SCOPE" "done"
# =============================================================================

# Sweet Dracula palette (24-bit true color)
readonly LOG_GREEN=$'\033[38;2;80;250;123m'
readonly LOG_YELLOW=$'\033[38;2;241;250;140m'
readonly LOG_CYAN=$'\033[38;2;139;233;253m'
readonly LOG_RED=$'\033[38;2;255;85;85m'
readonly LOG_PURPLE=$'\033[38;2;189;147;249m'
readonly LOG_NC=$'\033[0m'

# Format: <tag> SCOPE  message
# Matches kitchn layout.toml format

log_info() {
    local scope="$1"
    local msg="$2"
    echo -e "${LOG_CYAN}[i]${LOG_NC} ${scope}  ${msg}"
}

log_ok() {
    local scope="$1"
    local msg="$2"
    echo -e "${LOG_GREEN}[o]${LOG_NC} ${scope}  ${msg}"
}

log_warn() {
    local scope="$1"
    local msg="$2"
    echo -e "${LOG_YELLOW}[w]${LOG_NC} ${scope}  ${msg}" >&2
}

log_error() {
    local scope="$1"
    local msg="$2"
    echo -e "${LOG_RED}[f]${LOG_NC} ${scope}  ${msg}" >&2
}

log_debug() {
    local scope="$1"
    local msg="$2"
    if [[ -n "${VERBOSE:-}" ]]; then
        echo -e "${LOG_PURPLE}[d]${LOG_NC} ${scope}  ${msg}"
    fi
}

log_step() {
    local scope="$1"
    local msg="$2"
    echo -e "${LOG_PURPLE}[s]${LOG_NC} ${scope}  ${msg}"
}

# Die with error message
die() {
    log_error "$1" "$2"
    exit 1
}
