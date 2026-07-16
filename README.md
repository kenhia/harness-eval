# harness-eval

A comparative evaluation of AI coding-agent harnesses: the same model
(Claude Opus 4.8), the same prompt, the same greenfield project — run once
under each of five harnesses and twice with no harness at all, across two
runners (GitHub Copilot CLI and Claude Code CLI), then graded by two
independent AI reviewers with a consensus pass.

> [!IMPORTANT]
> **These results are from an initial exploration / proof of concept.**
> What was tested is deliberately narrow: **one run per harness (n = 1)**,
> each given a **single command** to build a **single small greenfield
> project**. The spreads on a 100-point scale are inside the noise floor —
> do not over-read the ranking. I expect higher separation as the
> evaluation harness improves and covers a wider scope of tasks (bug-fix
> sessions, refactors, resuming half-finished work, repeated trials).

## Results at a glance — runs 1 + 1.5

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

**Read the full story:**

- 📄 [White paper](_eval/run_01/report/whitepaper.md) — method, results, per-harness narratives, runner effect, threats to validity
- 🏁 [Final grades](_eval/run_01/grades/final.md) — consensus scores, per-grader raw scores, reconciliation notes
- 📊 [Infographic](_eval/run_01/report/infographic.html) ([rendered preview](https://htmlpreview.github.io/?https://github.com/kenhia/harness-eval/blob/main/_eval/run_01/report/infographic.html))
- 🧭 [Lessons learned](_eval/run_01/report/lessons-learned.md) — what runs 1 and 1.5 taught us about building the eval harness itself

## How it worked

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

- n = 1 per harness; single task type; single model.
- Two runners: cost units aren't comparable across them, and cross-runner
  score comparisons carry the measured runner effect (05 vs 07: +3).
- ATV-StarterKit bundles gstack, so runs 01 and 06 share DNA.
- The eval author is also the author of one contender (kprojects).
- Harness artifacts self-identify, so grader blinding is imperfect.
- Graders shared the machine environment (klams/korg MCP available to all
  runs equally).

See the white paper's *threats to validity* section for the full treatment.

## License

MIT — see [LICENSE](LICENSE).
