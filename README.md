# git-diff-tui

A space-efficient terminal UI for viewing git diffs with file tree navigation and delta integration.

## Features

- **File tree navigation** - Browse changed files with vim-like keybindings
- **Delta integration** - Beautiful syntax-highlighted diffs via delta (required)
- **Smart diff display** - Shows unstaged changes by default, toggle to staged with `s`
- **Hunk navigation** - Jump between diff hunks with `J`/`K`
- **Hot reload** - Automatically refreshes when files change
- **Toggle tree** - Hide/show file tree with `t` for full-width diff view

## Requirements

- **delta** - Must be installed and in PATH. Install from: https://github.com/dandavison/delta

## Installation

```bash
cargo install --path .
```

## Usage

Run in any git repository:

```bash
git-diff-tui
```

## Keybindings

All navigation uses `j`/`k` with modifiers:

| Key | Action |
|-----|--------|
| `q` | Quit |
| `j`/`k` | Navigate file tree |
| `Alt+j`/`Alt+k` | Scroll diff line by line |
| `Ctrl+j`/`Ctrl+k` | Scroll diff half page |
| `Shift+J`/`Shift+K` | Next / prev hunk |
| `l`/`Enter`/`→` | Expand folder |
| `h`/`←` | Collapse folder / go to parent |
| `Space`/`PageDown` | Page down diff |
| `PageUp` | Page up diff |
| `g`/`Home` | Top of diff |
| `G`/`End` | Bottom of diff |
| Mouse scroll | Scroll diff |
| `t` | Toggle file tree visibility |
| `s` | Toggle staged/unstaged (when file has both) |

## Status Icons

| Icon | Meaning |
|------|---------|
| `M` | Modified |
| `A` | Added |
| `D` | Deleted |
| `R` | Renamed |
| `?` | Untracked |
| `S` | Staged |
| `±` | Has both staged and unstaged changes |

## Planned Features

- **Commit history navigation** - When no uncommitted changes exist, show diffs from recent commits; navigate through commit history
- **Configuration file** - TOML config for delta args, keybindings, colors
- **CLI options** - Override config via command line (`--delta-args`, etc.)
- **Theme hot-reload** - Respond to terminal theme change signals (SIGUSR1)

## Development

### Testing with tmux

When developing with Claude Code, use a tmux pane for testing:

1. Open a tmux pane next to Claude Code
2. Get the pane info: `echo $TMUX_PANE && tmux display-message -p '#S'`
3. Share the session name and pane ID with Claude
4. Claude can then send commands and capture output for testing
