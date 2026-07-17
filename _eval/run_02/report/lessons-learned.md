# Lessons learned — run 02 + fix round 02.1

Continues the run-1 series (lessons 1–21,
`../../run_01/report/lessons-learned.md`). Input for eval v3.

## What the v2 changes bought (validation)

22. **Executable sealed acceptance eliminated the dispute class.** Zero
    grader reconciliations in both rounds (run 1 and 1.5 each needed
    one); correctness matched mechanically on all 13 graded cells;
    identical pre-consensus rank order in the build round. The one
    suite defect (S1, header-case) was caught by the first live repo,
    fixed under the documented adjudication path, and every completed
    repo re-run — the process held under fire.
23. **The hard tier discriminated — once.** 22.5-point build spread vs
    run 1's 4.5. But the discrimination came from a single shared
    failure (C9/H12), not from breadth: the other eleven hard probes
    passed everywhere. Pinned-spec semantics get implemented correctly
    at frontier capability; the discriminating checks are the ones
    probing what agents don't think to doubt (parser leniency, not
    boundary math).
24. **Mechanical run logs + preflight worked.** Auto-captured
    timestamps/tokens/versions across 13 runs + probes; the auth
    preflight caught and self-healed a credential expiry; the
    launch-failure guard kept aborted launches from leaving artifacts.
    Manual fields got filled because they were few and clearly marked.

## New findings

25. **Dependency choice was the eval's sharpest discriminator.** 2 of 8
    implementations picked a strict XML parser and were immune to the
    field's only correctness failure by construction; 6 picked the
    lenient one and independently converged on the same bug, the same
    inadequate fixture, and later the same fix. No harness influenced
    the choice; no agent justified it. v3 candidate: a task where a
    load-bearing dependency decision is explicit and scoreable.
26. **Agents test what their parser catches.** All six failing repos
    authored malformed-XML fixtures — in the flavor their parser
    rejects — and certified the failing behavior. Self-authored tests
    inherit the implementation's blind spots; only external
    adversarial checks (or a different implementation) expose them.
27. **The fix round inverted the build round.** Greenfield: harness
    machinery mostly trailed same-runner controls. Fix-own-bug: all
    four harness/control pairs went to the harness, driven by
    unprompted regression tests and verification passes. Small (5-pt
    field, N=1) but directionally consistent — the first evidence here
    that handoff/verify machinery pays on resume-style work. v3's
    refactor and resume-from-handoff cells should press exactly this.
28. **On the Copilot runner, fix cost tracked test depth ~linearly**
    (67→86→152→262 credits for verbatim-repro → +fixture → +e2e →
    +e2e+empty-guard). Thoroughness is a purchasable, measurable
    artifact — anchor future efficiency scoring to what the spend
    bought, not just how much was spent.
29. **Variance is now measured: single-run rankings are weather.** The
    99/07 rep pair: 44% wall-clock spread, identical throughput, and
    opposite outcomes on the decisive C9 check under an identical
    frozen prompt. Publish N=1 as preliminary, treat only
    cross-cutting patterns (shared bug, 4-for-4 fix direction) as
    findings, and buy reps before buying new cells.

## Environment/process (the E1 saga, condensed)

30. **Ambient services change under you mid-field.** Copilot CLI's MCP
    approval gate activated between two runs hours apart and was
    misdiagnosed three times (org policy → token class → approval
    gate) before probes isolated it. v3 preflight must capture a
    per-run MCP/tool availability manifest mechanically — lesson 7
    upgraded from "decide per cell" to "prove per run".
31. **Fake-HOME sharp edges, Copilot edition**: auth lives in the
    desktop keyring (per-profile `/login`, invisible to non-desktop
    shells); MCP servers imported via config file are untrusted until
    re-added in-CLI; VS Code terminals export a short-lived
    `COPILOT_GITHUB_TOKEN` that goes stale and hijacks launches (now
    scrubbed by run-eval.sh); and fish passes `HOME=~/...` literally,
    mirroring profile trees into `./~/` junk. All codified in
    _eval/README.md; the tooling now defends against each.
32. **Cost units are not stable across CLI versions.** Same cell, same
    premium requests: 143 credits (run 1) vs 703 (run 02). Record
    cost in the most primitive units available (premium requests,
    tokens, wall clock) and treat vendor cost abstractions as
    unversioned floats.
33. **Voiding needs a protocol and got one ad hoc.** E1's spineless
    run was voided: work preserved on a `void/*` branch, artifacts to
    `.scratch`, repo reset to the boundary tag, graders barred from
    the void. Worth writing into ADDING-A-HARNESS as the standard
    invalid-run path.
