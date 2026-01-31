use crate::config::ColorConfig;
use crate::model::{CommitInfo, DiffState, STICKY_FILE_HEADER_HEIGHT};
use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &DiffState,
    commit: Option<&CommitInfo>,
    colors: &ColorConfig,
) {
    let title = build_title(state, commit);

    let hunk_info = if !state.hunk_positions.is_empty() {
        format!(
            " Hunk {}/{} ",
            state.current_hunk + 1,
            state.hunk_positions.len()
        )
    } else {
        String::new()
    };

    let block = Block::default()
        .borders(Borders::NONE)
        .title(title)
        .title_bottom(hunk_info);

    let inner_area = block.inner(area);

    // Check if we need sticky headers
    let sticky_file_header = state.sticky_file_header();
    let sticky_hunk_header = state.sticky_hunk_header();

    // Convert logical scroll offset to visual offset accounting for wrapped lines
    let visual_offset = visual_scroll_offset(
        &state.content,
        state.scroll_offset,
        inner_area.width as usize,
    );

    let paragraph = Paragraph::new(state.content.clone())
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((visual_offset as u16, 0));

    frame.render_widget(paragraph, area);

    // Render sticky file header if needed (file name + divider = 2 lines)
    if let Some(header_pos) = sticky_file_header {
        let line_indices = [header_pos, header_pos + 1];
        render_sticky_header(frame, &state.content, &line_indices, inner_area, 0);
    }

    // Render sticky hunk header if needed (box top + marker + box bottom = 3 lines)
    if let Some(hunk_pos) = sticky_hunk_header {
        let y_offset = if sticky_file_header.is_some() {
            STICKY_FILE_HEADER_HEIGHT as u16
        } else {
            0
        };
        let line_indices = [hunk_pos - 1, hunk_pos, hunk_pos + 1];
        render_sticky_header(frame, &state.content, &line_indices, inner_area, y_offset);
    }

    // Draw scrollbar indicator if content is longer than view
    if state.total_lines > inner_area.height as usize {
        let scrollbar_height = inner_area.height as usize;
        let scroll_ratio = state.scroll_offset as f64 / state.total_lines.max(1) as f64;
        let thumb_pos = (scroll_ratio * scrollbar_height as f64) as u16;
        let thumb_pos = thumb_pos.min(inner_area.height.saturating_sub(1));

        // Render a simple scrollbar character at the right edge
        let scrollbar_x = area.x + area.width.saturating_sub(1);
        let scrollbar_y = inner_area.y + thumb_pos;

        if scrollbar_x < area.x + area.width && scrollbar_y < area.y + area.height {
            frame.buffer_mut().set_string(
                scrollbar_x,
                scrollbar_y,
                "█",
                Style::default().fg(colors.text_muted),
            );
        }
    }
}

fn build_title(state: &DiffState, commit: Option<&CommitInfo>) -> String {
    if let Some(c) = commit {
        let msg = truncate_message(&c.message, 50);
        return format!(" {}: {} ", c.oid, msg);
    }

    let staged_label = if state.showing_staged {
        "staged"
    } else {
        "unstaged"
    };
    let toggle_hint = if state.has_both { " [s to toggle]" } else { "" };

    if state.showing_staged || state.has_both {
        format!(" Diff ({staged_label}){toggle_hint} ")
    } else {
        " Diff ".to_string()
    }
}

fn truncate_message(msg: &str, max_len: usize) -> String {
    if msg.len() > max_len {
        format!("{}...", &msg[..max_len - 3])
    } else {
        msg.to_string()
    }
}

/// Calculate how many visual rows a single logical line occupies when wrapped.
fn visual_line_count(line: &Line, width: usize) -> usize {
    if width == 0 {
        return 1;
    }
    let len: usize = line.spans.iter().map(|s| s.content.chars().count()).sum();
    if len == 0 { 1 } else { len.div_ceil(width) }
}

/// Calculate the visual scroll offset by summing up visual rows for all lines
/// up to the logical scroll offset.
fn visual_scroll_offset(content: &Text, logical_offset: usize, width: usize) -> usize {
    content
        .lines
        .iter()
        .take(logical_offset)
        .map(|line| visual_line_count(line, width))
        .sum()
}

/// Render a sticky header by extracting lines at the given indices and displaying them
/// at the specified y_offset within inner_area.
fn render_sticky_header(
    frame: &mut Frame,
    content: &Text<'static>,
    line_indices: &[usize],
    inner_area: Rect,
    y_offset: u16,
) {
    let sticky_lines: Vec<_> = line_indices
        .iter()
        .filter_map(|&idx| content.lines.get(idx).cloned())
        .collect();

    if sticky_lines.is_empty() {
        return;
    }

    let sticky_area = Rect {
        x: inner_area.x,
        y: inner_area.y + y_offset,
        width: inner_area.width,
        height: sticky_lines.len() as u16,
    };

    frame.render_widget(Clear, sticky_area);
    frame.render_widget(Paragraph::new(Text::from(sticky_lines)), sticky_area);
}
