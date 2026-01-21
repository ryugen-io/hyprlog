#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="${SCRIPT_DIR}/../../.."
source "${SCRIPT_DIR}/../lib/log.sh"

SCOPE="LOC"

cd "$PROJECT_ROOT"

log_info "$SCOPE" "=== starting loc.sh ==="
log_info "$SCOPE" "working directory: $(pwd)"
log_info "$SCOPE" "filtering: Rust files only"
log_info "$SCOPE" "excluding: target, .git, .tmp"

log_step "$SCOPE" "detecting line counting tool"

if command -v tokei &>/dev/null; then
    log_ok "$SCOPE" "tokei found"
    log_step "$SCOPE" "executing: tokei -t rust --files --exclude target --exclude .git --exclude .tmp"
    echo ""
    tokei -t rust --files --exclude target --exclude .git --exclude .tmp
    echo ""

    log_step "$SCOPE" "generating TOP 10 files by LOC"
    echo ""
    echo "=== TOP 10 Rust Files by Lines of Code ==="
    echo ""
    # Parse tokei output: extract file paths and their code lines
    tokei -t rust --files --exclude target --exclude .git --exclude .tmp -o json 2>/dev/null | \
        jq -r '.Rust.reports[]? | "\(.stats.code)\t\(.name)"' 2>/dev/null | \
        sort -rn | head -10 | \
        awk 'BEGIN {printf "%-8s %s\n", "LOC", "FILE"; printf "%-8s %s\n", "---", "----"} {printf "%-8s %s\n", $1, $2}'
    echo ""
    log_ok "$SCOPE" "line count complete"
    log_info "$SCOPE" "=== loc.sh finished successfully ==="
elif command -v cloc &>/dev/null; then
    log_ok "$SCOPE" "cloc found (tokei fallback)"
    log_step "$SCOPE" "executing: cloc --include-lang=Rust --by-file --exclude-dir=target,.git,.tmp ."
    echo ""
    cloc --include-lang=Rust --by-file --exclude-dir=target,.git,.tmp .
    echo ""

    log_step "$SCOPE" "generating TOP 10 files by LOC"
    echo ""
    echo "=== TOP 10 Rust Files by Lines of Code ==="
    echo ""
    # Parse cloc output for top 10
    cloc --include-lang=Rust --by-file --exclude-dir=target,.git,.tmp --csv --quiet . 2>/dev/null | \
        tail -n +2 | grep -v "^$" | \
        awk -F',' 'NR>1 && $2=="Rust" {print $5"\t"$2}' | \
        sort -rn | head -10 | \
        awk 'BEGIN {printf "%-8s %s\n", "LOC", "FILE"; printf "%-8s %s\n", "---", "----"} {printf "%-8s %s\n", $1, $2}'
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
