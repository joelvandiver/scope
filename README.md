# scope

A terminal UI replacement for the Linux `watch` command, built with [ratatui](https://github.com/ratatui-org/ratatui).

## Install

```sh
cargo install --path .
```

## Usage

```sh
scope [OPTIONS] <COMMAND>...
```

### Options

| Flag           | Default | Description                                             |
| -------------- | ------- | ------------------------------------------------------- |
| `-n <seconds>` | `2.0`   | Interval between runs                                   |
| `-t`           | off     | Hide header                                             |
| `-c`           | off     | Interpret ANSI color codes                              |
| `-e`           | off     | Exit on non-zero exit code                              |
| `-x`           | off     | Pass command directly to exec (skip shell)              |
| `-p`           | off     | Precise timing (subtract command runtime from interval) |

### Key Bindings

| Key            | Action                               |
| -------------- | ------------------------------------ |
| `q` / `Ctrl+C` | Quit                                 |
| `j` / `↓`      | Scroll down one line                 |
| `k` / `↑`      | Scroll up one line                   |
| `d`            | Scroll down half page                |
| `u`            | Scroll up half page                  |
| `g` / `Home`   | Scroll to top                        |
| `G` / `End`    | Scroll to bottom (enables tracking)  |

## Examples

```sh
# Watch directory contents every 2 seconds
scope ls -la

# Update every 5 seconds
scope -n 5 cat /proc/meminfo

# Preserve color output
scope -c -- ls --color=always
```
