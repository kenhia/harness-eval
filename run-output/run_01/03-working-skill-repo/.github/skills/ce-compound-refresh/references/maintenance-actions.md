## Phase 2: Classify the Right Maintenance Action

After gathering evidence, assign one recommended action.

### Keep

The learning is still accurate and useful. Do not edit the file — report that it was reviewed and remains trustworthy. Only add `last_refreshed` if you are already making a meaningful update for another reason.

### Update

The core solution is still valid but references have drifted (paths, class names, links, code snippets, metadata). Apply the fixes directly.

### Consolidate

Choose **Consolidate** when Phase 1.75 identified docs that overlap heavily but are both materially correct. This is different from Update (which fixes drift in a single doc) and Replace (which rewrites misleading guidance). Consolidate handles the "both right, one subsumes the other" case.

**When to consolidate:**

- Two docs describe the same problem and recommend the same (or compatible) solution
- One doc is a narrow precursor and a newer doc covers the same ground more broadly
- The unique content from the subsumed doc can fit as a section or addendum in the canonical doc
- Keeping both creates drift risk without meaningful retrieval benefit

**When NOT to consolidate** (apply the Retrieval-Value Test from Phase 1.75):

- The docs cover genuinely different sub-problems that someone would search for independently
- Merging would create an unwieldy doc that harms navigation more than drift risk harms accuracy

**Consolidate vs Delete:** If the subsumed doc has unique content worth preserving (edge cases, alternative approaches, extra prevention rules), use Consolidate to merge that content first. If the subsumed doc adds nothing the canonical doc doesn't already say, skip straight to Delete.

The Consolidate action is: merge unique content from the subsumed doc into the canonical doc, then delete the subsumed doc. Not archive — delete. Git history preserves it.

### Replace

Choose **Replace** when the learning's core guidance is now misleading — the recommended fix changed materially, the root cause or architecture shifted, or the preferred pattern is different.

The user may have invoked the refresh months after the original learning was written. Do not ask them for replacement context they are unlikely to have — use agent intelligence to investigate the codebase and synthesize the replacement.

**Evidence assessment:**

By the time you identify a Replace candidate, Phase 1 investigation has already gathered significant evidence: the old learning's claims, what the current code actually does, and where the drift occurred. Assess whether this evidence is sufficient to write a trustworthy replacement:

- **Sufficient evidence** — you understand both what the old learning recommended AND what the current approach is. The investigation found the current code patterns, the new file locations, the changed architecture. → Proceed to write the replacement (see Phase 4 Replace Flow).
- **Insufficient evidence** — the drift is so fundamental that you cannot confidently document the current approach. The entire subsystem was replaced, or the new architecture is too complex to understand from a file scan alone. → Mark as stale in place:
   - Add `status: stale`, `stale_reason: [what you found]`, `stale_date: YYYY-MM-DD` to the frontmatter
   - Report what evidence you found and what is missing
   - Recommend the user run `ce-compound` after their next encounter with that area, when they have fresh problem-solving context

### Delete

Choose **Delete** when:

- The code or workflow no longer exists and the problem domain is gone
- The learning is obsolete and has no modern replacement worth documenting
- The learning is fully redundant with another doc (use Consolidate if there is unique content to merge first)
- There is no meaningful successor evidence suggesting it should be replaced instead

Action: delete the file. No archival directory, no metadata — just delete it. Git history preserves every deleted file if recovery is ever needed.

### Before deleting: check if the problem domain is still active

When a learning's referenced files are gone, that is strong evidence — but only that the **implementation** is gone. Before deleting, reason about whether the **problem the learning solves** is still a concern in the codebase:

- A learning about session token storage where `auth_token.rb` is gone — does the application still handle session tokens? If so, the concept persists under a new implementation. That is Replace, not Delete.
- A learning about a deprecated API endpoint where the entire feature was removed — the problem domain is gone. That is Delete.

Do not search mechanically for keywords from the old learning. Instead, understand what problem the learning addresses, then investigate whether that problem domain still exists in the codebase. The agent understands concepts — use that understanding to look for where the problem lives now, not where the old code used to be.

**Auto-delete only when both the implementation AND the problem domain are gone:**

- the referenced code is gone AND the application no longer deals with that problem domain
- the learning is fully superseded by a clearly better successor AND the old doc adds no distinct value
- the document is plainly redundant and adds nothing the canonical doc doesn't already say

If the implementation is gone but the problem domain persists (the app still does auth, still processes payments, still handles migrations), classify as **Replace** — the problem still matters and the current approach should be documented.

Do not keep a learning just because its general advice is "still sound" — if the specific code it references is gone, the learning misleads readers. But do not delete a learning whose problem domain is still active — that knowledge gap should be filled with a replacement.
