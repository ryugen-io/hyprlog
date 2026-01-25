#!/usr/bin/env bash
# shellcheck disable=SC2155
# =============================================================================
# hyprlog Install Script
# Builds and installs the hyprlog CLI and optionally the C-ABI library
#
# Usage:
#   From source (in repo):  ./install.sh
#   From release package:   ./install.sh
#   Remote install:         curl -fsSL https://raw.githubusercontent.com/ryugen-io/hyprlog/main/install.sh | bash
#   Specific version:       curl -fsSL ... | bash -s -- v0.1.0
#
# Installs:
#   CLI:    ~/.local/bin/hypr/hyprlog
#   C-ABI:  ~/.local/lib/libhyprlog.so (optional, source builds only)
#           ~/.local/include/hyprlog/hyprlog.h
# =============================================================================

set -euo pipefail
IFS=$'\n\t'

shopt -s inherit_errexit 2>/dev/null || true

# -----------------------------------------------------------------------------
# Configuration
# -----------------------------------------------------------------------------
readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" 2>/dev/null && pwd || echo "")"
readonly CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/hypr"
readonly CACHE_DIR="${XDG_CACHE_HOME:-$HOME/.cache}/hyprlog"
readonly STATE_DIR="${XDG_STATE_HOME:-$HOME/.local/state}/hyprlog"
readonly INSTALL_DIR="${HOME}/.local/bin/hypr"
readonly LIB_DIR="${HOME}/.local/lib"
readonly INCLUDE_DIR="${HOME}/.local/include"

# GitHub Release Settings
readonly REPO="ryugen-io/hyprlog"
readonly GITHUB_API="https://api.github.com/repos/${REPO}/releases"

# Installation mode: "source", "package", or "remote"
INSTALL_MODE=""

# -----------------------------------------------------------------------------
# Logging - Source shared lib or use inline fallback
# -----------------------------------------------------------------------------
if [[ -n "$SCRIPT_DIR" && -f "${SCRIPT_DIR}/dev/scripts/lib/log.sh" ]]; then
    # shellcheck source=dev/scripts/lib/log.sh
    source "${SCRIPT_DIR}/dev/scripts/lib/log.sh"
    log()     { log_info "INSTALL" "$*"; }
    success() { log_ok "INSTALL" "$*"; }
    warn()    { log_warn "INSTALL" "$*"; }
    error()   { log_error "INSTALL" "$*"; }
    die()     { log_error "INSTALL" "$*"; exit 1; }
    header()  { log_step "INSTALL" "$*"; }
else
    # Inline fallback (for remote install / extracted packages)
    readonly GREEN=$'\033[38;2;80;250;123m'
    readonly YELLOW=$'\033[38;2;241;250;140m'
    readonly CYAN=$'\033[38;2;139;233;253m'
    readonly RED=$'\033[38;2;255;85;85m'
    readonly PURPLE=$'\033[38;2;189;147;249m'
    readonly NC=$'\033[0m'

    log()     { echo -e "${CYAN}[info]${NC} INSTALL  $*"; }
    success() { echo -e "${GREEN}[ok]${NC}   INSTALL  $*"; }
    warn()    { echo -e "${YELLOW}[warn]${NC} INSTALL  $*" >&2; }
    error()   { echo -e "${RED}[error]${NC} INSTALL  $*" >&2; }
    die()     { error "$*"; exit 1; }
    header()  { echo -e "${PURPLE}[hyprlog]${NC} INSTALL  $*"; }
fi

# -----------------------------------------------------------------------------
# Cleanup & Signal Handling
# -----------------------------------------------------------------------------
cleanup() {
    local exit_code=$?
    exit "$exit_code"
}
trap cleanup EXIT
trap 'die "Interrupted"' INT TERM

# -----------------------------------------------------------------------------
# Utility Functions
# -----------------------------------------------------------------------------
command_exists() {
    command -v "$1" &>/dev/null
}

detect_arch() {
    local arch
    arch="$(uname -m)"
    case "$arch" in
        x86_64|amd64)   echo "x86_64-linux" ;;
        aarch64|arm64)  echo "aarch64-linux" ;;
        *)              die "Unsupported architecture: $arch" ;;
    esac
}

detect_install_mode() {
    if [[ -n "$SCRIPT_DIR" && -f "${SCRIPT_DIR}/Cargo.toml" ]]; then
        INSTALL_MODE="source"
    elif [[ -n "$SCRIPT_DIR" && -d "${SCRIPT_DIR}/bin" && -f "${SCRIPT_DIR}/bin/hyprlog" ]]; then
        INSTALL_MODE="package"
    else
        INSTALL_MODE="remote"
    fi
    log "Install mode: ${INSTALL_MODE}"
}

get_latest_release() {
    local url="${GITHUB_API}/latest"
    if command_exists curl; then
        curl -fsSL "$url" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/'
    elif command_exists wget; then
        wget -qO- "$url" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/'
    else
        die "Neither curl nor wget found"
    fi
}

download_release() {
    local version="$1"
    local arch="$2"
    local url="https://github.com/${REPO}/releases/download/${version}/hyprlog-${version}-${arch}.tar.gz"
    local tmp_dir
    tmp_dir="$(mktemp -d)"

    log "Downloading ${url}..."

    if command_exists curl; then
        curl -fsSL "$url" -o "${tmp_dir}/hyprlog.tar.gz" || die "Download failed"
    elif command_exists wget; then
        wget -q "$url" -O "${tmp_dir}/hyprlog.tar.gz" || die "Download failed"
    fi

    log "Extracting..."
    tar -xzf "${tmp_dir}/hyprlog.tar.gz" -C "$tmp_dir"

    local pkg_dir
    pkg_dir="$(find "$tmp_dir" -maxdepth 1 -type d -name 'hyprlog-*' | head -1)"

    if [[ -z "$pkg_dir" ]]; then
        die "Failed to extract release package"
    fi

    echo "$pkg_dir"
}

create_dir() {
    local dir="$1"
    if [[ ! -d "$dir" ]]; then
        mkdir -p "$dir" || die "Failed to create directory: $dir"
        success "Created $dir"
    else
        log "Directory exists: $dir"
    fi
}

# -----------------------------------------------------------------------------
# Installation Functions
# -----------------------------------------------------------------------------
install_from_source() {
    cd "$SCRIPT_DIR" || die "Failed to cd to script directory"

    if ! command_exists cargo; then
        die "Cargo not found. Install Rust: https://rustup.rs"
    fi

    log "Building release binary..."
    if ! cargo build --release --bin hyprlog 2>&1; then
        die "Build failed"
    fi
    success "Build complete"

    # Compact binary if UPX is available
    if command_exists upx; then
        log "Compacting binary with UPX..."
        compact_binary "target/release/hyprlog"
    fi

    # Install binary
    local src="target/release/hyprlog"
    [[ -f "$src" ]] && cp "$src" "$INSTALL_DIR/" || die "Binary not found: $src"
    chmod +x "${INSTALL_DIR}/hyprlog"
}

install_from_package() {
    local pkg_dir="$1"

    local src="${pkg_dir}/bin/hyprlog"
    [[ -f "$src" ]] && cp "$src" "$INSTALL_DIR/" || die "Binary not found: $src"
    chmod +x "${INSTALL_DIR}/hyprlog"
}

install_from_remote() {
    local version="${1:-}"
    local arch

    arch="$(detect_arch)"

    if [[ -z "$version" ]]; then
        log "Fetching latest release..."
        version="$(get_latest_release)"
    fi

    if [[ -z "$version" ]]; then
        die "Could not determine release version"
    fi

    log "Installing hyprlog ${version} for ${arch}"

    local pkg_dir
    pkg_dir="$(download_release "$version" "$arch")"

    install_from_package "$pkg_dir"

    rm -rf "$(dirname "$pkg_dir")"
}

compact_binary() {
    local bin="$1"
    if [[ -f "$bin" ]]; then
        local size_before=$(stat -c%s "$bin")
        upx --best --lzma --quiet "$bin" > /dev/null
        local size_after=$(stat -c%s "$bin")
        local saved=$(( size_before - size_after ))
        local percent=$(( (saved * 100) / size_before ))

        local size_before_fmt=$(numfmt --to=iec-i --suffix=B "$size_before")
        local size_after_fmt=$(numfmt --to=iec-i --suffix=B "$size_after")

        log "Optimized $(basename "$bin"): ${size_before_fmt} -> ${size_after_fmt} (-${percent}%)"
    fi
}

install_config() {
    local config_file="${CONFIG_DIR}/hyprlog.conf"

    if [[ -f "$config_file" ]]; then
        log "Config exists: $config_file"
        return
    fi

    # Try to find default config
    local default_config=""
    if [[ -n "$SCRIPT_DIR" && -f "${SCRIPT_DIR}/assets/hyprlog.conf" ]]; then
        default_config="${SCRIPT_DIR}/assets/hyprlog.conf"
    fi

    if [[ -n "$default_config" ]]; then
        cp "$default_config" "$config_file"
        success "Installed config: $config_file"
    else
        # Create minimal config
        cat > "$config_file" << 'CONF'
[general]
level = "info"
app_name = "hyprlog"

[terminal]
enabled = true
colors = true
icons = "nerdfont"
CONF
        success "Created default config: $config_file"
    fi
}

ask_yes_no() {
    local prompt="$1"
    local default="${2:-n}"
    local reply

    if [[ "$default" == "y" ]]; then
        prompt="$prompt [Y/n] "
    else
        prompt="$prompt [y/N] "
    fi

    read -r -p "$prompt" reply
    reply="${reply:-$default}"

    [[ "$reply" =~ ^[Yy]$ ]]
}

install_cabi() {
    # Only available when installing from source
    if [[ "$INSTALL_MODE" != "source" ]]; then
        return
    fi

    # Check if library exists
    local lib_src="${SCRIPT_DIR}/target/release/libhyprlog.so"
    local header_src="${SCRIPT_DIR}/include/hyprlog.h"

    if [[ ! -f "$lib_src" ]]; then
        log "Building C-ABI library..."
        cd "$SCRIPT_DIR" || die "Failed to cd to script directory"
        cargo build --release --features ffi 2>&1 || die "C-ABI build failed"
    fi

    if [[ ! -f "$lib_src" ]]; then
        warn "C-ABI library not found: $lib_src"
        return
    fi

    if [[ ! -f "$header_src" ]]; then
        warn "C-ABI header not found: $header_src"
        return
    fi

    # Create directories
    create_dir "$LIB_DIR"
    create_dir "${INCLUDE_DIR}/hyprlog"

    # Install library
    cp "$lib_src" "${LIB_DIR}/" || die "Failed to install library"
    success "Installed library: ${LIB_DIR}/libhyprlog.so"

    # Install header
    cp "$header_src" "${INCLUDE_DIR}/hyprlog/" || die "Failed to install header"
    success "Installed header: ${INCLUDE_DIR}/hyprlog/hyprlog.h"

    # ldconfig hint
    if [[ ":$LD_LIBRARY_PATH:" != *":$LIB_DIR:"* ]]; then
        warn "$LIB_DIR not in LD_LIBRARY_PATH"
        echo "  Add to shell config: export LD_LIBRARY_PATH=\"\$HOME/.local/lib:\$LD_LIBRARY_PATH\""
        echo "  Or run: sudo ldconfig $LIB_DIR"
    fi
}

# -----------------------------------------------------------------------------
# Main Installation
# -----------------------------------------------------------------------------
main() {
    local requested_version="${1:-}"

    header "starting installation"

    detect_install_mode

    # Create directories
    create_dir "$CONFIG_DIR"
    create_dir "$INSTALL_DIR"
    create_dir "$CACHE_DIR"
    create_dir "$STATE_DIR"
    create_dir "${STATE_DIR}/logs"

    # Install config
    install_config

    # Install based on mode
    case "$INSTALL_MODE" in
        source)
            install_from_source
            ;;
        package)
            install_from_package "$SCRIPT_DIR"
            ;;
        remote)
            install_from_remote "$requested_version"
            ;;
    esac

    success "Installed CLI to $INSTALL_DIR"

    # Ask about C-ABI installation (only for source builds)
    if [[ "$INSTALL_MODE" == "source" ]]; then
        echo ""
        if ask_yes_no "Install C-ABI library for C/C++ integration?"; then
            install_cabi
        fi
    fi

    # PATH check
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        warn "$INSTALL_DIR not in PATH"
        echo "  Add to config.fish: set -Ua fish_user_paths \$HOME/.local/bin/hypr"
    fi

    # Show installed version
    if command_exists "${INSTALL_DIR}/hyprlog"; then
        log "Installed version: $("${INSTALL_DIR}/hyprlog" --version 2>/dev/null || echo "unknown")"
    fi
}

main "$@"
