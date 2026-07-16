# 01-atv-starterkit — grader: sol
| dim | score /5 | note |
|---|---:|---|
| correctness | 3 | The supplied acceptance result is core 13/14 and hard 11/12. The core failure caps correctness at 3: truncated malformed XML is accepted when the event stream reaches EOF, so both isolation checks report an empty successful refresh. |
| code quality | 3 | The workspace is the smallest in the field and separates parsing, storage, fetching, refresh, API, client, and fixture concerns without speculative abstraction. The parser never checks that its element stack is empty at EOF and strips namespaces, while refresh silently treats a database error reading conditional headers as “no validators”; those are significant correctness and error-handling weaknesses. |
| tests | 2 | The repo has eleven parser/date unit tests and one real local-HTTP end-to-end test covering registration, refresh, queries, failure isolation, and deletion. It has no feedctl, database, or feedgen-focused tests, does not exercise update-in-place without a 304, pagination, exit codes, or exact conditional-header transmission, and its malformed fixture misses truncated XML. |
| docs | 5 | `README.md` independently covers build/install, the full quality gate, all three binaries, exit codes, fixtures, pinned semantics, object fields, and every REST endpoint. Its failure-isolation guarantee is incorrect for the acceptance fixture, but the required operating reference is complete and clear. |
| process | 4 | Five agent commits form a coherent core/server/feedgen/client/tests-docs sequence with meaningful messages. However, the run ended with uncommitted `.atv` runtime state, requiring evaluator snapshot commit `c950337`; the two timestamp-only observation records add no product or review value. |
| efficiency | 5 | At 16m22s with 15 premium requests, 596.7 credits, and 71.1k output tokens, the run is faster and cheaper than the Copilot control. It maps to the control-comparable 5 band. |
| autonomy | 4 | There were zero human interventions and the agent declared done with the product commits present. The working tree was not actually clean at handoff, so an external post-run snapshot commit was needed to preserve the harness state; this falls short of fully finished-and-committed autonomy. |
**Weighted total:** 68/100

**Best thing:** It delivered the field's leanest codebase with a clean five-step product commit sequence and complete documentation.

**Worst thing:** Regression protection is very thin, and the agent handed off a dirty tree while the only malformed-XML test failed to cover the acceptance case.

**Narrative (≤150 words):** StarterKit produced a compact, well-documented artifact quickly, with a sensible five-commit progression through the product surfaces. The implementation passes nearly all acceptance behavior, but shares the streaming-parser EOF defect that turns truncated XML into a successful empty feed. Its own suite is the weakest in the field: one broad e2e test and eleven parser/date tests leave feedctl exits, storage updates, pagination, and conditional request details unprotected. The final product commits existed, but `.atv` runtime state remained dirty and required an evaluator snapshot, so both process and autonomy receive deductions. Correctness is mechanically capped at 3 by the core failure.
