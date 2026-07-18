#!/usr/bin/env bash
# run-matrix.sh — drive a run group's cells serially: run-eval.sh then
# run-acceptance.sh per cell, resumable (cells with an existing runlog
# are skipped).
#
# Usage:
#   run-matrix.sh <run-group> [cell ...]      # default: all cells in cells.tsv
#
# Cell manifest: _eval/<run-group>/cells.tsv, pipe-separated:
#   cell|runner|profile|model|prompt_file      (# comments allowed)
# Model is per-cell (the capability axis is a manifest edit, not a
# tooling change). Provider config (e.g. Copilot BYOK endpoints) is a
# PROFILE concern — configure it in the profile, reference the profile
# here.
set -euo pipefail

EVAL_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

run_group="${1:?usage: run-matrix.sh <run-group> [cell ...]}"; shift || true
MANIFEST="$EVAL_ROOT/$run_group/cells.tsv"
[[ -f $MANIFEST ]] || { echo "no manifest at $MANIFEST" >&2; exit 1; }

only=("$@")
want() {
    [[ ${#only[@]} -eq 0 ]] && return 0
    local c; for c in "${only[@]}"; do [[ "$1" == "$c"* ]] && return 0; done
    return 1
}

ran=0 skipped=0 failed=0
while IFS='|' read -r cell runner profile model prompt; do
    [[ -z $cell || $cell == \#* ]] && continue
    want "$cell" || continue
    runlog="$EVAL_ROOT/$run_group/runs/${cell%%-*}-runlog.md"
    if [[ -e $runlog ]]; then
        echo "== $cell: runlog exists, skipping (move it aside to re-run)"
        skipped=$((skipped + 1)); continue
    fi
    echo
    echo "==================== $cell ($runner/$profile, $model) ===================="
    if ! "$EVAL_ROOT/bin/run-eval.sh" --runner "$runner" --profile "$profile" \
            --run-group "$run_group" --repo "$cell" --model "$model" \
            --headless --prompt-file "$EVAL_ROOT/$run_group/prompts/$prompt"; then
        echo "== $cell: RUN FAILED — stopping the matrix (fix, then re-invoke to resume)"
        failed=1; break
    fi
    "$EVAL_ROOT/bin/run-acceptance.sh" "$run_group" "$cell" || true  # acceptance failures are data
    ran=$((ran + 1))
done < "$MANIFEST"

echo
echo "matrix: $ran ran, $skipped skipped$( [[ $failed -eq 1 ]] && echo ', STOPPED ON FAILURE' )"
exit $failed
