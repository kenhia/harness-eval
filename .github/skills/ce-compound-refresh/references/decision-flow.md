## Phase 3: Ask for Decisions

### Autofix mode

**Skip this entire phase. Do not ask any questions. Do not present options. Do not wait for input.** Proceed directly to Phase 4 and execute all actions based on the classifications from Phase 2:

- Unambiguous Keep, Update, Consolidate, auto-Delete, and Replace (with sufficient evidence) → execute directly
- Ambiguous cases → mark as stale
- Then generate the report (see Output Format)

### Interactive mode

Most Updates and Consolidations should be applied directly without asking. Only ask the user when:

- The right action is genuinely ambiguous (Update vs Replace vs Consolidate vs Delete)
- You are about to Delete a document **and** the evidence is not unambiguous (see auto-delete criteria in Phase 2). When auto-delete criteria are met, proceed without asking.
- You are about to Consolidate and the choice of canonical doc is not clear-cut
- You are about to create a successor via Replace

Do **not** ask questions about whether code changes were intentional, whether the user wants to fix bugs in the code, or other concerns outside doc maintenance. Stay in your lane — doc accuracy.

#### Question Style

Always present choices using the platform's blocking question tool when available (`ask_user` in Copilot CLI). Otherwise, present numbered options in plain text and wait for the user's reply before proceeding.

Question rules:

- Ask **one question at a time**
- Prefer **multiple choice**
- Lead with the **recommended option**
- Explain the rationale for the recommendation in one concise sentence
- Avoid asking the user to choose from actions that are not actually plausible

#### Focused Scope

For a single artifact, present:

- file path
- 2-4 bullets of evidence
- recommended action

Then ask:

```text
This [learning/pattern] looks like a [Keep/Update/Consolidate/Replace/Delete].

Why: [one-sentence rationale based on the evidence]

What would you like to do?

1. [Recommended action]
2. [Second plausible action]
3. Skip for now
```

Do not list all five actions unless all five are genuinely plausible.

#### Batch Scope

For several learnings:

1. Group obvious **Keep** cases together
2. Group obvious **Update** cases together when the fixes are straightforward
3. Present **Consolidate** cases together when the canonical doc is clear
4. Present **Replace** cases individually or in very small groups
5. Present **Delete** cases individually unless they are strong auto-delete candidates

Ask for confirmation in stages:

1. Confirm grouped Keep/Update recommendations
2. Then handle Consolidate groups (present the canonical doc and what gets merged)
3. Then handle Replace one at a time
4. Then handle Delete one at a time unless the deletion is unambiguous and safe to auto-apply

#### Broad Scope

If the user asked for a sweeping refresh, keep the interaction incremental:

1. Narrow scope first
2. Investigate a manageable batch
3. Present recommendations
4. Ask whether to continue to the next batch

Do not front-load the user with a full maintenance queue.
