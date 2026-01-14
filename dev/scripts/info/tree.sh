#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="${SCRIPT_DIR}/../../.."
source "${SCRIPT_DIR}/../lib/log.sh"

SCOPE="TREE"

cd "$PROJECT_ROOT"

log_info "$SCOPE" "=== starting tree.sh ==="
log_info "$SCOPE" "working directory: $(pwd)"
log_info "$SCOPE" "excluding: target, .git, .tmp, .claude"

log_step "$SCOPE" "detecting tree tool"

if command -v tree &>/dev/null; then
    log_ok "$SCOPE" "tree found"
    log_step "$SCOPE" "executing: tree -I 'target|.git|.tmp|.claude' --dirsfirst"
    echo ""
    tree -I 'target|.git|.tmp|.claude' --dirsfirst
    echo ""
    log_ok "$SCOPE" "tree display complete"
elif command -v eza &>/dev/null; then
    log_ok "$SCOPE" "eza found (tree fallback)"
    log_step "$SCOPE" "executing: eza --tree --icons -I 'target|.git|.tmp|.claude'"
    echo ""
    eza --tree --icons -I 'target|.git|.tmp|.claude'
    echo ""
    log_ok "$SCOPE" "tree display complete"
else
    log_warn "$SCOPE" "no tree tool found (tree, eza)"
    log_info "$SCOPE" "falling back to find"
    log_step "$SCOPE" "executing: find with pruning"
    echo ""
    find . -type d \( -name target -o -name .git -o -name .tmp -o -name .claude \) -prune -o -print | sort
    echo ""
    log_ok "$SCOPE" "tree display complete"
fi

log_info "$SCOPE" "=== tree.sh finished successfully ==="
