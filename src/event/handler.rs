use crate::app::App;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

/// Handle key events. Returns true if the app should quit.
pub fn handle_key(app: &mut App, key: KeyEvent) -> Result<bool> {
    match (key.code, key.modifiers) {
        // Quit
        (KeyCode::Char('q'), KeyModifiers::NONE) => return Ok(true),
        (KeyCode::Char('c'), KeyModifiers::CONTROL) => return Ok(true),

        // File tree navigation
        (KeyCode::Char('j') | KeyCode::Down, KeyModifiers::NONE) => {
            let prev_file = app.file_tree.selected_file_path();
            app.file_tree.move_down();
            let new_file = app.file_tree.selected_file_path();
            if prev_file != new_file {
                app.request_diff();
            }
        }
        (KeyCode::Char('k') | KeyCode::Up, KeyModifiers::NONE) => {
            let prev_file = app.file_tree.selected_file_path();
            app.file_tree.move_up();
            let new_file = app.file_tree.selected_file_path();
            if prev_file != new_file {
                app.request_diff();
            }
        }
        (KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter, KeyModifiers::NONE) => {
            app.file_tree.expand();
        }
        (KeyCode::Char('h') | KeyCode::Left, KeyModifiers::NONE) => {
            app.file_tree.collapse();
        }

        // Hunk navigation (shift+j/k)
        (KeyCode::Char('J'), KeyModifiers::SHIFT) | (KeyCode::Down, KeyModifiers::SHIFT) => {
            app.diff_state.next_hunk();
        }
        (KeyCode::Char('K'), KeyModifiers::SHIFT) | (KeyCode::Up, KeyModifiers::SHIFT) => {
            app.diff_state.prev_hunk();
        }

        // Diff scrolling - half page
        (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
            app.diff_state.scroll_down(15);
        }
        (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
            app.diff_state.scroll_up(15);
        }

        // Diff scrolling - full page
        (KeyCode::PageDown, _) | (KeyCode::Char(' '), KeyModifiers::NONE) => {
            app.diff_state.scroll_down(30);
        }
        (KeyCode::PageUp, _) => {
            app.diff_state.scroll_up(30);
        }

        // Diff scrolling - single line (vim style)
        (KeyCode::Char('e'), KeyModifiers::CONTROL) => {
            app.diff_state.scroll_down(1);
        }
        (KeyCode::Char('y'), KeyModifiers::CONTROL) => {
            app.diff_state.scroll_up(1);
        }

        // Diff scrolling - top/bottom
        (KeyCode::Char('g'), KeyModifiers::NONE) => {
            app.diff_state.scroll_to_top();
        }
        (KeyCode::Char('G'), KeyModifiers::SHIFT) | (KeyCode::End, _) => {
            app.diff_state.scroll_to_bottom();
        }
        (KeyCode::Home, _) => {
            app.diff_state.scroll_to_top();
        }

        // Toggle tree visibility
        (KeyCode::Char('t'), KeyModifiers::NONE) => {
            app.toggle_tree();
        }

        // Toggle staged/unstaged
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
