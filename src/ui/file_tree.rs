use crate::config::ColorConfig;
use crate::model::{CommitInfo, FileStatus, FileTree, HorizontalItem};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

/// Returns the status icon and semantic color for a file status.
fn status_icon_and_color(
    status: Option<FileStatus>,
    colors: &ColorConfig,
) -> (&'static str, ratatui::style::Color) {
    match status {
        Some(FileStatus::Modified) => ("M ", colors.warning),
        Some(FileStatus::Added) => ("A ", colors.success),
        Some(FileStatus::Deleted) => ("D ", colors.error),
        Some(FileStatus::Renamed) => ("R ", colors.info),
        Some(FileStatus::Untracked) => ("? ", colors.text_muted),
        Some(FileStatus::Staged) => ("S ", colors.success),
        Some(FileStatus::StagedModified) => ("± ", colors.warning),
        None => ("  ", ratatui::style::Color::Reset),
    }
}

pub fn render(
    frame: &mut Frame,
    area: Rect,
    tree: &FileTree,
    colors: &ColorConfig,
    commit: Option<&CommitInfo>,
) {
    let visible = tree.visible_items();
    let items: Vec<ListItem> = visible
        .iter()
        .enumerate()
        .map(|(i, node)| {
            let indent = "  ".repeat(node.depth);

            let mut spans = vec![Span::raw(indent)];

            if node.is_dir {
                let icon = if node.expanded { "▼ " } else { "▶ " };
                spans.push(Span::styled(icon, Style::default().fg(colors.accent)));
                spans.push(Span::styled(
                    node.name.as_str(),
                    Style::default()
                        .fg(colors.accent)
                        .add_modifier(Modifier::BOLD),
                ));
            } else {
                let (icon, icon_color) = status_icon_and_color(node.status, colors);
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

    let list = List::new(items).block(Block::default().borders(Borders::RIGHT).title(title));

    frame.render_widget(list, area);
}

pub fn render_horizontal(
    frame: &mut Frame,
    area: Rect,
    tree: &FileTree,
    colors: &ColorConfig,
    commit: Option<&CommitInfo>,
) {
    let rows = tree.get_horizontal_rows();

    // Build lines for each row
    let lines: Vec<Line> = rows
        .iter()
        .map(|row| {
            let mut spans: Vec<Span> = Vec::new();

            for (i, item) in row.items.iter().enumerate() {
                if i > 0 {
                    spans.push(Span::raw("  ")); // separator between items
                }

                let item_spans = render_horizontal_item(item, colors);
                spans.extend(item_spans);
            }

            Line::from(spans)
        })
        .collect();

    let title = match commit {
        Some(c) => format!(" {} ", c.oid),
        None => " Files ".to_string(),
    };

    let paragraph =
        Paragraph::new(lines).block(Block::default().borders(Borders::TOP).title(title));

    frame.render_widget(paragraph, area);
}

fn render_horizontal_item(item: &HorizontalItem, colors: &ColorConfig) -> Vec<Span<'static>> {
    let mut spans: Vec<Span> = Vec::new();

    // Status icon for files
    if !item.is_dir {
        let (icon, icon_color) = status_icon_and_color(item.status, colors);
        spans.push(Span::styled(
            icon.to_string(),
            Style::default().fg(icon_color),
        ));
    }

    // Name with appropriate styling
    let name = if item.is_dir {
        format!("{}/", item.name)
    } else {
        item.name.clone()
    };

    let style = if item.is_selected {
        // Selected item: reverse video
        Style::default()
            .add_modifier(Modifier::REVERSED)
            .add_modifier(Modifier::BOLD)
    } else if item.is_on_path {
        // On path but not selected: bold + underlined for visibility
        Style::default()
            .fg(colors.accent)
            .add_modifier(Modifier::BOLD)
            .add_modifier(Modifier::UNDERLINED)
    } else if item.is_dir {
        // Directory not on path: accent color, dimmed
        Style::default()
            .fg(colors.accent)
            .add_modifier(Modifier::DIM)
    } else {
        // Regular file: dimmed
        Style::default().add_modifier(Modifier::DIM)
    };

    spans.push(Span::styled(name, style));

    spans
}
