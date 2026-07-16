#!/usr/bin/env bash
# run-acceptance.sh — run a run-group's acceptance suite against one repo,
# archiving the full output to _eval/<run-group>/runs/NN-acceptance.txt.
#
# Usage:
#   run-acceptance.sh <run-group> <NN-name | /abs/path/to/repo>
#
# Example:
#   run-acceptance.sh run_02 05-baseline
set -euo pipefail

EVAL_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STAGING_ROOT="$HOME/src/ai-agents/harness-eval-runs"

run_group="${1:?usage: run-acceptance.sh <run-group> <NN-name|path> [--fix]}"
repo="${2:?usage: run-acceptance.sh <run-group> <NN-name|path> [--fix]}"
fix_round=""
[[ "${3:-}" == "--fix" ]] && fix_round=1

SUITE="$EVAL_ROOT/$run_group/acceptance"
[[ -d $SUITE ]] || { echo "no acceptance suite at $SUITE" >&2; exit 1; }
REPO_DIR="$repo"
[[ -d $REPO_DIR ]] || REPO_DIR="$STAGING_ROOT/$run_group/$repo"
[[ -d $REPO_DIR ]] || { echo "no repo at $repo or $REPO_DIR" >&2; exit 1; }
RUN_NAME="$(basename "$REPO_DIR")"
OUT_DIR="$EVAL_ROOT/$run_group/runs"
mkdir -p "$OUT_DIR"
OUT="$OUT_DIR/${RUN_NAME%%-*}${fix_round:+-fix}-acceptance.txt"

{
    echo "# acceptance: $run_group / $RUN_NAME"
    echo "# repo: $REPO_DIR @ $(git -C "$REPO_DIR" rev-parse --short HEAD 2>/dev/null || echo '?')"
    echo "# date: $(date -Is)"
    echo
} > "$OUT"

set +e
FEEDHUB_REPO="$REPO_DIR" ACCEPTANCE_REPO="$REPO_DIR" FIX_ROUND="$fix_round" \
    uv run --with pytest pytest "$SUITE" -v 2>&1 | tee -a "$OUT"
rc=${PIPESTATUS[0]}
set -e

{
    echo
    echo "# exit code: $rc"
    echo "# tier summary:"
    for tier in core hard ${fix_round:+fix}; do
        tot=$(grep -cE "test_${tier}\.py::.*(PASSED|FAILED|ERROR)" "$OUT" || true)
        pass=$(grep -cE "test_${tier}\.py::.*PASSED" "$OUT" || true)
        echo "#   $tier: $pass/$tot"
    done
} >> "$OUT"
tail -4 "$OUT"
echo "archived: $OUT"
