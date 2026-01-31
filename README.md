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

- **delta** - Must be installed and in PATH:
  ```bash
  # macOS
  brew install git-delta

  # Cargo
  cargo install git-delta

  # Other: https://dandavison.github.io/delta/installation.html
  ```

## Installation

### Homebrew (macOS/Linux)

```bash
brew install Crazytieguy/tap/kibitz
```

### Cargo

```bash
cargo install kibitz
```

### Pre-built binaries

Download from [GitHub Releases](https://github.com/Crazytieguy/kibitz/releases).

Or use the install script (macOS/Linux):

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/Crazytieguy/kibitz/releases/latest/download/kibitz-installer.sh | sh
```

## Usage

Run in any git repository:

```bash
kibitz
```

## Keybindings

Arrow keys and `j`/`k` are interchangeable. Press `?` for in-app help.

| Key | Action |
|-----|--------|
| `q` | Quit |
| `?` | Show help |
| `j`/`k` or `↓`/`↑` | Navigate file tree |
| `Alt+j`/`Alt+k` or `Alt+↓`/`Alt+↑` | Scroll diff line by line |
| `Ctrl+j`/`Ctrl+k` or `PageUp`/`PageDown` | Scroll diff half page |
| `Shift+J`/`Shift+K` or `Shift+↓`/`Shift+↑` | Next / prev hunk |
| `l`/`Enter`/`→` | Expand folder |
| `h`/`←` | Collapse folder / go to parent |
| `Space` | Page down diff |
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

1. **Global**: `~/Library/Application Support/kibitz/config.toml` (macOS) or `~/.config/kibitz/config.toml` (Linux)
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

- **Semantic color palette** - Unify colors across UI with semantic names (text-primary, text-muted, accent, etc.) instead of per-element colors
- **Configurable keybindings** - Remap keys via config file
- **CLI options** - Override config via command line (`--delta-args`, etc.)
- **Theme hot-reload** - Respond to terminal theme change signals (SIGUSR1)
- **XDG config path** - Use `~/.config/kibitz/` on all platforms instead of platform-native paths

## Development

### Testing with tmux

When developing with Claude Code, use a tmux pane for testing:

1. Open a tmux pane next to Claude Code
2. Get the pane info: `echo $TMUX_PANE && tmux display-message -p '#S'`
3. Share the session name and pane ID with Claude
4. Claude can then send commands and capture output for testing
