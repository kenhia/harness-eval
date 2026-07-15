# harness-eval

A comparative evaluation of AI coding-agent harnesses: the same model
(Claude Opus 4.8 via GitHub Copilot CLI), the same prompt, the same
greenfield project — run once under each of four harnesses and once with no
harness at all, then graded by two independent AI reviewers with a
consensus pass.

> [!IMPORTANT]
> **These results are from an initial exploration / proof of concept.**
> What was tested is deliberately narrow: **one run per harness (n = 1)**,
> each given a **single command** to build a **single small greenfield
> project**. A 4.5-point spread on a 100-point scale is inside the noise
> floor — do not over-read the ranking. I expect higher separation as the
> evaluation harness improves and covers a wider scope of tasks (bug-fix
> sessions, refactors, resuming half-finished work, repeated trials).

## Results at a glance — POC run 1

| rank | harness | repo | score | acceptance | AI credits | wall clock |
|---|---|---|---:|---:|---:|---:|
| 1 | kprojects | [`04-kprojects`](04-kprojects/) | 92.5 | 12/12 | 252 | 6m 52s |
| 2= | working-skill-repo (KB) | [`03-working-skill-repo`](03-working-skill-repo/) | 92 | 12/12 | 216 | 6m 40s |
| 2= | none (baseline control) | [`05-baseline`](05-baseline/) | 92 | 12/12 | 143 | 4m 32s |
| 4 | ATV-Phoenix | [`02-atv-phoenix`](02-atv-phoenix/) | 89 | 12/12 | 265 | 7m 12s |
| 5 | ATV-StarterKit 2.x | [`01-atv-starterkit`](01-atv-starterkit/) | 88 | 12/12 | 440 | 10m 54s |

Every agent shipped a fully working project (12/12 objective acceptance
checks). On this small task, harness machinery cost 1.5×–3.1× the baseline's
credits without buying additional correctness — the differences showed up in
tests, docs, process artifacts, and efficiency instead.

**Read the full story:**

- 📄 [White paper](_eval/report/whitepaper.md) — method, results, per-harness narratives, threats to validity
- 🏁 [Final grades](_eval/grades/final.md) — consensus scores, per-grader raw scores, reconciliation notes
- 📊 [Infographic](_eval/report/infographic.html) ([rendered preview](https://htmlpreview.github.io/?https://github.com/kenhia/harness-eval/blob/main/_eval/report/infographic.html))
- 🧭 [Lessons learned](_eval/report/lessons-learned.md) — what run 1 taught us about building the eval harness itself

## How it worked

Five clean repos, one harness installed per repo (committed, then tagged
`pre-run` so agent-authored work is cleanly diffable). Each agent got a
byte-identical project spec — the [loglens CLI](_eval/prompts/00-project-spec.md),
a Combined-Log-Format access-log analyzer — with only the harness's own
"go" command varying on the first line ([prompts](_eval/prompts/)). Runs
were hands-off (zero interventions, all five). Grading: two independent
reviewers (Claude Fable, GPT Sol) each ran a sealed 12-check
[acceptance pass](_eval/acceptance.md) plus a 7-dimension weighted
[rubric](_eval/rubric.md), followed by a consensus session that adjudicated
factual disputes and reconciled score gaps. Full design:
[`_eval/README.md`](_eval/README.md).

## Repo map

```
0N-<harness>/          final state of each run (what the agent built)
_eval/README.md        eval design: contenders, isolation, run protocol
_eval/prompts/         project spec + the five per-harness prompts
_eval/rubric.md        grading dimensions, weights, grader instructions
_eval/acceptance.md    the 12 objective checks (sealed from the agents)
_eval/grades/          both graders' sheets, adjudication, final consensus
_eval/report/          white paper, infographic, lessons learned
_eval/runs/            run logs (timing, tokens, observations) + transcripts
_eval/grader_prompts/  the prompts that drove the grading sessions
```

### Per-run git history

`main` holds each run's final tree. The complete commit-by-commit story of
every run is preserved on namespaced refs:

- branches `history/0N-<harness>` — the run repo's full history
- tags `pre-run/0N-<harness>` — the boundary before the agent started

so `git diff pre-run/04-kprojects..history/04-kprojects` shows exactly what
that agent authored.

## Caveats (the honest list)

- n = 1 per harness; single task type; single model and runner.
- The eval author is also the author of one contender (kprojects).
- Harness artifacts self-identify, so grader blinding is imperfect.
- Graders shared the machine environment (klams/korg MCP available to all
  runs equally).

See the white paper's *threats to validity* section for the full treatment.

## License

MIT — see [LICENSE](LICENSE).
