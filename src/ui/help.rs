use crate::config::ColorConfig;
use crate::event::{KEYBINDINGS, KeyCategory};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

pub fn render_hint_line(frame: &mut Frame, area: Rect, colors: &ColorConfig) {
    let hint = Paragraph::new(" Press ? for help").style(Style::default().fg(colors.text));
    frame.render_widget(hint, area);
}

pub fn render_help_popup(frame: &mut Frame, colors: &ColorConfig) {
    let area = centered_rect(60, 70, frame.area());

    // Clear the area behind the popup
    frame.render_widget(Clear, area);

    let content = build_help_content(colors);

    let popup = Paragraph::new(content).block(
        Block::default()
            .title(" Keyboard Shortcuts ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(colors.accent)),
    );

    frame.render_widget(popup, area);
}

fn build_help_content(colors: &ColorConfig) -> Vec<Line<'static>> {
    let max_key_width = KEYBINDINGS
        .iter()
        .map(|b| b.keys.chars().count())
        .max()
        .unwrap_or(0);
    let key_col_width = max_key_width + 3;

    let category_style = Style::default()
        .add_modifier(Modifier::BOLD)
        .add_modifier(Modifier::UNDERLINED)
        .fg(colors.accent);
    let key_style = Style::default().fg(colors.accent);

    let mut lines = vec![Line::from("")];
    let mut current_category: Option<KeyCategory> = None;

    for binding in KEYBINDINGS {
        if current_category != Some(binding.category) {
            if current_category.is_some() {
                lines.push(Line::from(""));
            }
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(binding.category.name(), category_style),
            ]));
            current_category = Some(binding.category);
        }

        let padding = " ".repeat(key_col_width.saturating_sub(binding.keys.chars().count()));
        lines.push(Line::from(vec![
            Span::raw("    "),
            Span::styled(binding.keys, key_style),
            Span::raw(padding),
            Span::raw(binding.description),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  Press ?, q, or Esc to close",
        Style::default().fg(colors.text),
    )));

    lines
}

/// Create a centered rect of given percentage of parent
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
