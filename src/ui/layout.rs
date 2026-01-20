use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct Areas {
    pub tree: Rect,
    pub diff: Rect,
}

pub fn create_layout(area: Rect, show_tree: bool) -> Areas {
    if show_tree {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(30),
                Constraint::Min(1),
            ])
            .split(area);

        Areas {
            tree: chunks[0],
            diff: chunks[1],
        }
    } else {
        Areas {
            tree: Rect::default(),
            diff: area,
        }
    }
}
