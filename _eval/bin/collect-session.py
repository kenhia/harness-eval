#!/usr/bin/env python3
"""collect-session.py — parse a runner session log into run-log metrics.

Usage:
  collect-session.py --runner claude  <session.jsonl> [subagent.jsonl ...]
  collect-session.py --runner copilot <events.jsonl>

Emits a markdown fragment (auto-captured metrics) on stdout. Used by
run-eval.sh, but safe to run by hand against any session file:

  claude:  <profile>/.claude/projects/<cwd-slug>/<session-id>.jsonl
           (pass the subagents/*.jsonl files too for full token totals)
  copilot: <profile>/.copilot/session-state/<session-id>/events.jsonl
"""

import json
import sys
from datetime import datetime
from pathlib import Path


def die(msg: str) -> None:
    print(f"collect-session.py: {msg}", file=sys.stderr)
    sys.exit(1)


def parse_ts(ts: str) -> datetime:
    return datetime.fromisoformat(ts.replace("Z", "+00:00"))


def fmt_dur(seconds: float) -> str:
    m, s = divmod(int(round(seconds)), 60)
    h, m = divmod(m, 60)
    return f"{h}h {m}m {s}s" if h else f"{m}m {s}s"


def fmt_tokens(n: int) -> str:
    if n >= 1_000_000:
        return f"{n / 1_000_000:.1f}m"
    if n >= 1_000:
        return f"{n / 1_000:.1f}k"
    return str(n)


def collect_claude(paths: list[Path]) -> list[str]:
    first_ts = last_ts = None
    versions: set[str] = set()
    # One assistant message may span several jsonl records (one per content
    # block) whose usage grows as output streams — keep the last record per
    # message id, then sum.
    usage_by_msg: dict[str, tuple[str, dict]] = {}

    for path in paths:
        with path.open() as fh:
            for line in fh:
                line = line.strip()
                if not line:
                    continue
                try:
                    rec = json.loads(line)
                except json.JSONDecodeError:
                    continue
                ts = rec.get("timestamp")
                if ts:
                    t = parse_ts(ts)
                    first_ts = t if first_ts is None else min(first_ts, t)
                    last_ts = t if last_ts is None else max(last_ts, t)
                if rec.get("version"):
                    versions.add(rec["version"])
                msg = rec.get("message") or {}
                if rec.get("type") == "assistant" and isinstance(msg, dict):
                    usage = msg.get("usage")
                    msg_id = msg.get("id") or rec.get("uuid", "")
                    if usage:
                        usage_by_msg[msg_id] = (msg.get("model", "unknown"), usage)

    usage_by_model: dict[str, dict[str, int]] = {}
    turns = len(usage_by_msg)
    for model, usage in usage_by_msg.values():
        agg = usage_by_model.setdefault(
            model, {"input": 0, "output": 0, "cache_read": 0, "cache_write": 0}
        )
        agg["input"] += usage.get("input_tokens", 0)
        agg["output"] += usage.get("output_tokens", 0)
        agg["cache_read"] += usage.get("cache_read_input_tokens", 0)
        agg["cache_write"] += usage.get("cache_creation_input_tokens", 0)

    if first_ts is None:
        die("no timestamped records found in claude session log(s)")

    out = [
        f"- session file(s): {', '.join(p.name for p in paths)}",
        f"- claude-code version: {', '.join(sorted(versions)) or 'unknown'}",
        f"- session start (first record): {first_ts.isoformat()}",
        f"- session end (last record): {last_ts.isoformat()}",
        f"- session span: {fmt_dur((last_ts - first_ts).total_seconds())}",
        f"- assistant turns (incl. subagents): {turns}",
    ]
    for model, u in sorted(usage_by_model.items()):
        out.append(
            f"- {model}: {fmt_tokens(u['input'])} input, {fmt_tokens(u['output'])} output, "
            f"{fmt_tokens(u['cache_read'])} cache read, {fmt_tokens(u['cache_write'])} cache write"
        )
    out.append("- cost: fill from `/cost` if interactive (token sums above are authoritative)")
    return out


def collect_copilot(paths: list[Path]) -> list[str]:
    if len(paths) != 1:
        die("copilot mode takes exactly one events.jsonl")
    start = shutdown = None
    model = None
    version = None
    with paths[0].open() as fh:
        for line in fh:
            line = line.strip()
            if not line:
                continue
            try:
                rec = json.loads(line)
            except json.JSONDecodeError:
                continue
            if rec.get("type") == "session.start":
                start = rec.get("data", {})
                model = start.get("selectedModel")
                version = start.get("copilotVersion")
            elif rec.get("type") == "session.shutdown":
                shutdown = rec.get("data", {})

    if not start:
        die("no session.start event found")
    out = [
        f"- session file: {paths[0]}",
        f"- copilot version: {version or 'unknown'}",
        f"- model: {model or 'unknown'}",
        f"- session start: {start.get('startTime', 'unknown')}",
    ]
    if not shutdown:
        out.append("- WARNING: no session.shutdown event (session still open or crashed?)")
        return out

    credits = shutdown.get("totalNanoAiu")
    tok = shutdown.get("tokenDetails", {})

    def tc(key: str) -> str:
        return fmt_tokens(tok.get(key, {}).get("tokenCount", 0))

    api_ms = shutdown.get("totalApiDurationMs")
    changes = shutdown.get("codeChanges", {})
    out += [
        f"- premium requests: {shutdown.get('totalPremiumRequests', 'unknown')}",
        f"- AI credits: {credits / 1e9:.1f}" if credits is not None else "- AI credits: unknown",
        f"- tokens: {tc('input')} input, {tc('output')} output, "
        f"{tc('cache_read')} cache read, {tc('cache_write')} cache write",
        f"- API duration: {fmt_dur(api_ms / 1000)}" if api_ms else "- API duration: unknown",
        f"- code changes: +{changes.get('linesAdded', '?')} / -{changes.get('linesRemoved', '?')} lines, "
        f"{len(changes.get('filesModified', []))} files",
    ]
    return out


def main() -> None:
    args = sys.argv[1:]
    if len(args) < 3 or args[0] != "--runner" or args[1] not in ("claude", "copilot"):
        die(__doc__)
    runner = args[1]
    paths = [Path(p) for p in args[2:]]
    for p in paths:
        if not p.is_file():
            die(f"not a file: {p}")
    lines = collect_claude(paths) if runner == "claude" else collect_copilot(paths)
    print("\n".join(lines))


if __name__ == "__main__":
    main()
