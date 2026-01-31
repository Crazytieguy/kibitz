//! Keybinding definitions - single source of truth for help generation.
//!
//! When updating keybindings in handler.rs, update this file too.
//! The help popup is generated from these definitions.

use KeyCategory::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum KeyCategory {
    General,
    FileTree,
    DiffScrolling,
    Toggles,
    History,
}

impl KeyCategory {
    pub fn name(self) -> &'static str {
        match self {
            General => "General",
            FileTree => "File Tree",
            DiffScrolling => "Diff Scrolling",
            Toggles => "Toggles",
            History => "History",
        }
    }
}

pub struct Keybinding {
    pub keys: &'static str,
    pub description: &'static str,
    pub category: KeyCategory,
}

/// All keybindings for help display. Keep in sync with handler.rs.
pub static KEYBINDINGS: &[Keybinding] = &[
    Keybinding {
        keys: "q",
        description: "Quit",
        category: General,
    },
    Keybinding {
        keys: "?",
        description: "Toggle help",
        category: General,
    },
    Keybinding {
        keys: "j / k / \u{2191} / \u{2193}",
        description: "Navigate files",
        category: FileTree,
    },
    Keybinding {
        keys: "l / Enter / \u{2192}",
        description: "Expand folder",
        category: FileTree,
    },
    Keybinding {
        keys: "h / \u{2190}",
        description: "Collapse / go to parent",
        category: FileTree,
    },
    Keybinding {
        keys: "Alt + (j / k / \u{2191} / \u{2193})",
        description: "Scroll line by line",
        category: DiffScrolling,
    },
    Keybinding {
        keys: "Ctrl + (j / k) / PgUp / PgDn",
        description: "Scroll half page",
        category: DiffScrolling,
    },
    Keybinding {
        keys: "Space",
        description: "Page down",
        category: DiffScrolling,
    },
    Keybinding {
        keys: "g / Home",
        description: "Top of diff",
        category: DiffScrolling,
    },
    Keybinding {
        keys: "G / End",
        description: "Bottom of diff",
        category: DiffScrolling,
    },
    Keybinding {
        keys: "Shift + (J / K / \u{2191} / \u{2193})",
        description: "Next / prev hunk",
        category: DiffScrolling,
    },
    Keybinding {
        keys: "t",
        description: "Toggle file tree",
        category: Toggles,
    },
    Keybinding {
        keys: "s",
        description: "Toggle staged / unstaged",
        category: Toggles,
    },
    Keybinding {
        keys: "[ / ]",
        description: "Prev / next commit",
        category: History,
    },
];
