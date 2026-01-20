use crate::model::{FileStatus, FileTree};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

pub fn render(frame: &mut Frame, area: Rect, tree: &FileTree) {
    let visible = tree.visible_items();
    let items: Vec<ListItem> = visible
        .iter()
        .enumerate()
        .map(|(i, (name, depth, is_dir, expanded, status))| {
            let indent = "  ".repeat(*depth);

            let mut spans = vec![Span::raw(indent)];

            if *is_dir {
                let icon = if *expanded { "▼ " } else { "▶ " };
                spans.push(Span::styled(icon, Style::default().fg(Color::Blue)));
                spans.push(Span::styled(
                    name.as_str(),
                    Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD),
                ));
            } else {
                let (icon, icon_color) = match status {
                    Some(FileStatus::Modified) => ("M ", Color::Yellow),
                    Some(FileStatus::Added) => ("A ", Color::Green),
                    Some(FileStatus::Deleted) => ("D ", Color::Red),
                    Some(FileStatus::Renamed) => ("R ", Color::Cyan),
                    Some(FileStatus::Untracked) => ("? ", Color::Gray),
                    Some(FileStatus::Staged) => ("S ", Color::Green),
                    Some(FileStatus::StagedModified) => ("± ", Color::Magenta),
                    None => ("  ", Color::White),
                };
                spans.push(Span::styled(icon, Style::default().fg(icon_color)));
                spans.push(Span::raw(name.as_str()));
            }

            let mut item = ListItem::new(Line::from(spans));

            if i == tree.selected_index {
                item = item.style(
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                );
            }

            item
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::RIGHT).title(" Files "));

    frame.render_widget(list, area);
}
