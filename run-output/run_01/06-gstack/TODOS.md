# TODOS

Deferred from the /autoplan review. Each was raised by a reviewer and is a real gap;
each expands the CLI surface beyond the spec loglens was built to, so none were built.
Ordered by value.

- [ ] **Read from stdin (`LOGFILE` = `-`) and accept multiple files (`nargs="+"`).**
  The strongest finding of the review: a log tool that cannot sit in a pipe loses to an
  `awk` one-liner for the exact users most likely to adopt it. Unlocks
  `zcat access.log.*.gz | loglens summary -` and rotated-log globs, which are the norm.
  The streaming reader already supports it; `source.py` needs a source abstraction and
  the exit-2 path needs to skip the stat check for `-`.

- [ ] **Transparent gzip.** `/var/log/nginx/access.log.1.gz` is the most common log file
  on any server, and today loglens reads the binary as text, parses nothing, and reports
  "no valid log lines" — a wrong diagnosis, not just an unhelpful one. Sniff `\x1f\x8b`
  and route through `gzip.open()`. ~3 lines. `goaccess` does this natively.

- [ ] **`--since`/`--until` on `summary`, `top`, and `hourly`.** "Top paths in the last
  hour" is arguably the most common question anyone asks a log, and it's currently
  impossible. The spec puts the flags on `errors` only. `analyze.filter_entries()` is
  already standalone and the flags already live on a shared parent parser, so this is
  moving two `add_argument` calls and threading one filter.

- [ ] **Distinguish the exit-1 causes.** Empty file, gzipped file, wrong log format, and
  JSON-format logs all collapse into "no valid log lines". Report which, with the first
  unparsable line next to an example of the expected format.

- [ ] **Parse stats for JSON consumers.** A `| jq` pipeline cannot see stderr, so it has
  no way to know whether it's reading a clean parse or a 40%-garbage one. Silent
  data-quality loss in the machine-readable path. Cannot be fixed by adding a `stats`
  key to the document — the spec requires the count on stderr, never stdout — so it
  needs a `--stats-fd` or an explicit opt-in flag.

- [ ] **`--strict`** (exit nonzero if any line is malformed) for CI pipelines, and
  **`-v`** to list every malformed line rather than just the first.

- [ ] **`loglens demo`** plus `sample.log` shipped as package data. Every subcommand
  needs a LOGFILE and no log ships with the tool, so a fresh install has nothing to run
  against.

- [ ] **Hypothesis property test** over the parser: generate CLF lines from random
  fields (quotes, backslashes, control chars), format, parse, assert round-trip. The
  cheapest possible defense for the escaping rules, but it adds a dev dependency.

- [ ] **`--anonymize`** to hash client IPs. They are personal data in some jurisdictions
  and `--format json` exports them verbatim.
