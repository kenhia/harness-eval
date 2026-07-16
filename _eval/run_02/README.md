# run_02 — complex greenfield: feedhub (STATUS: FROZEN — executing)

**Frozen 2026-07-16** (after shakedown validation): the spec
(`prompts/00-project-spec.md`), the prompt files, and the acceptance
suite (`acceptance/`) are immutable for the life of this field. Changes
past this point invalidate comparability; suite defects discovered
mid-field are recorded here and adjudicated, not silently patched.
Copy-paste run helpers: `.scratch/NN-run-cmds.txt` (gitignored).

The "do the heavy harnesses pull their weight?" run: **feedhub**, a Rust
workspace of three cooperating binaries (`feedd` REST server / `feedctl`
CLI client / `feedgen` fixture server) with SQLite storage and pinned
RSS/Atom edge semantics. Decisions locked 2026-07-16: Rust, SQLite pinned
in spec, all 7 cells from run 1, **headless** runs (recorded as a
covariate vs run 1's interactive mode).

## Contents

- `prompts/` — frozen project spec + per-harness prompts (only the "go"
  line differs; body frozen once the first contender runs)
- `acceptance.md` + `acceptance/` — **executable** sealed acceptance:
  pytest, black-box, hermetic; core tier C1–C13 (gate) + hard tier
  H1–H12 (spread)
- `rubric.md` — weights unchanged from run_01; correctness fed by the
  suite, efficiency anchored to same-runner control
- `grades/precedents.md` — inherited interpretations
- `runs/` — run logs (auto-filled by `run-eval.sh`) + acceptance output

Sealed materials (acceptance, grader prompts, grades) are never shown to
working agents.

## The 7 cells

| repo | harness | runner | profile | prompt |
|---|---|---|---|---|
| 01-atv-starterkit | ATV-StarterKit | Copilot CLI | `clean` | `/lfg …` |
| 02-atv-phoenix | ATV-Phoenix | Copilot CLI | `phoenix` | `/phoenix-goal …` |
| 03-working-skill-repo | working-skill-repo (KB) | Copilot CLI | `clean` | `kb-start …` |
| 04-kprojects | kprojects | Copilot CLI | `clean` | ambient |
| 05-baseline | none (control) | Copilot CLI | `clean` | raw prompt |
| 06-gstack | gstack | Claude Code | `claude-gstack` | `/autoplan …` |
| 07-baseline-claude | none (control) | Claude Code | `claude-clean` | raw prompt |

Staging repos created at `~/src/ai-agents/harness-eval-runs/run_02/`
(controls already `pre-run`-tagged; harness cells await installs).

## Harness install refresh — DONE 2026-07-16

All 7 staging repos at `pre-run`, clean trees, install commits record
exact commands + versions:

| repo | installed | version/source |
|---|---|---|
| 01-atv-starterkit | `npx atv-starterkit@latest init` | **2.6.3** (94 files) |
| 02-atv-phoenix | none in repo (global) | `phoenix` profile verified: 19 skills, phoenix + token-master agents, phoenix-mcp binary answers JSON-RPC initialize |
| 03-working-skill-repo | `kb-install.mjs --target repo --profile core --router skip` | source `Irtechie/working-skill-repo` @ **34804ea** (same commit as run 1; checkout at `~/src/ai-agents/working-skill-repo`) |
| 04-kprojects | `install.sh --agent both` | kprojects @ **b3c5af8** |
| 06-gstack | `gstack-team-init required` (run under profile HOME) | gstack **1.60.1.0** in `claude-gstack` profile |

Version-drift notes vs run 1:
- StarterKit 2.6.3 installs `.github` + `.vscode` only — **no vendored
  `.atv`/`.gstack` dirs** (run 1's 2.x had them). The "01/06 share DNA"
  caveat weakens to 3 textual gstack mentions; restate accordingly.
- Copilot profiles (`clean`, `phoenix`) gained a profile-root
  `.gitconfig` — required now that runs use fake-HOME (agents commit;
  run 1's symlink method used the real `~/.gitconfig`).
- Real-HOME leak check clean (no `~/.atv`, `~/.gstack`, `~/.kb`,
  `~/.agents`).

## Remaining before the field runs (in order)

1. **Dry-run shakedown** — one throwaway control run end-to-end
   (`run-eval.sh --headless` on a copy of 07's setup), then run the
   acceptance suite against the result. This validates the suite itself
   (it has not yet met a real implementation) and gives a wall-clock/cost
   calibration for the Rust task. Suite bugs found here are process
   fixes; log them below. Suggested:

   ```bash
   _eval/bin/new-run.sh run_02 99-shakedown --no-harness
   _eval/bin/run-eval.sh --runner claude --profile claude-clean \
     --run-group run_02 --repo 99-shakedown --model claude-opus-4-8 \
     --headless --prompt-file _eval/run_02/prompts/07-baseline-claude.md
   _eval/bin/run-acceptance.sh run_02 99-shakedown
   # then: delete 99-shakedown + its runlog; log findings below
   ```

2. **Freeze** — DONE 2026-07-16 (see header).
3. **Execute** (in progress) — headless, hands-off, one cell at a time
   via `run-eval.sh`, then `run-acceptance.sh` (archives output +
   core/hard tally to `runs/NN-acceptance.txt` automatically). Run cells
   serially — concurrent runs share account rate limits and muddy the
   real-HOME leak canary. Planned order: 06, 07 (tonight), then 01–05.

Note for Rust runs: `run-eval.sh` passes `CARGO_HOME`/`RUSTUP_HOME`
through to the real installs — rustup breaks under a bare fake-HOME
(verified during setup).

## Shakedown log

**2026-07-16 — shakedown run complete (99-shakedown, bare Claude Code).**

- Run mechanics: headless end-to-end worked; runlog auto-filled. Wall
  clock **52m 05s**, ~207k output tokens (vs 8m38s / 122.7k for the same
  cell on loglens) — the Rust task is ≈6× wall clock before harness
  overhead. Plan field execution accordingly (7 cells ≈ a full day of
  serial runs).
- Acceptance suite vs first real implementation: **25/26 first pass; the
  one failure was a suite bug, not an implementation bug.** H11 assumed
  `q=rust` would not match "C**rust**acean recipes", but the spec pins
  substring semantics — the implementation was right. Fixed: H11 now
  asserts Crustacean IS matched (deliberate substring probe). Exactly
  the fixture-vs-spec error class (run 1's A5) the executable suite
  exists to catch — and the pinned spec settled it mechanically. After
  fix: **core 14/14, hard 12/12**.
- Output capture wasn't automatic (Ken hit this) — added
  `_eval/bin/run-acceptance.sh`: runs the suite with `FEEDHUB_REPO` set,
  archives full output + tier tally to `runs/NN-acceptance.txt`.
- Calibration note: the bare-Claude control passed the full hard tier.
  Spread, if any, will come from the harness cells — or the hard tier
  needs to get harder in run 03+.
