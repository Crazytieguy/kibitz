use crate::model::DiffState;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
};

pub fn render(frame: &mut Frame, area: Rect, state: &DiffState) {
    let title = if state.has_both {
        if state.showing_staged {
            " Diff (staged) [s to toggle] "
        } else {
            " Diff (unstaged) [s to toggle] "
        }
    } else if state.showing_staged {
        " Diff (staged) "
    } else {
        " Diff "
    };

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

    let paragraph = Paragraph::new(state.content.clone())
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((state.scroll_offset as u16, 0));

    frame.render_widget(paragraph, area);

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
                Style::default().fg(Color::DarkGray),
            );
        }
    }
}
