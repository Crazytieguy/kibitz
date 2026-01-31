use crate::app::App;
use crate::config::LayoutMode;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

/// Handle key events. Returns true if the app should quit.
/// Keep in sync with keybindings.rs!
pub fn handle_key(app: &mut App, key: KeyEvent) -> Result<bool> {
    // Handle help popup first - it captures most keys when open
    if app.show_help {
        match key.code {
            KeyCode::Char('?') | KeyCode::Char('q') | KeyCode::Esc => {
                app.show_help = false;
            }
            _ => {}
        }
        return Ok(false);
    }

    match (key.code, key.modifiers) {
        // Quit
        (KeyCode::Char('q'), KeyModifiers::NONE) => return Ok(true),
        (KeyCode::Char('c'), KeyModifiers::CONTROL) => return Ok(true),

        // Help
        (KeyCode::Char('?'), KeyModifiers::NONE) => {
            app.show_help = true;
        }

        // === j/k family - all navigation ===

        // j/k alone - navigate file tree (layout-dependent)
        (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Down, KeyModifiers::NONE) => {
            match app.config.layout.mode {
                LayoutMode::Vertical => app.navigate_tree(|tree| tree.move_down()),
                LayoutMode::Horizontal => app.navigate_tree(|tree| tree.move_to_child()),
            }
        }
        (KeyCode::Char('k'), KeyModifiers::NONE) | (KeyCode::Up, KeyModifiers::NONE) => {
            match app.config.layout.mode {
                LayoutMode::Vertical => app.navigate_tree(|tree| tree.move_up()),
                LayoutMode::Horizontal => app.navigate_tree(|tree| tree.move_to_parent()),
            }
        }

        // Alt+j/k or Alt+arrows - scroll diff line by line
        (KeyCode::Char('j'), KeyModifiers::ALT) | (KeyCode::Down, KeyModifiers::ALT) => {
            app.diff_state.scroll_down(1);
        }
        (KeyCode::Char('k'), KeyModifiers::ALT) | (KeyCode::Up, KeyModifiers::ALT) => {
            app.diff_state.scroll_up(1);
        }

        // Ctrl+j/k - scroll diff half page
        (KeyCode::Char('j'), KeyModifiers::CONTROL) => {
            app.diff_state.scroll_down(15);
        }
        (KeyCode::Char('k'), KeyModifiers::CONTROL) => {
            app.diff_state.scroll_up(15);
        }

        // Shift+J/K or Shift+arrows - next/prev hunk
        (KeyCode::Char('J'), KeyModifiers::SHIFT) | (KeyCode::Down, KeyModifiers::SHIFT) => {
            app.diff_state.next_hunk();
        }
        (KeyCode::Char('K'), KeyModifiers::SHIFT) | (KeyCode::Up, KeyModifiers::SHIFT) => {
            app.diff_state.prev_hunk();
        }

        // === File tree expansion / sibling navigation (layout-dependent) ===
        (KeyCode::Char('l') | KeyCode::Right, KeyModifiers::NONE) => match app.config.layout.mode {
            LayoutMode::Vertical => app.file_tree.expand(),
            LayoutMode::Horizontal => app.navigate_tree(|tree| tree.move_to_next_sibling()),
        },
        (KeyCode::Char('h') | KeyCode::Left, KeyModifiers::NONE) => match app.config.layout.mode {
            LayoutMode::Vertical => app.file_tree.collapse(),
            LayoutMode::Horizontal => app.navigate_tree(|tree| tree.move_to_prev_sibling()),
        },
        // Enter always expands/enters in both modes
        (KeyCode::Enter, KeyModifiers::NONE) => {
            app.file_tree.expand();
        }

        // === Additional scroll keys ===
        (KeyCode::Char(' '), KeyModifiers::NONE) => {
            app.diff_state.scroll_down(30);
        }
        (KeyCode::PageDown, _) => {
            app.diff_state.scroll_down(15);
        }
        (KeyCode::PageUp, _) => {
            app.diff_state.scroll_up(15);
        }
        (KeyCode::Char('g'), KeyModifiers::NONE) | (KeyCode::Home, _) => {
            app.diff_state.scroll_to_top();
        }
        (KeyCode::Char('G'), KeyModifiers::SHIFT) | (KeyCode::End, _) => {
            app.diff_state.scroll_to_bottom();
        }

        // === Toggles ===
        (KeyCode::Char('t'), KeyModifiers::NONE) => {
            app.toggle_tree();
        }
        (KeyCode::Char('s'), KeyModifiers::NONE) => {
            app.toggle_staged();
        }

        // === History navigation ===
        (KeyCode::Char('['), KeyModifiers::NONE) => {
            app.go_back_in_history()?;
        }
        (KeyCode::Char(']'), KeyModifiers::NONE) => {
            app.go_forward_in_history()?;
        }

        _ => {}
    }

    Ok(false)
}

pub fn handle_mouse(app: &mut App, mouse: MouseEvent) -> Result<()> {
    match mouse.kind {
        MouseEventKind::ScrollDown => {
            app.diff_state.scroll_down(3);
        }
        MouseEventKind::ScrollUp => {
            app.diff_state.scroll_up(3);
        }
        _ => {}
    }
    Ok(())
}
