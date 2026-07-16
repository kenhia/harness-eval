#!/usr/bin/env bash
# new-run.sh — stamp out a staging repo for one eval run.
#
# Usage:
#   new-run.sh <run-group> <NN-name>              # harness run: stops before install
#   new-run.sh <run-group> <NN-name> --no-harness # control run: tags pre-run immediately
#
# Examples:
#   new-run.sh run_02 01-atv-starterkit
#   new-run.sh run_02 07-baseline-claude --no-harness
#
# Staging repos live OUTSIDE the eval repo (main's run dirs are flattened
# publish trees): ~/src/ai-agents/harness-eval-runs/<run-group>/<NN-name>
set -euo pipefail

STAGING_ROOT="$HOME/src/ai-agents/harness-eval-runs"

run_group="${1:?usage: new-run.sh <run-group> <NN-name> [--no-harness]}"
name="${2:?usage: new-run.sh <run-group> <NN-name> [--no-harness]}"
no_harness=false
[[ "${3:-}" == "--no-harness" ]] && no_harness=true

dir="$STAGING_ROOT/$run_group/$name"
[[ -e "$dir" ]] && { echo "refusing: $dir already exists" >&2; exit 1; }

mkdir -p "$dir"
git -C "$dir" init -b main -q
git -C "$dir" commit --allow-empty -q -m "eval: clean repo baseline"

if $no_harness; then
    git -C "$dir" tag pre-run
    echo "created $dir (control run — pre-run tagged, ready to run)"
else
    echo "created $dir"
    cat <<EOF
next steps:
  1. cd $dir
  2. install the harness (repo-local pieces only; global pieces go in a profile)
  3. git add -A && git commit -m "eval: install <harness> (<exact command>)"
  4. git tag pre-run
then launch with run-eval.sh.
EOF
fi
