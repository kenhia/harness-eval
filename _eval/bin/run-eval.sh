#!/usr/bin/env bash
# run-eval.sh — preflight, launch, time, and log one eval run.
#
# Wraps a runner (Claude Code or Copilot CLI) in a fake-HOME profile,
# records timestamps mechanically, and auto-fills the run log's
# "Auto-captured" section from the session logs afterward.
#
# Usage:
#   run-eval.sh --runner claude|copilot --profile <name> \
#               --run-group run_NN --repo <NN-name> \
#               [--model <id>] [--prompt-file <path>] [--headless]
#
# Examples:
#   # interactive Claude run (you paste the prompt, hands off after):
#   run-eval.sh --runner claude --profile claude-clean \
#     --run-group run_02 --repo 07-baseline-claude --model claude-opus-4-8
#
#   # headless Copilot run, prompt piped in:
#   run-eval.sh --runner copilot --profile clean \
#     --run-group run_02 --repo 05-baseline \
#     --prompt-file _eval/run_02/prompts/05-baseline.md --headless
#
# Notes:
# - Interactive Copilot runs: select the model with /model in the TUI
#   (recorded from the session log afterward either way).
# - Interactive Claude runs: paste `/cost` output into the Manual section
#   before exiting; everything else is captured automatically.
set -euo pipefail

EVAL_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STAGING_ROOT="$HOME/src/ai-agents/harness-eval-runs"

runner="" profile="" run_group="" repo="" model="" prompt_file="" headless=false
while [[ $# -gt 0 ]]; do
    case "$1" in
        --runner)      runner="$2"; shift 2 ;;
        --profile)     profile="$2"; shift 2 ;;
        --run-group)   run_group="$2"; shift 2 ;;
        --repo)        repo="$2"; shift 2 ;;
        --model)       model="$2"; shift 2 ;;
        --prompt-file) prompt_file="$2"; shift 2 ;;
        --headless)    headless=true; shift ;;
        *) echo "unknown arg: $1" >&2; exit 1 ;;
    esac
done
[[ -n $runner && -n $profile && -n $run_group && -n $repo ]] ||
    { grep '^#' "$0" | head -30 >&2; exit 1; }
[[ $runner == claude || $runner == copilot ]] || { echo "runner must be claude|copilot" >&2; exit 1; }

PROFILE_DIR="$EVAL_ROOT/profiles/$profile"
REPO_DIR="$repo"
[[ -d $REPO_DIR ]] || REPO_DIR="$STAGING_ROOT/$run_group/$repo"
RUN_NAME="$(basename "$REPO_DIR")"
RUNLOG_DIR="$EVAL_ROOT/$run_group/runs"
RUNLOG="$RUNLOG_DIR/${RUN_NAME%%-*}-runlog.md"   # NN-runlog.md

fail() { echo "PREFLIGHT FAIL: $*" >&2; exit 1; }
note() { echo "  $*"; }

# ---------- preflight ----------
echo "== preflight =="
[[ -d $PROFILE_DIR ]] || fail "no profile at $PROFILE_DIR"
if [[ $runner == claude ]]; then
    [[ -f $PROFILE_DIR/.claude/.credentials.json ]] || fail "profile missing .claude/.credentials.json"
    [[ -f $PROFILE_DIR/.claude.json ]] || fail "profile missing .claude.json at profile ROOT (HOME-override reads \$HOME/.claude.json)"
    [[ -f $PROFILE_DIR/.claude/settings.json ]] || fail "profile missing .claude/settings.json (needs bypassPermissions for hands-off runs)"
else
    [[ -d $PROFILE_DIR/.copilot ]] || fail "profile missing .copilot/"
fi
note "profile: $PROFILE_DIR ok"

[[ -d $REPO_DIR/.git ]] || fail "$REPO_DIR is not a git repo (new-run.sh creates staging repos)"
git -C "$REPO_DIR" rev-parse -q --verify pre-run >/dev/null || fail "no pre-run tag in $REPO_DIR"
[[ -z "$(git -C "$REPO_DIR" status --porcelain)" ]] || fail "working tree not clean in $REPO_DIR"
if [[ "$(git -C "$REPO_DIR" rev-parse HEAD)" != "$(git -C "$REPO_DIR" rev-parse pre-run^{commit})" ]]; then
    echo "  WARNING: HEAD != pre-run (re-run on a used repo?)"
fi
note "repo: $REPO_DIR ok (pre-run tagged, clean)"

runner_version="$(env HOME="$PROFILE_DIR" "$runner" --version 2>/dev/null | head -1)"
note "runner: $runner $runner_version"

[[ -d $RUNLOG_DIR ]] || mkdir -p "$RUNLOG_DIR"
[[ -e $RUNLOG ]] && fail "$RUNLOG already exists — move it aside first"

if [[ -n $prompt_file ]]; then
    [[ -f $prompt_file ]] || fail "no prompt file: $prompt_file"
fi
$headless && [[ -z $prompt_file ]] && fail "--headless requires --prompt-file"

# stamp for finding new session files + real-HOME leak canary
STAMP="$(mktemp)"; trap 'rm -f "$STAMP"' EXIT

# ---------- launch ----------
# Toolchain passthrough: rustup/cargo resolve toolchains via $HOME, which a
# fake-HOME profile breaks. Point them at the real installs (shared build
# cache is fine — it's the same class of shared resource as network access).
toolchain_env=()
[[ -d $HOME/.cargo ]] && toolchain_env+=(CARGO_HOME="$HOME/.cargo")
[[ -d $HOME/.rustup ]] && toolchain_env+=(RUSTUP_HOME="$HOME/.rustup")
launch=(env HOME="$PROFILE_DIR" "${toolchain_env[@]}" "$runner")
if [[ $runner == claude ]]; then
    [[ -n $model ]] && launch+=(--model "$model")
    $headless && launch+=(-p "$(cat "$prompt_file")")
else
    [[ -n $model ]] && launch+=(--model "$model")
    $headless && launch+=(-p "$(cat "$prompt_file")" --allow-all-tools)
fi

echo
echo "== launch =="
echo "  cd $REPO_DIR"
echo "  ${launch[*]:0:4} ..."
if ! $headless && [[ -n $prompt_file ]]; then
    echo "  --> paste the FULL contents of: $prompt_file"
fi
echo

start_iso="$(date -Is)"; start_s=$SECONDS
set +e
( cd "$REPO_DIR" && "${launch[@]}" )
runner_exit=$?
set -e
end_iso="$(date -Is)"
wall=$(( SECONDS - start_s ))
wall_h="$(printf '%dm %02ds' $((wall / 60)) $((wall % 60)))"

# ---------- collect ----------
echo
echo "== collect =="
session_metrics="(no new session files found — collect by hand with collect-session.py)"
if [[ $runner == claude ]]; then
    mapfile -t new_sessions < <(find "$PROFILE_DIR/.claude/projects" -name '*.jsonl' -newer "$STAMP" 2>/dev/null | sort)
else
    mapfile -t new_sessions < <(find "$PROFILE_DIR/.copilot/session-state" -name 'events.jsonl' -newer "$STAMP" 2>/dev/null | sort)
fi
if [[ ${#new_sessions[@]} -gt 0 ]]; then
    session_metrics="$(python3 "$EVAL_ROOT/bin/collect-session.py" --runner "$runner" "${new_sessions[@]}" 2>&1 || true)"
fi

diffstat="$(git -C "$REPO_DIR" diff --stat pre-run..HEAD 2>/dev/null | tail -1)"
committed=yes
[[ -z "$(git -C "$REPO_DIR" status --porcelain)" ]] || committed="NO — uncommitted changes present"

leak=""
for real in "$HOME/.claude" "$HOME/.copilot"; do
    [[ -d $real ]] || continue
    n="$(find "$real" -newer "$STAMP" -type f 2>/dev/null | wc -l)"
    [[ $n -gt 0 ]] && leak+="$real: $n file(s) touched during run; "
done
[[ -n $leak ]] && echo "  NOTE: real-HOME activity during run ($leak) — fine if you had other sessions open, otherwise investigate"

# ---------- write runlog ----------
sed "s/NN-<repo>/$RUN_NAME/" "$EVAL_ROOT/templates/runlog-template.md" > "$RUNLOG"
auto_block="$(cat <<EOF
- date: $(date +%F)
- runner: $runner ($runner_version) on $(hostname)
- model (requested): ${model:-"(interactive selection — see session metrics)"}
- profile used: $profile (HOME-sandbox: $PROFILE_DIR)
- launch: (cd $REPO_DIR && ${launch[*]:0:3} ...)
- headless: $headless
- start time: $start_iso
- end time: $end_iso
- wall clock: $wall_h
- runner exit code: $runner_exit
- working tree clean after run: $committed
- git diff --stat pre-run..HEAD: ${diffstat:-"(empty)"}
- real-HOME leak canary: ${leak:-clean}

### Session metrics ($runner session log)

$session_metrics
EOF
)"
AUTO_BLOCK="$auto_block" python3 - "$RUNLOG" <<'PYEOF'
import os, sys
path = sys.argv[1]
text = open(path).read()
marker = '## Auto-captured (filled by `_eval/bin/run-eval.sh` — do not hand-edit)'
head, sep, rest = text.partition(marker)
if not sep:
    sys.exit(f"runlog template marker not found in {path}")
comment_end = rest.find('-->') + 3   # drop the fill-by-hand comment
auto = os.environ['AUTO_BLOCK']
open(path, 'w').write(head + marker + '\n\n' + auto + '\n\n' + rest[comment_end:].lstrip('\n'))
PYEOF

echo "  run log written: $RUNLOG"
echo
echo "== yours to fill (Manual section) =="
echo "  - agent declared done / final state committed"
echo "  - /cost paste (interactive Claude runs only)"
echo "  - interventions (verbatim) + observations"
exit $runner_exit
