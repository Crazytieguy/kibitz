# kibitz

A terminal UI for kibitzing on your coding agent's changes. Watch git diffs in real-time with file tree navigation and delta integration.

## Features

- **File tree navigation** - Browse changed files with vim-like keybindings
- **Delta integration** - Beautiful syntax-highlighted diffs via delta (required)
- **Smart diff display** - Shows unstaged changes by default, toggle to staged with `s`
- **Hunk navigation** - Jump between diff hunks with `J`/`K`
- **Commit history** - Browse through commit history with `[`/`]`
- **Hot reload** - Automatically refreshes when files change
- **Toggle tree** - Hide/show file tree with `t` for full-width diff view
- **Configurable** - TOML config for delta args and colors

## Requirements

- **delta** - Must be installed and in PATH. Install from: https://github.com/dandavison/delta

## Installation

```bash
cargo install --path .
```

## Usage

Run in any git repository:

```bash
kibitz
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
| `[` | Go back one commit in history |
| `]` | Go forward (toward working tree) |

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

## Configuration

Configuration is loaded from TOML files in two locations (local overrides global):

1. **Global**: `~/.config/kibitz/config.toml`
2. **Local**: `.kibitz.toml` in repository root

### Example Configuration

```toml
[delta]
# Additional arguments passed to delta (appended after defaults)
args = "--side-by-side --line-numbers"

[colors]
# Colors can be specified as:
# - ANSI index (0-255): 4
# - Named color: "blue", "red", "green", "yellow", "cyan", "magenta", "white", "black"
# - Hex RGB: "#ff5500"

folder = 4           # ANSI blue (default)
modified = 3         # ANSI yellow (default)
added = 2            # ANSI green (default)
deleted = 1          # ANSI red (default)
renamed = 6          # ANSI cyan (default)
staged = 2           # ANSI green (default)
staged_modified = 5  # ANSI magenta (default)
untracked = 8        # ANSI bright black (default)
```

### Delta Arguments

The `delta.args` field accepts any arguments supported by delta. These are appended after the default arguments (`--paging=never --features={theme}`). Common options:

- `--side-by-side` - Side-by-side diff view
- `--line-numbers` - Show line numbers
- `--navigate` - Enable navigation markers
- `--dark` / `--light` - Force color theme

See [delta documentation](https://dandavison.github.io/delta/) for all options.

## Planned Features

- **Configurable keybindings** - Remap keys via config file
- **CLI options** - Override config via command line (`--delta-args`, etc.)
- **Theme hot-reload** - Respond to terminal theme change signals (SIGUSR1)

## Development

### Testing with tmux

When developing with Claude Code, use a tmux pane for testing:

1. Open a tmux pane next to Claude Code
2. Get the pane info: `echo $TMUX_PANE && tmux display-message -p '#S'`
3. Share the session name and pane ID with Claude
4. Claude can then send commands and capture output for testing
