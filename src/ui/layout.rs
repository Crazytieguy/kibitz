use crate::config::LayoutMode;
use crate::model::FileTree;
use ratatui::layout::{Constraint, Direction, Layout, Rect};

const MIN_TREE_WIDTH: u16 = 20;
const MAX_TREE_WIDTH: u16 = 50;
const TREE_PADDING: u16 = 4; // For icon, spacing, and border

pub struct Areas {
    pub tree: Rect,
    pub diff: Rect,
    pub hint: Rect,
}

pub fn create_layout_for_mode(
    area: Rect,
    show_tree: bool,
    file_tree: &FileTree,
    mode: LayoutMode,
    max_rows: u16,
) -> Areas {
    let (main_area, hint) = split_hint_area(area);

    match mode {
        LayoutMode::Vertical => create_vertical_areas(main_area, hint, show_tree, file_tree),
        LayoutMode::Horizontal => {
            create_horizontal_areas(main_area, hint, show_tree, file_tree, max_rows)
        }
    }
}

fn split_hint_area(area: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area);
    (chunks[0], chunks[1])
}

fn create_vertical_areas(
    main_area: Rect,
    hint: Rect,
    show_tree: bool,
    file_tree: &FileTree,
) -> Areas {
    if show_tree {
        let tree_width = calculate_tree_width(file_tree, main_area.width);
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(tree_width), Constraint::Min(1)])
            .split(main_area);

        Areas {
            tree: chunks[0],
            diff: chunks[1],
            hint,
        }
    } else {
        Areas {
            tree: Rect::default(),
            diff: main_area,
            hint,
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

fn create_horizontal_areas(
    main_area: Rect,
    hint: Rect,
    show_tree: bool,
    file_tree: &FileTree,
    max_rows: u16,
) -> Areas {
    if show_tree {
        let row_count = file_tree.get_horizontal_rows().len() as u16;
        let tree_height = (row_count + 2).min(max_rows + 2);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(tree_height)])
            .split(main_area);

        Areas {
            diff: chunks[0],
            tree: chunks[1],
            hint,
        }
    } else {
        Areas {
            tree: Rect::default(),
            diff: main_area,
            hint,
        }
    }
}
