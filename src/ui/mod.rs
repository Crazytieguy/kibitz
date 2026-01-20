mod diff_view;
mod file_tree;
mod layout;

use crate::app::App;
use ratatui::Frame;

pub fn render(frame: &mut Frame, app: &App) {
    let areas = layout::create_layout(frame.area(), app.show_tree, &app.file_tree);

    if app.show_tree {
        file_tree::render(frame, areas.tree, &app.file_tree);
    }

    diff_view::render(frame, areas.diff, &app.diff_state);
}
