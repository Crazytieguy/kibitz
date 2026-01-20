use crate::config::ColorConfig;
use crate::model::{CommitInfo, FileStatus, FileTree};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

pub fn render(frame: &mut Frame, area: Rect, tree: &FileTree, colors: &ColorConfig, commit: Option<&CommitInfo>) {
    let visible = tree.visible_items();
    let items: Vec<ListItem> = visible
        .iter()
        .enumerate()
        .map(|(i, node)| {
            let indent = "  ".repeat(node.depth);

            let mut spans = vec![Span::raw(indent)];

            if node.is_dir {
                let icon = if node.expanded { "▼ " } else { "▶ " };
                spans.push(Span::styled(icon, Style::default().fg(colors.folder)));
                spans.push(Span::styled(
                    node.name.as_str(),
                    Style::default().fg(colors.folder).add_modifier(Modifier::BOLD),
                ));
            } else {
                let (icon, icon_color) = match node.status {
                    Some(FileStatus::Modified) => ("M ", colors.modified),
                    Some(FileStatus::Added) => ("A ", colors.added),
                    Some(FileStatus::Deleted) => ("D ", colors.deleted),
                    Some(FileStatus::Renamed) => ("R ", colors.renamed),
                    Some(FileStatus::Untracked) => ("? ", colors.untracked),
                    Some(FileStatus::Staged) => ("S ", colors.staged),
                    Some(FileStatus::StagedModified) => ("± ", colors.staged_modified),
                    None => ("  ", Color::Reset),
                };
                spans.push(Span::styled(icon, Style::default().fg(icon_color)));
                spans.push(Span::raw(node.name.as_str()));
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

    let title = match commit {
        Some(c) => format!(" {} ", c.oid),
        None => " Changes ".to_string(),
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::RIGHT).title(title));

    frame.render_widget(list, area);
}
