mod diff_view;
mod file_tree;
mod layout;

use crate::app::App;
use crate::config::LayoutMode;
use ratatui::Frame;

pub fn render(frame: &mut Frame, app: &App) {
    let areas = layout::create_layout_for_mode(
        frame.area(),
        app.show_tree,
        &app.file_tree,
        app.config.layout.mode,
        app.config.layout.max_rows,
    );

    if app.show_tree {
        match app.config.layout.mode {
            LayoutMode::Vertical => {
                file_tree::render(
                    frame,
                    areas.tree,
                    &app.file_tree,
                    &app.config.colors,
                    app.current_commit.as_ref(),
                );
            }
            LayoutMode::Horizontal => {
                file_tree::render_horizontal(
                    frame,
                    areas.tree,
                    &app.file_tree,
                    &app.config.colors,
                    app.current_commit.as_ref(),
                );
            }
        }
    }

    diff_view::render(
        frame,
        areas.diff,
        &app.diff_state,
        app.current_commit.as_ref(),
    );
}
