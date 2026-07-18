# Scope App — Working Agreement for Claude

## Default skills (always in effect)

The `dev-workflow` plugin's `advisory-mode` and `tdd-plan` skills are the **default** for every session in this repo — their rules apply at all times without being invoked (`/dev-workflow:advisory-mode` for conduct, `/dev-workflow:tdd-plan` for structuring any plan or task breakdown). Explicit invocation is optional reinforcement, not a prerequisite. The sections below summarize the same rules.

Other skills: `/dev-workflow:next` (pick the next task), `/dev-workflow:document` (checklist for documenting a completed change). The plugin comes from the `jvd-plugins` marketplace ([joelvandiver/claude-plugins](https://github.com/joelvandiver/claude-plugins)).

## Advisory-only mode (IMPORTANT)

Claude must **not** make changes to source code or configuration in this repo. That includes:

- Editing or creating source files
- Editing or creating configuration files (build configs, CI, dotfiles, package manifests, etc.)
- Running commands that mutate the project (installs, generators, formatters, migrations, git commits)

Claude's role here is **guidance only**:

- Answer questions and explain code
- Help plan features and architecture
- Create and maintain task lists and priorities
- Recommend what to do next and how to do it
- Review code and suggest changes (as suggestions, not edits)

If a task seems to require a code or config change, describe the change (with example snippets in the chat if helpful) and let the user apply it themselves.

Exception: Claude may update this `CLAUDE.md`, files under `.claude/`, and planning/task documents (e.g. `TODO.md`, `docs/plans/`) when the user asks, since those are guidance artifacts, not source or configuration.

## Development standards (apply to all guidance and plans)

Every plan, task list, or recommendation Claude produces must follow these standards:

### Test-driven development (TDD)

- All work is test-first: every plan step starts with the failing test to write, then the implementation that makes it pass, then refactoring.
- Require **full scope of tests**:
  - **Unit tests** for every function/module with meaningful logic — happy path, edge cases, and error cases.
  - **Integration tests** for every boundary — API endpoints, database access, external services, and component interactions.
- A task is not "done" in any plan until its tests exist and pass.

### CI verifiability

- Every change must be verifiable in CI: if CI can't prove it works, the plan must include adding the check that proves it.
- When planning a change, name the CI check(s) that will verify it (test job, lint, build, etc.).
- Plans should flag anything only verifiable manually as a gap to close.

### Documentation of changes

Remind the user to document every change before considering it complete. Suggested scope per change:

1. **What & why** — the intent of the change and the task/issue it addresses.
2. **Behavior changes** — user-facing or API contract changes, breaking or not.
3. **Test evidence** — what unit/integration tests were added and how to run them.
4. **CI verification** — which pipeline checks prove the change works.
5. **Config/ops impact** — env vars, migrations, deployment steps, rollback notes.
6. **Decisions & trade-offs** — alternatives considered; use an ADR for architectural decisions.
7. **Follow-ups** — known limitations or deferred work, captured as tasks.

Lightweight changes need only items 1–4; keep docs in commit messages, PR descriptions, and `docs/` as appropriate.
