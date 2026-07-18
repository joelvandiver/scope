# Scope — TODO

Derived from [docs/plans/0001-scope-mvp.md](docs/plans/0001-scope-mvp.md).
Work top-to-bottom; use `/dev-workflow:next` to pick up the next task.

**Definition of done for every task:** failing test written first → implementation makes it pass → CI green (fmt, clippy, test) → change documented (`/dev-workflow:document`). A checked box means all four.

## M0 — Cargo scaffold proven by CI

- [ ] Red: `tests/smoke.rs::binary_prints_usage_without_args` (via `assert_cmd`)
- [ ] Green: `cargo init` binary crate with `lib.rs`/`main.rs` split; clap args: `command: Vec<String>`, `--interval/-n` (default 2.0s)
- [ ] CI: confirm the Test step actually runs the new test (no longer trivially green)
- [ ] Document: commit message with what/why + test evidence

## M1 — Command runner + Snapshot model

- [ ] Red (integration, `tests/runner_it.rs`): stdout capture, stderr handling, non-zero exit code, nonexistent command → error snapshot, duration/timestamp via fake `Clock`
- [ ] Red (unit): `history_ring_buffer_caps_at_limit` incl. limit 0/1 edge cases
- [ ] Green: `Runner::run(&cmd) -> Snapshot` via `sh -c`; `History` ring buffer; injected `Clock` trait
- [ ] Refactor: extract output normalization (trailing newline, tabs) into pure helpers
- [ ] Document: mini-ADR on stderr handling decision (merged vs tagged)

## M2 — Diff engine

- [ ] Red (unit, pure): identical → no highlights; changed line → intra-line word spans; added/removed line policy; mid-insertion doesn't misalign following lines
- [ ] Red (edge cases): empty→nonempty, whitespace-only change, very long line, unicode/emoji width
- [ ] Red (modes): since-last, since-baseline, cumulative (`watch -d=permanent` parity)
- [ ] Green: `diff(old, new, mode) -> Vec<StyledLine>` on `similar` + word-level refinement
- [ ] Refactor: intern styles
- [ ] Document: ADR on diff semantics (what "changed" means, removal display policy)

## M3 — TUI rendering (TestBackend)

- [ ] Red: header (command/interval/last run), non-zero exit shown prominently, diff highlights present in buffer
- [ ] Red: scroll clamps top/bottom (incl. content shorter than viewport); render at 5×5 and 200×50 without panic
- [ ] Green: `render(&AppState, &mut Frame)` — header, styled body, scrollbar
- [ ] Refactor: layout constants; optional `insta` buffer snapshots
- [ ] Document: README screenshot/asciinema (known manual gap)

## M4 — Event loop, keybindings, e2e binary

- [ ] Red (state machine): tick-when-due runs / tick-when-paused doesn't (fake Clock); keys — pause (space), refresh (r), quit (q/Ctrl-C), scroll, interval ±  with 0.1s floor, diff-mode cycle (d), baseline (b), help (?)
- [ ] Red (error case): error snapshot keeps UI alive with status shown
- [ ] Red (e2e, `tests/e2e.rs`): `--frames 2 --dump -n 0.1 -- echo hi` exits 0; dump contains output; changing-output fixture shows `«»` change markers
- [ ] Green: crossterm event loop feeding `update()`; implement `--frames` and `--dump`
- [ ] Refactor: keymap table as single source for help overlay
- [ ] Document: README usage + keybindings table

## M5 — `watch` parity flags (one red→green loop each)

- [ ] `-p/--precise` — no drift after slow run (fake Clock test)
- [ ] `-x/--exec` — no shell; args with spaces not word-split
- [ ] `-e/--errexit` — pause + message on non-zero exit
- [ ] `-g/--chgexit` — exit on change (e2e counter fixture)
- [ ] `-b/--beep` — BEL emitted as effect on change
- [ ] `-t/--no-title` — header hidden (TestBackend)
- [ ] `-c/--color` — ANSI passthrough via `ansi-to-tui`; truncated escape doesn't panic; diff highlight = background, passthrough = foreground
- [ ] Document: flag reference + parity matrix vs procps `watch`

## M6 — Differentiators (backlog, priority order — TDD breakdown when picked up)

- [ ] History scrubbing (`[`/`]` through ring buffer, live continues)
- [ ] Diff any two snapshots from history
- [ ] Regex highlight (`/`) and filter (`&`); regex-error shows message, not crash
- [ ] Alerts: `--notify-on-change`, `--alert <regex>` (effects, unit-tested)
- [ ] Session logging (`--log file.jsonl`) + `scope replay`
- [ ] Change-frequency sparkline in header
- [ ] Horizontal scroll / wrap toggle (`w`)
- [ ] Config file + shell completions (`clap_complete`)
- [ ] Split panes (multiple commands) — largest scope, last

## Known gaps (flagged, not blocking)

- [ ] README screenshot — manual verification only
- [ ] OS matrix in CI — add when Windows/macOS support attempted
- [ ] Perf bench job — add if diff performance becomes a concern
