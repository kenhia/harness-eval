# Grader prompt — GPT Sol: run_03 S2 delta re-grade (cells 03 and 06 only)

You previously graded all seven run_03 (Haiku 4.5 tier) repos. A suite
defect has since been found and fixed, and it changed the acceptance
results for **two cells only**. Re-grade **03-working-skill-repo** and
**06-gstack**, and nothing else.

**Your grader id is `sol`.** You are running as a CLI session on kai
— everything is local. `$EVAL` = `/home/ken/src/ai-agents/harness-eval`;
repos at `~/src/ai-agents/harness-eval-runs/run_03/NN-<name>`.

## What changed (defect S2 — read `$EVAL/_eval/run_03/DEFECTS.md` first)

The acceptance suite ran `<subcommand> … --format json` and only
retried the leading form `--format json <subcommand> …` when stderr
said "unrecognized" (argparse's wording). Click says `No such option`,
so the retry never fired for click-based CLIs — failing them on JSON
checks they actually pass. **The spec calls `--format` a global option;
accepting it before the subcommand is a legitimate reading. Those
repos were right and the suite was wrong.** A companion fix (S2b) makes
the one-command check accept `Justfile` as `just` itself does.

Corrected results (supplied — do not re-derive):

| cell | before | after | flipped to PASS |
|---|---|---|---|
| 03 | 10/12, 6/9 | **12/12 core, 8/9 hard** | A3, A7, H7, H8 |
| 06 | 8/12, 6/9 | **9/12 core, 8/9 hard** | A3, H7, H8 |

Residual failures that still stand: 03 fails H9 (naive `--since`); 06
fails A7, A9, A12, H9.

## Hard rules

- **Re-score all seven dimensions for these two cells from scratch**,
  using the corrected acceptance output in `runs/NN-acceptance.txt`.
  Your other five sheets are frozen and must not be touched.
- **Discount reasoning that rested on the defect.** Any criticism of
  these repos for "wrong `--format` placement", for JSON checks that
  failed only under the old fixture, or for a capital-J `Justfile`, is
  void — the suite was at fault.
- **Do NOT excuse genuinely agent-owned defects** that S2 does not
  cover, e.g.: 03's README documents the trailing `--format` form its
  own CLI rejects (a real docs defect); 06's dependency layout breaks
  `uv run pytest` and `just check` from a clean environment.
- Correctness: apply the rubric's **mechanical mapping table** to the
  corrected (core, hard) pair; state the pair and the resulting score.
- Everything else from your original brief still applies: tier-own
  calibration (do NOT anchor to run_01/run_02 sheets), efficiency vs
  the same-runner control, S1 layout note for 03 (nesting is a Process
  observation, not a correctness failure), grade only
  `pre-run..HEAD`, work on clones under `/tmp/`, never modify run repos.
- **Shared-memory blackout still in force**: do not query klams/korg
  about this eval and do not write grading content to them.
- **Independence**: do not read anything with `fable` in the filename,
  and nothing under `grades/reconcile/`.

## Output

- Overwrite `$EVAL/_eval/run_03/grades/03-sol.md` and
  `06-sol.md` with the re-scored sheets (same template).
- Update the two affected rows in
  `$EVAL/_eval/run_03/grades/summary-sol.md`; leave the other rows
  untouched.
- Add a one-line note at the top of each re-scored sheet:
  `> Re-scored after suite defect S2 (corrected acceptance results).`
- Commit: `git add _eval/run_03/grades && git commit -m
  "grades(sol): run_03 S2 delta re-grade — cells 03 and 06"`.

Report the two revised rows in chat and stop. No reconciliation, no
peeking at the other grader's work.
