use crate::model::FileTree;
use ratatui::layout::{Constraint, Direction, Layout, Rect};

const MIN_TREE_WIDTH: u16 = 20;
const MAX_TREE_WIDTH: u16 = 50;
const TREE_PADDING: u16 = 4; // For icon, spacing, and border

pub struct Areas {
    pub tree: Rect,
    pub diff: Rect,
}

pub fn create_layout(area: Rect, show_tree: bool, file_tree: &FileTree) -> Areas {
    if show_tree {
        let tree_width = calculate_tree_width(file_tree, area.width);

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(tree_width),
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

fn calculate_tree_width(file_tree: &FileTree, max_available: u16) -> u16 {
    let max_name_width = file_tree
        .visible_items()
        .iter()
        .map(|node| {
            // Calculate display width: indent (2 chars per depth) + icon (2) + name
            (node.depth as u16 * 2) + 2 + node.name.len() as u16
        })
        .max()
        .unwrap_or(MIN_TREE_WIDTH);

    let desired_width = max_name_width + TREE_PADDING;

    // Clamp to min/max and don't exceed half the screen
    let max_allowed = (max_available / 2).max(MIN_TREE_WIDTH);
    desired_width.clamp(MIN_TREE_WIDTH, MAX_TREE_WIDTH.min(max_allowed))
}
