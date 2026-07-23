# harness-eval

A comparative evaluation of AI coding-agent harnesses: the same model
(Claude Opus 4.8), the same prompt, the same project — run once under
each of five harnesses and once with no harness per runner (GitHub
Copilot CLI and Claude Code CLI), graded by two independent AI reviewers
with a consensus pass. Three runs so far: a small greenfield CLI
(run 1), a multi-binary Rust service plus a fix-your-own-bug round
(run 02 + 02.1), and a **model-capability axis** re-running run 1's task
on a weaker model (run 03).

> [!IMPORTANT]
> **Preliminary: N = 1 per cell.** Run 02 measured single-run variance
> directly — the same cell, same frozen prompt, ran twice and spread
> 44% in wall clock *and* landed on opposite sides of the decisive
> correctness check. Cross-cutting patterns (the shared bug, the fix
> round's 4-for-4 direction) are findings; point-level rankings are
> weather. Reps accumulate as budget allows.

## Results at a glance — run 03: the model-capability axis (2026-07-23)

**Does harness value grow as the model gets weaker?** Run 03 re-runs
run 1's frozen loglens spec with the same seven cells and the same
harness versions, changing **one variable: the model** (Claude Haiku 4.5).
Each harness is scored against its own same-runner control, and the
cross-tier statistic is the **harness-minus-control delta** — raw totals
are not comparable across capability tiers.

| harness | delta at frontier | delta at Haiku 4.5 | shift |
|---|---:|---:|---:|
| working-skill-repo (KB) | 0 | **+15.25** | **+15.25** |
| kprojects | +0.5 | **+7.75** | +7.25 |
| ATV-StarterKit | −4 | −2.75 | +1.25 |
| gstack | −8 | −9.25 | −1.25 |
| ATV-Phoenix | −3 | **−12** | −9 |

**Yes — but it splits by harness weight.** Harnesses that *encode
knowledge* (conventions, skills, review habits) supply what a weaker
model lacks and gained sharply. Harnesses that *demand the model drive
an autonomous process* need capability it doesn't have, so the process
becomes overhead it must service. Actionable version: **the weaker your
model, the more you want conventions and the less you want ceremony.**

This tier's own field (not comparable point-for-point to other runs):
KB 77.5 · kprojects 70 · bare Claude control 66 · bare Copilot control
62.25 · StarterKit 59.5 · gstack 56.75 · Phoenix 50.25. Only one repo
passed all 12 core acceptance checks, where the frontier field went
7-for-7 — and the tier's signature failure was universal: **every repo
solved exactly half the naive/aware timezone problem**, in
complementary halves.

## Results — run 02 + fix round 02.1 (2026-07-17)

**Build** — feedhub, a three-binary Rust feed-aggregation service
(REST + SQLite + RSS/Atom + hermetic fixture server), headless, under an
executable sealed acceptance suite (26 tests, core + hard tiers):

| rank | harness | repo | runner | score | acceptance | wall clock |
|---|---|---|---|---:|---:|---:|
| 1 | working-skill-repo (KB) | [`03`](run-output/run_02/03-working-skill-repo/) | Copilot CLI | **93.5** | **26/26** | 18m 01s |
| 2 | none (control) | [`07`](run-output/run_02/07-baseline-claude/) | Claude Code | 84.5 | 24/26 | 36m 09s |
| 3 | gstack | [`06`](run-output/run_02/06-gstack/) | Claude Code | 82.5 | 24/26 | 48m 11s |
| 4 | none (control) | [`05`](run-output/run_02/05-baseline/) | Copilot CLI | 77.5 | 24/26 | 19m 35s |
| 5 | kprojects | [`04`](run-output/run_02/04-kprojects/) | Copilot CLI | 75 | 24/26 | 20m 38s |
| 6 | ATV-Phoenix | [`02`](run-output/run_02/02-atv-phoenix/) | Copilot CLI | 73.5 | 24/26 | 17m 16s |
| 7 | ATV-StarterKit 2.6.3 | [`01`](run-output/run_02/01-atv-starterkit/) | Copilot CLI | 71 | 24/26 | 16m 22s |

Six of seven builds shipped the **same bug** — quick-xml's streaming
EOF read as success, so truncated feeds parsed as valid-and-empty. The
two implementations that chose strict parsing (03, plus the ungraded
shakedown rep) were immune by construction: the eval's sharpest
discriminator was a dependency default. **Fix round (02.1)**: each
failing repo got a bug report against its own code — all six fixed it
completely (26/26 + a 3-test sealed addendum), five with literally the
same fix, and **every harness beat its same-runner control** on the
fix-delta rubric (gstack 99 > bare Claude 98; kprojects 96, StarterKit
95, Phoenix 94.5 > bare Copilot 94). Machinery that read as ceremony on
greenfield turned into unprompted regression tests on the resume task.

## Results — runs 1 + 1.5 (2026-07-15, small greenfield CLI)

| rank | harness | repo | runner | score | acceptance | cost† | wall clock |
|---|---|---|---|---:|---:|---:|---:|
| 1 | none (baseline control) | [`07-baseline-claude`](run-output/run_01/07-baseline-claude/) | Claude Code | 95 | 12/12 | $3.61 | 8m 38s |
| 2 | kprojects | [`04-kprojects`](run-output/run_01/04-kprojects/) | Copilot CLI | 92.5 | 12/12 | 252 cr | 6m 52s |
| 3= | working-skill-repo (KB) | [`03-working-skill-repo`](run-output/run_01/03-working-skill-repo/) | Copilot CLI | 92 | 12/12 | 216 cr | 6m 40s |
| 3= | none (baseline control) | [`05-baseline`](run-output/run_01/05-baseline/) | Copilot CLI | 92 | 12/12 | 143 cr | 4m 32s |
| 5 | ATV-Phoenix | [`02-atv-phoenix`](run-output/run_01/02-atv-phoenix/) | Copilot CLI | 89 | 12/12 | 265 cr | 7m 12s |
| 6 | ATV-StarterKit 2.x | [`01-atv-starterkit`](run-output/run_01/01-atv-starterkit/) | Copilot CLI | 88 | 12/12 | 440 cr | 10m 54s |
| 7 | gstack | [`06-gstack`](run-output/run_01/06-gstack/) | Claude Code | 87 | 12/12 | $10.84 | 19m 05s |

† Cost units differ by runner (Copilot AI credits vs Claude Code `/cost`
dollars) and are **not comparable across runners** — compare each entry to
its own runner's control (05 for Copilot, 07 for Claude Code).

Every agent shipped a fully working project (12/12 objective acceptance
checks; 7 × 12/12 after consensus adjudication). Harness machinery cost
1.5×–3.1× its runner's baseline without buying additional correctness —
the differences showed up in tests, docs, process artifacts, and
efficiency instead. Run 1.5's twist: the biggest quality delta in the eval
came from the *runner*, not any harness — the bare Claude Code control
(07) topped the field, and gstack (06) landed below its own baseline after
spending 3× as much for robustness margins the spec never asked for.
05-vs-07 isolates the runner effect; 06-vs-07 isolates gstack's
contribution — see the white paper's "Runner effect" section.

## Read the full story

### Run 03 — model-capability axis

- 📄 [White paper](_eval/run_03/report/whitepaper.md) — the cross-tier statistic, the tier signature, threats
- 🏁 [Final grades](_eval/run_03/grades/final.md) — consensus, deltas vs control, zero reconciliations
- 📊 [Infographic](_eval/run_03/report/infographic.html) ([rendered preview](https://htmlpreview.github.io/?https://github.com/kenhia/harness-eval/blob/main/_eval/run_03/report/infographic.html))
- 🧭 [Lessons learned](_eval/run_03/report/lessons-learned.md) — lessons 34–44
- 🐛 [Defect log](_eval/run_03/DEFECTS.md) — S1/S2 suite defects and the I1 interrupted run

### Run 02 + fix round 02.1

- 📄 [White paper](_eval/run_02/report/whitepaper.md) — method, both rounds, the dependency finding, variance, threats
- 🏁 [Final grades](_eval/run_02/grades/final.md) — build + fix consensus, zero reconciliations
- 📊 [Infographic](_eval/run_02/report/infographic.html) ([rendered preview](https://htmlpreview.github.io/?https://github.com/kenhia/harness-eval/blob/main/_eval/run_02/report/infographic.html))
- 🧭 [Lessons learned](_eval/run_02/report/lessons-learned.md) — lessons 22–33, input for eval v3

### Runs 1 + 1.5

- 📄 [White paper](_eval/run_01/report/whitepaper.md) — method, results, per-harness narratives, runner effect, threats to validity
- 🏁 [Final grades](_eval/run_01/grades/final.md) — consensus scores, per-grader raw scores, reconciliation notes
- 📊 [Infographic](_eval/run_01/report/infographic.html) ([rendered preview](https://htmlpreview.github.io/?https://github.com/kenhia/harness-eval/blob/main/_eval/run_01/report/infographic.html))
- 🧭 [Lessons learned](_eval/run_01/report/lessons-learned.md) — what runs 1 and 1.5 taught us about building the eval harness itself

## How it worked

Run 02 upgraded the method (details in its white paper): **executable
sealed acceptance** (hermetic pytest, core/hard tiers + fix addendum)
replaced the prose checklist and grader-authored fixtures — grader
reconciliations went from one per round to zero; runs went **headless**
with fake-HOME profiles for both runners; run logs, timing, tokens, and
versions are captured mechanically (`_eval/bin/`). Run 1 worked as
follows:

Seven clean repos, one harness installed per repo (committed, then tagged
`pre-run` so agent-authored work is cleanly diffable); the two controls got
no install commit. Each agent got a byte-identical project spec — the
[loglens CLI](_eval/run_01/prompts/00-project-spec.md), a Combined-Log-Format
access-log analyzer — with only the harness's own "go" command varying on
the first line ([prompts](_eval/run_01/prompts/)). Runs were hands-off (zero
interventions, all seven). Runs 01–05 ran under GitHub Copilot CLI (run 1);
runs 06–07 under Claude Code CLI (run 1.5), because gstack is
Claude-Code-native — 07 is the control that anchors that runner. Grading:
two independent reviewers (Claude Fable, GPT Sol) each ran a sealed
12-check [acceptance pass](_eval/run_01/acceptance.md) plus a 7-dimension weighted
[rubric](_eval/run_01/rubric.md), followed by a consensus session that adjudicated
factual disputes and reconciled score gaps. Run 1.5 was graded as a frozen
delta: new grader sessions, prior scores never reopened
([process](_eval/ADDING-A-HARNESS.md)). Full design:
[`_eval/README.md`](_eval/README.md).

## Repo map

```
run-output/run_NN/0N-<harness>/  final state of each run (what the agent built)
_eval/README.md                  eval design: isolation, run protocol
_eval/ADDING-A-HARNESS.md        how contenders are added incrementally
_eval/bin/                       run tooling (new-run.sh, run-eval.sh, collect-session.py)
_eval/templates/                 run-log template
_eval/run_NN/                    everything specific to one run of the eval:
  prompts/                         project spec + the per-harness prompts
  rubric.md                        grading dimensions, weights, grader instructions
  acceptance.md                    the objective checks (sealed from the agents)
  grades/                          both graders' sheets, adjudication, final consensus
  report/                          white paper, infographic, lessons learned
  runs/                            run logs (timing, tokens, observations) + transcripts
  grader_prompts/                  the prompts that drove the grading sessions
sprints/                         project evolution + sprints/planning/roadmap.md
```

(Run 1 and its 1.5 expansion share `run_01/` — one comparable field.)

### Per-run git history

`main` holds each run's final tree. The complete commit-by-commit story of
every run is preserved on namespaced refs:

- branches `history/0N-<harness>` — the run repo's full history
- tags `pre-run/0N-<harness>` — the boundary before the agent started

so `git diff pre-run/04-kprojects..history/04-kprojects` shows exactly what
that agent authored. (Run 02 onward namespaces by run group:
`history/run_NN/0N-<harness>`, `pre-run/run_NN/0N-<harness>`.)

## Caveats (the honest list)

- N = 1 per cell — and run 02 *measured* the consequence: same cell,
  same prompt, 44% wall-clock spread and opposite outcomes on the
  decisive correctness check across two reps.
- Two runners: cost units aren't comparable across them — and Copilot's
  credit unit itself drifted ~5× between run 1 and run 02 CLI versions
  (premium requests and wall clock travel best).
- Auto-updating CLIs mean runner versions drift, including mid-field
  (recorded per run log).
- The eval author is also the author of one contender (kprojects).
- Harness artifacts self-identify, so grader blinding is imperfect.
- Run 1: klams/korg MCP available to all runs equally; run 02.1: ambient
  MCP uniformly absent on Copilot cells (approval-gate incident E1 —
  see `_eval/run_02/FIX-ROUND.md`), Claude cells kept theirs.
- ATV-StarterKit 2.x bundled gstack in run 1 (01/06 shared DNA); 2.6.3
  no longer vendors it.
- Scores are calibrated **within** a run; totals from different runs
  (and especially different capability tiers) are not comparable
  point-for-point. Cross-tier claims use harness-minus-control deltas.

See each run's white paper *threats to validity* for the full treatment.

## License

MIT — see [LICENSE](LICENSE).
