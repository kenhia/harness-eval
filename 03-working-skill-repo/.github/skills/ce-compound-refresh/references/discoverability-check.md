# Discoverability Check

After the refresh report is generated, check whether the project's instruction files would lead an agent to discover and search `docs/solutions/` before starting work in a documented area. This runs every time; the knowledge store only compounds value when agents can find it.

If this check produces edits, they are committed as part of or immediately after the refresh commit.

## Assessment

1. Identify whether `AGENTS.md` exists at the repo root. If it does, that is the assessment and edit target. If it does not, skip this check.
2. Assess whether an agent reading the instruction files would learn three things:
   - a searchable knowledge store of documented solutions exists;
   - enough about its structure to search effectively, including category organization and YAML frontmatter fields such as `module`, `tags`, and `problem_type`;
   - when it is relevant, such as implementing features, debugging issues, or making decisions in documented areas.
3. Treat this as a semantic assessment, not a string match. The information may live in an architecture section, gotchas section, directory listing, conventions block, or several small references.
4. If an agent would reasonably discover and use the knowledge store after reading the file, the check passes.

## Edit Guidance

If the instruction file does not surface `docs/solutions/`:

1. Match the file's existing structure, tone, and density.
2. Prefer one line in the closest related section. A line in an existing architecture tree, directory listing, documentation section, or conventions block is usually better than a new section.
3. Add a new headed section only when no existing section is remotely related.
4. Describe the knowledge store itself, not the plugin.
5. Keep timing informational, not imperative. Use wording like "relevant when implementing or debugging in documented areas" rather than "always check before implementing."

Example line for an existing directory listing:

```text
docs/solutions/  # documented solutions to past problems, organized by category with YAML frontmatter
```

Example short section when there is no natural fit:

```markdown
## Documented Solutions

`docs/solutions/` contains documented solutions to past problems, organized by category with YAML frontmatter (`module`, `tags`, `problem_type`). Relevant when implementing or debugging in documented areas.
```

## Interaction and Commit

In interactive mode, explain why the edit matters, show the proposed change and location, and get consent before editing.

In autofix mode, include the gap as a discoverability recommendation in the report. Do not edit instruction files from autofix mode because autofix scope is doc maintenance, not project configuration.

If an instruction-file edit is made and the refresh changes were already committed, either amend the existing commit if it has not been pushed or create a follow-up commit such as `docs: add docs/solutions discoverability to AGENTS.md`. If the branch was already pushed, push the follow-up commit too.
