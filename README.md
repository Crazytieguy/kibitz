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

| Key | Action |
|-----|--------|
| `q` | Quit |
| `j`/`k`/`↑`/`↓` | Navigate file tree |
| `l`/`Enter`/`→` | Expand folder |
| `h`/`←` | Collapse folder / go to parent |
| `J`/`K` (shift) | Next / prev hunk |
| `Ctrl-d`/`Ctrl-u` | Half-page scroll diff |
| `g`/`G` | Top / bottom of diff |
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

## Development

### Testing with tmux

When developing with Claude Code, use a tmux pane for testing:

1. Open a tmux pane next to Claude Code
2. Get the pane info: `echo $TMUX_PANE && tmux display-message -p '#S'`
3. Share the session name and pane ID with Claude
4. Claude can then send commands and capture output for testing
