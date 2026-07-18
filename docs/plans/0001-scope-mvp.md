# Plan 0001 — Scope MVP: `watch` replacement with colored inter-refresh diffs

**Status:** Draft — awaiting user review
**Stack:** Rust (stable), ratatui + crossterm, clap
**Verified by:** existing CI (`cargo fmt --check`, `cargo clippy -D warnings`, `cargo test --workspace`)

## Product summary

`scope <command>` runs a command on an interval (like `watch`) and renders its
output in a TUI, highlighting what changed between refreshes in color. The core
twist over `watch -d`: richer diff modes (since-last, since-baseline,
cumulative), scrollback, and a snapshot history you can scrub through.

## Architecture principle (drives testability)

Split into a **pure core library** and a thin **TUI/IO shell**:

```
scope/
├── src/
│   ├── lib.rs          # pure core: fully unit-testable, no terminal, no clock
│   │   ├── runner.rs   # spawn command, capture stdout/stderr/exit/duration
│   │   ├── snapshot.rs # Snapshot { lines, exit, timestamp, duration }, ring-buffer history
│   │   ├── diff.rs     # line + intra-line diff between snapshots → styled spans
│   │   ├── app.rs      # App state machine: update(AppState, Event) -> AppState + effects
│   │   └── ui.rs       # render(AppState, Frame) — testable via ratatui TestBackend
│   └── main.rs         # arg parsing glue, terminal setup, event loop, real clock
```

- Time is injected (`trait Clock`) so scheduler logic unit-tests without sleeping.
- The event loop is `state = update(state, event)` (Elm-style) so every keybinding
  and tick is a pure-function unit test.
- Rendering is tested with `ratatui::backend::TestBackend` buffer assertions —
  no PTY needed for most tests.
- A `--frames N` flag (run N refreshes then exit) plus `--dump` (print final
  buffer to stdout, no alternate screen) makes true end-to-end CLI tests
  possible in CI via `assert_cmd`.

**Suggested crates:** `ratatui`, `crossterm`, `clap` (derive), `similar` (diff),
`ansi-to-tui` (ANSI passthrough, M5), `thiserror`/`anyhow`, dev-deps
`assert_cmd`, `predicates`, `insta` (optional snapshot tests).

---

## Milestone 0 — Cargo scaffold proven by CI

**Red:** `tests/smoke.rs::binary_prints_usage_without_args` — running the built
binary with no args exits non-zero and prints usage containing `scope` (needs
`assert_cmd` dev-dep; will fail until the crate exists).
**Green:** `cargo init` a binary crate with `lib.rs` + `main.rs`; clap arg
struct with a required `command: Vec<String>` and `--interval/-n` (default 2.0s);
no-args → clap usage error.
**Refactor:** none yet.
**CI:** existing workflow already runs fmt/clippy/test — this milestone makes it
go green on a real crate for the first time (today it trivially passes on an
empty repo; confirm the Test step actually executes the new test).
**Document:** commit message with what/why + test evidence (items 1–4).

## Milestone 1 — Command runner + Snapshot model

**Red (unit — `runner.rs`, `snapshot.rs`):**
- `runs_command_via_shell_and_captures_stdout` — `echo hi` → snapshot lines `["hi"]`, exit 0.
- `captures_stderr_interleaved_or_tagged` — command writing to stderr; assert it is captured (decide: merged like `watch`, tag lines for later styling).
- `captures_nonzero_exit_code` — `sh -c 'exit 3'` → exit code 3 recorded, output still shown.
- `nonexistent_command_yields_error_snapshot` — snapshot carries a spawn/shell error, app does not panic.
- `snapshot_records_duration_and_timestamp` — via injected `Clock` fake.
- `history_ring_buffer_caps_at_limit` — push limit+1 snapshots, oldest evicted (edge case: limit 0/1).
**Red (integration):** the above runner tests *are* integration tests of the
process boundary (real `sh -c` spawns); keep them in `tests/runner_it.rs`.
**Green:** `Runner::run(&cmd) -> Snapshot` using `std::process::Command` with
`sh -c` (exec-without-shell comes in M5); `History` ring buffer.
**Refactor:** extract output-normalization (trailing newline, tab expansion) into pure helpers.
**CI:** `cargo test --workspace` covers all of it; no new checks needed.
**Document:** note the stderr-handling decision as a mini-ADR in the PR description.

## Milestone 2 — Diff engine (the core twist)

**Red (unit — `diff.rs`, pure functions, no IO):**
- `identical_snapshots_produce_no_highlights`
- `changed_line_is_marked_changed_with_intra_line_word_spans` — `"cpu 10%"` → `"cpu 12%"` highlights only `12%`.
- `added_line_marked_added`, `removed_line_marked_removed` (decide policy: `watch` shows current output only — represent removals as a marker/optional gutter, not inline old text).
- `line_count_change_does_not_misalign_following_lines` — insertion in the middle; following equal lines must not all light up (this is `similar`'s job; test guards against naive index-pairing regressions).
- Edge cases: empty→nonempty output, output with only whitespace change, very long line (no panic, spans within bounds), unicode/emoji width safety.
- Diff modes as pure state: `mode_since_last`, `mode_since_baseline` (diff vs pinned snapshot), `mode_cumulative` (union of all changes since start — `watch -d=permanent` parity).
**Green:** `diff(old: &Snapshot, new: &Snapshot, mode) -> Vec<StyledLine>` built on `similar` line diff + word-level refinement on changed line pairs.
**Refactor:** intern styles; benchmark guard optional (large outputs).
**CI:** unit tests in the normal test job. If diff perf matters later, add a `cargo bench` job then — flagged as a gap, not needed for MVP.
**Document:** ADR-worthy: diff semantics (what "changed" means, removal display policy).

## Milestone 3 — TUI rendering via TestBackend

**Red (unit/integration — `ui.rs` with `ratatui::TestBackend`):**
- `renders_header_with_command_interval_and_last_run_time` — parity with `watch`'s title bar; also shows exit status and run duration.
- `renders_nonzero_exit_status_prominently` (e.g., red status cell).
- `renders_diff_highlights_in_body` — changed span carries the highlight style in the buffer.
- `body_scrolls_and_clamps` — content taller than viewport; scroll offset clamps at top/bottom (edge: content shorter than viewport → offset stays 0).
- `resize_rewraps_or_truncates_without_panic` — render at 5×5 and 200×50.
**Green:** `render(&AppState, &mut Frame)`; header `Paragraph`, body `Paragraph`/custom widget from `Vec<StyledLine>`, scrollbar.
**Refactor:** extract layout constants; consider `insta` snapshots of `TestBackend` buffers for the header.
**CI:** all runs headless under `cargo test` — TestBackend needs no TTY.
**Document:** screenshot/asciinema in README (manual — flag as the one non-CI-verifiable artifact; acceptable gap).

## Milestone 4 — Event loop, keybindings, end-to-end binary

**Red (unit — `app.rs` state machine, no terminal):**
- `tick_when_due_triggers_run_effect` / `tick_when_paused_does_not` (fake Clock).
- `space_toggles_pause`, `r_forces_immediate_refresh`, `q_and_ctrl_c_quit`,
  `arrows_and_page_keys_scroll`, `plus_minus_adjust_interval` (edge: interval floor, e.g. 0.1s),
  `d_cycles_diff_mode`, `b_sets_baseline_to_current_snapshot`, `question_mark_toggles_help_overlay`.
- Error case: run effect returning error snapshot keeps UI alive and shows status.
**Red (integration — `tests/e2e.rs` via `assert_cmd`):**
- `frames_flag_runs_n_times_and_exits_zero` — `scope --frames 2 --dump -n 0.1 -- echo hi`.
- `dump_output_contains_command_output`.
- `chg_detection_smoke` — command whose output changes (`date +%N` or a counter script fixture) → dump contains highlight markers (`--dump` should emit a plain-text marker form, e.g. changed spans wrapped in `«»`, so e2e tests don't parse ANSI).
**Green:** event loop in `main.rs`: crossterm event stream + tick timer feeding `update()`; effects executed by the shell; `--frames`/`--dump` implemented.
**Refactor:** keymap table → single source for the help overlay.
**CI:** e2e tests run in the existing test job (Linux has `sh`; fine). Flag as gap: no macOS/Windows runner yet — add an OS matrix entry when Windows support is attempted.
**Document:** README usage section + keybindings table; use `/dev-workflow:document` checklist.

## Milestone 5 — `watch` parity flags

Each flag is its own red→green loop; unit-test parsing + behavior, integration-test through `--frames/--dump` where observable:
- `-n/--interval <secs>` (done in M0/M4), `-p/--precise` — schedule on absolute ticks; unit test with fake Clock: next-run times don't drift after a slow run.
- `-x/--exec` — spawn without shell; test: args with spaces pass through un-word-split.
- `-e/--errexit` — pause + message on non-zero exit (test: state machine enters `Errexit` on error snapshot).
- `-g/--chgexit` — exit when output changes (e2e: counter fixture exits with code 0 after 2 frames).
- `-b/--beep` — emit BEL on change (unit: effect emitted; don't try to assert audio).
- `-t/--no-title` — header hidden (TestBackend assertion).
- `-c/--color` ANSI passthrough — parse command's ANSI output into ratatui styles via `ansi-to-tui`; unit: red ANSI input renders red span; edge: broken/truncated escape sequence doesn't panic. Decide interaction with diff highlighting (suggest: diff highlight = background color, passthrough = foreground).
**CI:** all covered by existing test job.
**Document:** flag reference table in README; parity matrix vs `procps watch`.

## Milestone 6 — Differentiators (backlog, priority order)

1. **History scrubbing** — `[`/`]` steps back/forward through the snapshot ring buffer; header shows "viewing t-3"; live updates continue in background. (Unit: navigation clamps; view-follows-live resumes on `End`.)
2. **Diff any two snapshots** — pick A/B from history.
3. **Regex highlight & filter** — `/pattern` highlights matches; `&pattern` shows only matching lines (watch has no equivalent; test regex-error case shows message, not crash).
4. **Alerts** — `--notify-on-change` (desktop notification) and `--alert <regex>`; unit-test as emitted effects.
5. **Session logging & replay** — `--log file.jsonl` appends each snapshot; `scope replay file.jsonl` reuses the history-scrubbing UI. Great for postmortems; also makes e2e tests richer.
6. **Change-frequency sparkline** in header (which refreshes changed, run durations).
7. **Horizontal scroll / wrap toggle** (`w`).
8. **Config file** (`~/.config/scope/config.toml`) + shell completions (`clap_complete`).
9. **Split panes** — multiple commands side-by-side (`scope 'cmd1' --and 'cmd2'`). Biggest differentiator, biggest scope; keep last.

Each item gets its own TDD breakdown when picked up (use `/dev-workflow:next`).

---

## Test scope checklist (rollup)

- [ ] Unit: diff engine, app state machine, scheduler (fake Clock), ANSI parsing, history buffer — happy/edge/error per milestone above
- [ ] Integration: process spawning (runner tests), terminal buffer (TestBackend), CLI e2e (`assert_cmd` + `--frames`/`--dump`)
- [ ] Regression: n/a yet (greenfield); every future bugfix adds one
- [ ] CI: existing workflow runs everything; gaps flagged — README screenshot (manual), OS matrix (deferred), perf bench (deferred)

## Known non-CI-verifiable gaps

1. Visual look-and-feel / flicker on a real terminal — manual; mitigated by TestBackend buffer tests.
2. BEL/desktop notifications actually firing — unit-tested as effects only.
3. Cross-platform (Windows/`cmd`) — out of MVP scope; add CI matrix when attempted.
