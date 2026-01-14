#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="${SCRIPT_DIR}/../../.."
source "${SCRIPT_DIR}/../lib/log.sh"

SCOPE="CHANGES"
MODE="${1:-}"

cd "$PROJECT_ROOT"

log_info "$SCOPE" "=== starting changes.sh ==="
log_info "$SCOPE" "working directory: $(pwd)"
log_info "$SCOPE" "mode: ${MODE:-summary}"

log_step "$SCOPE" "checking git repository"
if ! git rev-parse --git-dir &>/dev/null; then
    log_error "$SCOPE" "not a git repository"
    log_info "$SCOPE" "=== changes.sh failed ==="
    exit 1
fi
log_ok "$SCOPE" "git repository found"

BRANCH=$(git branch --show-current 2>/dev/null || echo "detached")
log_info "$SCOPE" "current branch: ${BRANCH}"

case "$MODE" in
    --staged)
        log_info "$SCOPE" "showing staged changes detail"
        log_step "$SCOPE" "executing: git diff --cached --stat"
        echo ""
        git diff --cached --stat
        echo ""
        log_info "$SCOPE" "=== changes.sh finished successfully ==="
        ;;
    --files)
        log_info "$SCOPE" "showing changed files only"
        log_step "$SCOPE" "executing: git status --short"
        echo ""
        git status --short
        echo ""
        log_info "$SCOPE" "=== changes.sh finished successfully ==="
        ;;
    --help|-h)
        echo "Usage: changes.sh [--staged|--files]"
        echo ""
        echo "  (none)   summary of all changes"
        echo "  --staged show staged changes detail"
        echo "  --files  list changed files only"
        exit 0
        ;;
    "")
        log_info "$SCOPE" "generating change summary"

        log_step "$SCOPE" "counting staged files"
        staged=$(git diff --cached --name-only | wc -l)
        log_info "$SCOPE" "staged: ${staged}"

        log_step "$SCOPE" "counting unstaged files"
        unstaged=$(git diff --name-only | wc -l)
        log_info "$SCOPE" "unstaged: ${unstaged}"

        log_step "$SCOPE" "counting untracked files"
        untracked=$(git ls-files --others --exclude-standard | wc -l)
        log_info "$SCOPE" "untracked: ${untracked}"

        echo ""
        if [[ $staged -gt 0 ]]; then
            log_ok "$SCOPE" "${staged} file(s) staged for commit"
        fi
        if [[ $unstaged -gt 0 ]]; then
            log_warn "$SCOPE" "${unstaged} file(s) modified but not staged"
        fi
        if [[ $untracked -gt 0 ]]; then
            log_info "$SCOPE" "${untracked} file(s) untracked"
        fi

        if [[ $staged -eq 0 && $unstaged -eq 0 && $untracked -eq 0 ]]; then
            log_ok "$SCOPE" "working tree is clean"
        else
            echo ""
            log_step "$SCOPE" "file list:"
            git status --short
        fi

        log_info "$SCOPE" "=== changes.sh finished successfully ==="
        ;;
    *)
        log_error "$SCOPE" "unknown flag: $MODE"
        echo ""
        echo "Usage: changes.sh [--staged|--files]"
        echo ""
        echo "  (none)   summary of all changes"
        echo "  --staged show staged changes detail"
        echo "  --files  list changed files only"
        exit 1
        ;;
esac
