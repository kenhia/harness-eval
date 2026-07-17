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
boundary_tag="pre-run" log_suffix="" inject_gh_token=false
while [[ $# -gt 0 ]]; do
    case "$1" in
        --runner)      runner="$2"; shift 2 ;;
        --profile)     profile="$2"; shift 2 ;;
        --run-group)   run_group="$2"; shift 2 ;;
        --repo)        repo="$2"; shift 2 ;;
        --model)       model="$2"; shift 2 ;;
        --prompt-file) prompt_file="$2"; shift 2 ;;
        --headless)    headless=true; shift ;;
        --tag)         boundary_tag="$2"; shift 2 ;;
        --suffix)      log_suffix="-$2"; shift 2 ;;
        --inject-gh-token) inject_gh_token=true; shift ;;
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
RUNLOG="$RUNLOG_DIR/${RUN_NAME%%-*}${log_suffix}-runlog.md"   # NN[-suffix]-runlog.md

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
git -C "$REPO_DIR" rev-parse -q --verify "$boundary_tag" >/dev/null || fail "no $boundary_tag tag in $REPO_DIR"
[[ -z "$(git -C "$REPO_DIR" status --porcelain)" ]] || fail "working tree not clean in $REPO_DIR"
if [[ "$(git -C "$REPO_DIR" rev-parse HEAD)" != "$(git -C "$REPO_DIR" rev-parse "$boundary_tag"^{commit})" ]]; then
    echo "  WARNING: HEAD != $boundary_tag (re-run on a used repo?)"
fi
note "repo: $REPO_DIR ok ($boundary_tag tagged, clean)"

runner_version="$(env HOME="$PROFILE_DIR" "$runner" --version 2>/dev/null | head -1)"
note "runner: $runner $runner_version"

# Auth check. Profile credentials are snapshots; OAuth refresh tokens
# rotate, so a profile that sat idle while other sessions refreshed the
# chain can expire unrecoverably. For Claude runners: verify with a cheap
# haiku call, re-sync from the real HOME once on failure.
if [[ $runner == claude ]]; then
    auth_probe() {
        env HOME="$PROFILE_DIR" claude --model claude-haiku-4-5-20251001 \
            -p "Reply with exactly: ok" </dev/null >/dev/null 2>&1
    }
    if ! auth_probe; then
        echo "  auth probe failed — re-syncing credentials from ~/.claude"
        cp "$HOME/.claude/.credentials.json" "$PROFILE_DIR/.claude/.credentials.json"
        auth_probe || fail "auth still failing after credential re-sync — log in on the real account and retry"
    fi
    note "auth: ok"
else
    # Copilot uses the profile's own login (do `env HOME=<profile> copilot`
    # then /login, once per profile). gh-token injection is a last resort:
    # since 2026-07-16 GitHub policy-blocks ALL MCP servers under gh-CLI
    # tokens, so an injected run loses klams/korg/harness MCP.
    if $inject_gh_token; then
        copilot_token="$(gh auth token 2>/dev/null)" || fail "gh auth token unavailable — run 'gh auth login' and retry"
        echo "  WARNING: --inject-gh-token — MCP servers will be BLOCKED BY POLICY under this token class"
    else
        note "auth: profile stored login (if launch fails instantly: env HOME=$PROFILE_DIR copilot -> /login)"
    fi
fi

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
# -u scrubs ambient token vars: VS Code integrated terminals export a
# short-lived COPILOT_GITHUB_TOKEN PAT that goes stale and hijacks auth.
launch=(env -u COPILOT_GITHUB_TOKEN -u GITHUB_TOKEN -u GH_TOKEN HOME="$PROFILE_DIR" "${toolchain_env[@]}")
[[ $runner == copilot ]] && $inject_gh_token && launch+=(COPILOT_GITHUB_TOKEN="$copilot_token")
launch+=("$runner")
if [[ $runner == claude ]]; then
    [[ -n $model ]] && launch+=(--model "$model")
    $headless && launch+=(-p "$(cat "$prompt_file")")
else
    [[ -n $model ]] && launch+=(--model "$model")
    $headless && launch+=(-p "$(cat "$prompt_file")" --allow-all)
fi

echo
echo "== launch =="
echo "  cd $REPO_DIR"
echo "  ${launch[*]:0:2} ..."   # never echo further elements (token may follow)
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

# Launch-class failure (auth, bad flags): runner died without ever starting
# a session — don't write a run log for a run that never happened.
if [[ $runner_exit -ne 0 && ${#new_sessions[@]} -eq 0 ]]; then
    echo "  runner exited $runner_exit before starting a session — no run log written; fix and re-run"
    exit "$runner_exit"
fi

diffstat="$(git -C "$REPO_DIR" diff --stat "$boundary_tag"..HEAD 2>/dev/null | tail -1)"
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
- launch: (cd $REPO_DIR && ${launch[*]:0:2} ... $runner)
- headless: $headless
- start time: $start_iso
- end time: $end_iso
- wall clock: $wall_h
- runner exit code: $runner_exit
- working tree clean after run: $committed
- git diff --stat $boundary_tag..HEAD: ${diffstat:-"(empty)"}
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
