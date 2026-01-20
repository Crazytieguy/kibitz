use crate::model::{FileStatus, FileTree};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

// Use ANSI indexed colors (0-15) which respect terminal theme
const BLUE: Color = Color::Indexed(4);      // ANSI blue
const YELLOW: Color = Color::Indexed(3);    // ANSI yellow
const GREEN: Color = Color::Indexed(2);     // ANSI green
const RED: Color = Color::Indexed(1);       // ANSI red
const CYAN: Color = Color::Indexed(6);      // ANSI cyan
const MAGENTA: Color = Color::Indexed(5);   // ANSI magenta
const BRIGHT_BLACK: Color = Color::Indexed(8); // For dimmed text

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
                spans.push(Span::styled(icon, Style::default().fg(BLUE)));
                spans.push(Span::styled(
                    name.as_str(),
                    Style::default().fg(BLUE).add_modifier(Modifier::BOLD),
                ));
            } else {
                let (icon, icon_color) = match status {
                    Some(FileStatus::Modified) => ("M ", YELLOW),
                    Some(FileStatus::Added) => ("A ", GREEN),
                    Some(FileStatus::Deleted) => ("D ", RED),
                    Some(FileStatus::Renamed) => ("R ", CYAN),
                    Some(FileStatus::Untracked) => ("? ", BRIGHT_BLACK),
                    Some(FileStatus::Staged) => ("S ", GREEN),
                    Some(FileStatus::StagedModified) => ("± ", MAGENTA),
                    None => ("  ", Color::Reset),
                };
                spans.push(Span::styled(icon, Style::default().fg(icon_color)));
                spans.push(Span::raw(name.as_str()));
            }

            let mut item = ListItem::new(Line::from(spans));

            if i == tree.selected_index {
                // Use reverse video for selection - works with any theme
                item = item.style(
                    Style::default()
                        .add_modifier(Modifier::REVERSED)
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
