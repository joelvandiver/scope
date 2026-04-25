# PLAN.md

## Project: `scope` — a `ratatui`-based `watch` replacement

### Crate Structure

```
scope/
├── Cargo.toml
└── src/
    ├── main.rs       # Entry point, async runtime, terminal init, panic hook, task orchestration
    ├── cli.rs        # Clap Args struct
    ├── app.rs        # AppState struct wrapped in Arc<Mutex<>>
    ├── executor.rs   # Command execution loop, timing, diff invocation, state updates
    ├── diff.rs       # Diff computation using `similar`, DiffLine type
    └── tui.rs        # Ratatui render function, event loop, scroll handling
```

### Key Dependencies

| Crate           | Purpose                                                     |
| --------------- | ----------------------------------------------------------- |
| `ratatui`       | TUI framework                                               |
| `crossterm`     | Terminal backend, raw mode, event polling                   |
| `clap` (derive) | Argument parsing                                            |
| `tokio` (full)  | Async runtime + process spawning                            |
| `similar`       | Myers diff algorithm (powers difftastic)                    |
| `ansi-to-tui`   | Convert ANSI escape codes to ratatui `Text` (for `-c` flag) |

### Architecture

Two concurrent tasks share `Arc<Mutex<AppState>>`:

```
┌─────────────────────────────────────────────────┐
│ main()                                          │
│  parse args → init terminal → spawn executor    │
│  run TUI event loop (main thread)               │
└─────────────────────────────────────────────────┘
         │                         ▲
         │ Arc<Mutex<AppState>>    │ crossterm key events
         ▼                         │
┌──────────────────┐   ┌───────────────────────────┐
│ executor task    │   │ TUI event loop            │
│ (tokio::spawn)   │   │  poll events (100ms tick) │
│                  │   │  re-render each tick      │
│ loop:            │──▶│  handle q/Ctrl+C          │
│  sleep(interval) │   │  j/k/g/G scroll           │
│  run command     │   └───────────────────────────┘
│  capture output  │
│  compute diff    │
│  lock & update   │
│  AppState        │
└──────────────────┘
```

No `mpsc` channel — the TUI reads state on every 100ms tick, so a mutex is simpler and sufficient.

### CLI Flags (matching `watch`)

`-n` interval, `-d` differences, `-t` no title, `-c` ANSI color, `-e` exit on error, `-x` exec mode (skip shell), `-p` precise timing. The `command` arg is `Vec<String>` so no quoting needed.

### Key Design Decisions

- **`tokio::time::MissedTickBehavior`**: `Skip` for `--precise`, `Delay` otherwise
- **Diff always computed** — when `-d` is off, kinds are all `Same`, renderer just skips coloring (no branch)
- **Scroll offset lives in `AppState`** so the executor can implement auto-scroll-to-bottom (disengages on manual scroll)
- **Panic hook** restores terminal before printing — required for usable shell after crash

### Implementation Order

- [x] 1. Project scaffold + `Cargo.toml`
- [x] 2. CLI parsing (`cli.rs`)
- [x] 3. Command execution loop without TUI (`executor.rs`)
- [x] 4. Diff computation with unit tests (`diff.rs`)
- [x] 5. `AppState` struct (`app.rs`)
- [x] 6. Minimal TUI loop with clean exit + panic hook (`tui.rs`)
- [x] 7. Wire executor to TUI — tool is functionally complete
- [x] 8. Header rendering (`--no-title`)
- [x] 9. Diff highlighting (`--differences`)
- [ ] 10. ANSI color support (`--color`)
- [ ] 11. Scrolling + auto-scroll-to-bottom
- [ ] 12. `--errexit` via `CancellationToken`
- [ ] 13. Edge cases (no output, resize, unicode, binary)
