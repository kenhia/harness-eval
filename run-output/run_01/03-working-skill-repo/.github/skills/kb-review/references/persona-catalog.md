# Persona Catalog

17 reviewer personas organized into always-on, cross-cutting conditional, and stack-specific conditional layers, plus shared learning/runtime agents. The KB orchestrator replaces the standard maintainability reviewer with a thermonuclear code-quality reviewer.

Runtime note: use only Agent tool types that exist in the current runtime. `kb-review` is the orchestrating skill name, not an Agent tool type. If a named persona is unavailable, use `code-review` with that persona's instructions in the prompt instead of inventing an agent type.

## Always-on (4 personas + 2 CE agents)

Spawned on every review regardless of diff content.

**Persona agents (structured JSON output):**

| Persona | Agent | Focus |
|---------|-------|-------|
| `correctness` | `correctness-reviewer` | Logic errors, edge cases, state bugs, error propagation, intent compliance |
| `testing` | `testing-reviewer` | Coverage gaps, weak assertions, brittle tests, missing edge case tests |
| `thermo-nuclear-code-quality` | `thermo-nuclear-code-quality-reviewer` | Ambitious structural simplification, file sprawl, spaghetti branching, thin wrappers, code-judo opportunities |
| `project-standards` | `project-standards-reviewer` | AGENTS.md compliance -- frontmatter, references, naming, cross-platform portability, tool selection |

**CE agents (unstructured output, synthesized separately):**

| Agent | Focus |
|-------|-------|
| `agent-native-reviewer` | Verify new features are agent-accessible |
| `learnings-researcher` | Search docs/solutions/ for past issues related to this PR's modules and patterns |

## Conditional (8 personas)

Spawned when the orchestrator identifies relevant patterns in the diff. The orchestrator reads the full diff and reasons about selection -- this is agent judgment, not keyword matching.

| Persona | Agent | Select when diff touches... |
|---------|-------|---------------------------|
| `security` | `security-reviewer` | Auth middleware, public endpoints, user input handling, permission checks, secrets management |
| `performance` | `performance-reviewer` | Database queries, ORM calls, loop-heavy data transforms, caching layers, async/concurrent code |
| `api-contract` | `api-contract-reviewer` | Route definitions, serializer/interface changes, event schemas, exported type signatures, API versioning |
| `data-migrations` | `data-migrations-reviewer` | Migration files, schema changes, backfill scripts, data transformations |
| `reliability` | `reliability-reviewer` | Error handling, retry logic, circuit breakers, timeouts, background jobs, async handlers, health checks |
| `adversarial` | `adversarial-reviewer` | Diff has >=50 changed non-test, non-generated, non-lockfile lines, OR touches auth, payments, data mutations, external API integrations, or other high-risk domains |
| `cli-readiness` | `cli-readiness-reviewer` | CLI command definitions, argument parsing, CLI framework usage, command handler implementations |
| `previous-comments` | `previous-comments-reviewer` | **PR-only.** Reviewing a PR that has existing review comments or review threads from prior review rounds. Skip entirely when no PR metadata was gathered in Stage 1. |

## Stack-Specific Conditional (5 personas)

These reviewers keep their original opinionated lens. They are additive with the cross-cutting personas above, not replacements for them.

| Persona | Agent | Select when diff touches... |
|---------|-------|---------------------------|
| `dhh-rails` | `dhh-rails-reviewer` | Rails architecture, service objects, authentication/session choices, Hotwire-vs-SPA boundaries, or abstractions that may fight Rails conventions |
| `kieran-rails` | `kieran-rails-reviewer` | Rails controllers, models, views, jobs, components, routes, or other application-layer Ruby code where clarity and conventions matter |
| `kieran-python` | `kieran-python-reviewer` | Python modules, endpoints, services, scripts, or typed domain code |
| `kieran-typescript` | `kieran-typescript-reviewer` | TypeScript components, services, hooks, utilities, or shared types |
| `julik-frontend-races` | `julik-frontend-races-reviewer` | Stimulus/Turbo controllers, DOM event wiring, timers, async UI flows, animations, or frontend state transitions with race potential |

## CE Conditional Agents (migration-specific)

These CE-native agents provide specialized analysis beyond what the persona agents cover. Spawn them when the diff includes database migrations, schema.rb, or data backfills.

| Agent | Focus |
|-------|-------|
| `schema-drift-detector` | Cross-references schema changes against included migrations to catch unrelated drift |
| `deployment-verification-agent` | Produces Go/No-Go deployment checklist with verification queries and rollback procedures |

## Selection rules

1. **Always spawn all 4 always-on personas** plus the 2 shared learning/runtime agents. In `kb-review`, the structural quality slot is `thermo-nuclear-code-quality-reviewer`, not `maintainability-reviewer`.
2. **For each cross-cutting conditional persona**, the orchestrator reads the diff and decides whether the persona's domain is relevant. This is a judgment call, not a keyword match.
3. **For each stack-specific conditional persona**, use file types and changed patterns as a starting point, then decide whether the diff actually introduces meaningful work for that reviewer. Do not spawn language-specific reviewers just because one config or generated file happens to match the extension.
4. **For shared conditional agents**, spawn when the diff includes migration files (`db/migrate/*.rb`, `db/schema.rb`) or data backfill scripts.
5. **Announce the team** before spawning with a one-line justification per conditional reviewer selected.
