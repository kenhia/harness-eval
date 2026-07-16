# run_02 — complex greenfield: feedhub (STATUS: setup)

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

## Remaining before the field runs (in order)

1. **Harness install refresh** — per ADDING-A-HARNESS §0–§3: re-install
   each harness into its staging repo under current versions, commit,
   tag `pre-run`; re-verify profiles (incl. Phoenix's global piece in the
   `phoenix` profile, gstack's in `claude-gstack`), doctor checks, no
   real-HOME leakage. Record harness versions in the install commits.
2. **Dry-run shakedown** — one throwaway control run end-to-end
   (`run-eval.sh --headless` on a copy of 07's setup), then run the
   acceptance suite against the result. This validates the suite itself
   (it has not yet met a real implementation) and gives a wall-clock/cost
   calibration for the Rust task. Suite bugs found here are process
   fixes; log them below.
3. **Freeze** — spec + acceptance freeze when the first real contender
   runs.
4. **Execute** — headless, hands-off, one cell at a time via
   `run-eval.sh`; acceptance output archived to `runs/NN-acceptance.txt`.

Note for Rust runs: `run-eval.sh` passes `CARGO_HOME`/`RUSTUP_HOME`
through to the real installs — rustup breaks under a bare fake-HOME
(verified during setup).

## Shakedown log

(none yet)
