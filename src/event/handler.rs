use crate::app::App;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

/// Handle key events. Returns true if the app should quit.
pub fn handle_key(app: &mut App, key: KeyEvent) -> Result<bool> {
    match (key.code, key.modifiers) {
        // Quit
        (KeyCode::Char('q'), KeyModifiers::NONE) => return Ok(true),
        (KeyCode::Char('c'), KeyModifiers::CONTROL) => return Ok(true),

        // === j/k family - all navigation ===

        // j/k alone - navigate file tree
        (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Down, KeyModifiers::NONE) => {
            app.navigate_tree(|tree| tree.move_down());
        }
        (KeyCode::Char('k'), KeyModifiers::NONE) | (KeyCode::Up, KeyModifiers::NONE) => {
            app.navigate_tree(|tree| tree.move_up());
        }

        // Alt+j/k - scroll diff line by line
        (KeyCode::Char('j'), KeyModifiers::ALT) => {
            app.diff_state.scroll_down(1);
        }
        (KeyCode::Char('k'), KeyModifiers::ALT) => {
            app.diff_state.scroll_up(1);
        }

        // Ctrl+j/k - scroll diff half page
        (KeyCode::Char('j'), KeyModifiers::CONTROL) => {
            app.diff_state.scroll_down(15);
        }
        (KeyCode::Char('k'), KeyModifiers::CONTROL) => {
            app.diff_state.scroll_up(15);
        }

        // Shift+J/K - next/prev hunk
        (KeyCode::Char('J'), KeyModifiers::SHIFT) => {
            app.diff_state.next_hunk();
        }
        (KeyCode::Char('K'), KeyModifiers::SHIFT) => {
            app.diff_state.prev_hunk();
        }

        // === File tree expansion ===
        (KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter, KeyModifiers::NONE) => {
            app.file_tree.expand();
        }
        (KeyCode::Char('h') | KeyCode::Left, KeyModifiers::NONE) => {
            app.file_tree.collapse();
        }

        // === Additional scroll keys ===
        (KeyCode::PageDown, _) | (KeyCode::Char(' '), KeyModifiers::NONE) => {
            app.diff_state.scroll_down(30);
        }
        (KeyCode::PageUp, _) => {
            app.diff_state.scroll_up(30);
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
