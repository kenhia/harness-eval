# ksandbox containerized runs — design (corrected 2026-07-18)

Status: **design + Claude-side proof only.** Not on the run_03 critical
path (see Blocker). Supersedes the roadmap sketch, which mis-stated the
architecture.

## Corrected architecture (what's actually true)

- **kai is the eval controller; ksandbox is a Docker host it drives over
  ssh.** The `sandbox` docker context lives on **kai**
  (`ssh://ken@ksandbox`, verified: `docker -c sandbox run --rm alpine …`
  works). Nothing runs on the ksandbox host directly.
- **The runner CLI runs INSIDE a container image**, not on ksandbox.
  ksandbox has Docker 29.6.1 and no `claude`/`copilot`/`node` — by
  design. My earlier "run `claude setup-token` on ksandbox" was wrong on
  both count (claude isn't there) and location.
- **Auth is injected, not installed.** `claude setup-token` runs **on
  kai** (claude 2.1.215, authed) and yields a long-lived token passed to
  the container as an env var — no credential-snapshot rotation, no
  keyring. This is the clean win over fake-HOME for the Claude runner.

## Blocker: the field is majority-Copilot

run_03's 7 cells are 5 Copilot + 2 Claude. **Copilot-in-a-container auth
is unsolved** (the E1 saga: login lives in the desktop keyring, MCP
approval is per-login, VS Code PAT injection is what got MCP
policy-blocked). Until that spike lands, a container can run only the 2
Claude cells — not the field. So **fake-HOME remains the only path that
runs the whole run_03 field today**; ksandbox is pursued as its own
track, not a run_03 prerequisite.

## One-time Ken actions (when we pursue the Claude-side proof)

1. `claude setup-token` on kai (interactive browser OAuth) → capture the
   long-lived token into `_eval/profiles/.sandbox-token` (gitignored).
2. Nothing else — the `sandbox` context already works.

## Build sketch (Claude-side proof first)

- **Image** `harness-eval/runner-claude`: base + uv + rust toolchain +
  `npm i -g @anthropic-ai/claude-code`, pinned versions (kills mid-field
  CLI drift — a real advantage over auto-updating local CLIs).
- **`run-eval.sh --sandbox`**: instead of `env HOME=<profile> claude …`,
  `docker -c sandbox run --rm --network … -v <staging-repo>:/work
  -e CLAUDE_CODE_OAUTH_TOKEN=… <image> claude -p …`; collect session
  logs from a mounted volume; same runlog/acceptance flow after.
- **Egress**: NOT hermetic — needs Anthropic API, crates.io/PyPI,
  tailnet MCP. `--network` allowlist, not `none`. (Contrast kvllm's
  `--network none` coding sandbox: different threat model — there the
  model is local, here the runner must reach its API.)
- **Manifest win**: the image digest IS the environment manifest
  (lesson 30) — captured mechanically per run, no drift.

## Copilot-in-container spike (gates the other 5 cells)

Open questions, each a killer if unanswerable: does `copilot` auth
persist headlessly without the desktop keyring? can MCP approval be
pre-baked into an image or mounted? is there a device-flow/token path
that isn't the policy-blocked gh-CLI class? Run the spike in one
throwaway container before committing to containerizing the Copilot
runner.
