use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // All variants defined for completeness
pub enum FileStatus {
    Modified,
    Added,
    Deleted,
    Renamed,
    Untracked,
    Staged,
    StagedModified, // Has both staged and unstaged changes
}

impl FileStatus {
    pub fn has_staged(&self) -> bool {
        matches!(self, FileStatus::Staged | FileStatus::StagedModified)
    }

    pub fn has_both(&self) -> bool {
        matches!(self, FileStatus::StagedModified)
    }
}

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub expanded: bool,
    pub status: Option<FileStatus>,
    pub children: Vec<TreeNode>,
}

impl TreeNode {
    pub fn new_file(name: String, path: PathBuf, status: FileStatus) -> Self {
        Self {
            name,
            path,
            is_dir: false,
            expanded: false,
            status: Some(status),
            children: Vec::new(),
        }
    }

    pub fn new_dir(name: String, path: PathBuf) -> Self {
        Self {
            name,
            path,
            is_dir: true,
            expanded: true,
            status: None,
            children: Vec::new(),
        }
    }
}

pub struct FileTree {
    pub root: Vec<TreeNode>,
    pub selected_index: usize,
    flat_list: Vec<FlatNode>,
    file_statuses: HashMap<PathBuf, FileStatus>,
    /// Tracks the last visited child path for each folder (for navigation memory)
    last_visited_child: HashMap<PathBuf, PathBuf>,
}

/// A flattened view of a tree node for display
#[derive(Debug, Clone)]
pub struct VisibleNode {
    pub name: String,
    pub depth: usize,
    pub is_dir: bool,
    pub expanded: bool,
    pub status: Option<FileStatus>,
}

/// A row in the horizontal tree view
#[derive(Debug, Clone)]
pub struct HorizontalRow {
    pub items: Vec<HorizontalItem>,
    #[allow(dead_code)] // Will be used for horizontal scrolling
    pub active_index: usize, // which item in this row is on the path to selected
}

/// An item in a horizontal row
#[derive(Debug, Clone)]
pub struct HorizontalItem {
    pub name: String,
    #[allow(dead_code)] // May be used for navigation or display
    pub path: PathBuf,
    pub is_dir: bool,
    pub status: Option<FileStatus>,
    pub is_on_path: bool,  // is this item an ancestor of selected?
    pub is_selected: bool, // is this the actual selected item?
}

#[derive(Debug, Clone)]
struct FlatNode {
    path: PathBuf,
    depth: usize,
    is_dir: bool,
    expanded: bool,
    name: String,
    status: Option<FileStatus>,
}

impl FileTree {
    pub fn from_git_status(repo_path: &Path) -> Result<Self> {
        let (files, file_statuses) = crate::git::status::get_status(repo_path)?;
        Ok(Self::from_files(files, file_statuses))
    }

    /// Build a FileTree from a list of files (used for commit file views)
    pub fn from_commit_files(files: Vec<(PathBuf, FileStatus)>) -> Self {
        let file_statuses: HashMap<PathBuf, FileStatus> = files.iter().cloned().collect();
        Self::from_files(files, file_statuses)
    }

    fn from_files(
        files: Vec<(PathBuf, FileStatus)>,
        file_statuses: HashMap<PathBuf, FileStatus>,
    ) -> Self {
        let mut children = Vec::new();

        for (path, status) in &files {
            Self::insert_path(&mut children, path, *status);
        }

        Self::sort_tree(&mut children);

        // Wrap everything in a "." root folder
        let root_node = TreeNode {
            name: ".".to_string(),
            path: PathBuf::from("."),
            is_dir: true,
            expanded: true,
            status: None,
            children,
        };

        // Prefix all file_statuses keys with "./" to match the tree paths
        let prefixed_statuses: HashMap<PathBuf, FileStatus> = file_statuses
            .into_iter()
            .map(|(path, status)| (PathBuf::from(".").join(&path), status))
            .collect();

        let mut tree = Self {
            root: vec![root_node],
            selected_index: 0,
            flat_list: Vec::new(),
            file_statuses: prefixed_statuses,
            last_visited_child: HashMap::new(),
        };

        tree.rebuild_flat_list();

        tree
    }

    fn insert_path(nodes: &mut Vec<TreeNode>, path: &Path, status: FileStatus) {
        let components: Vec<_> = path.components().collect();
        if components.is_empty() {
            return;
        }

        let mut current = nodes;
        // Start with "." so all paths are children of the root "." folder
        let mut current_path = PathBuf::from(".");

        for (i, component) in components.iter().enumerate() {
            let name = component.as_os_str().to_string_lossy().to_string();
            current_path.push(&name);
            let is_last = i == components.len() - 1;

            let pos = current.iter().position(|n| n.name == name);

            if is_last {
                // It's a file
                if pos.is_none() {
                    current.push(TreeNode::new_file(name, current_path.clone(), status));
                }
            } else {
                // It's a directory
                let idx = if let Some(idx) = pos {
                    idx
                } else {
                    current.push(TreeNode::new_dir(name, current_path.clone()));
                    current.len() - 1
                };
                current = &mut current[idx].children;
            }
        }
    }

    fn sort_tree(nodes: &mut Vec<TreeNode>) {
        nodes.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        });
        for node in nodes {
            Self::sort_tree(&mut node.children);
        }
    }

    fn rebuild_flat_list(&mut self) {
        self.flat_list.clear();
        Self::flatten_nodes(&self.root, 0, &mut self.flat_list);
    }

    fn flatten_nodes(nodes: &[TreeNode], depth: usize, flat: &mut Vec<FlatNode>) {
        for node in nodes {
            flat.push(FlatNode {
                path: node.path.clone(),
                depth,
                is_dir: node.is_dir,
                expanded: node.expanded,
                name: node.name.clone(),
                status: node.status,
            });
            if node.is_dir && node.expanded {
                Self::flatten_nodes(&node.children, depth + 1, flat);
            }
        }
    }

    pub fn visible_items(&self) -> Vec<VisibleNode> {
        self.flat_list
            .iter()
            .map(|n| VisibleNode {
                name: n.name.clone(),
                depth: n.depth,
                is_dir: n.is_dir,
                expanded: n.expanded,
                status: n.status,
            })
            .collect()
    }

    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.selected_index + 1 < self.flat_list.len() {
            self.selected_index += 1;
        }
    }

    pub fn expand(&mut self) {
        if let Some(node) = self.flat_list.get(self.selected_index) {
            if node.is_dir && !node.expanded {
                let path = node.path.clone();
                Self::set_expanded(&mut self.root, &path, true);
                self.rebuild_flat_list();
            } else {
                self.move_down();
            }
        }
    }

    pub fn collapse(&mut self) {
        if let Some(node) = self.flat_list.get(self.selected_index) {
            if node.is_dir && node.expanded {
                let path = node.path.clone();
                Self::set_expanded(&mut self.root, &path, false);
                self.rebuild_flat_list();
            } else if !node.is_dir || !node.expanded {
                // Go to parent directory
                if let Some(parent) = node.path.parent()
                    && !parent.as_os_str().is_empty()
                {
                    // Find parent in flat list
                    if let Some(idx) = self.flat_list.iter().position(|n| n.path == parent) {
                        self.selected_index = idx;
                    }
                }
            }
        }
    }

    fn set_expanded(nodes: &mut [TreeNode], path: &Path, expanded: bool) {
        for node in nodes {
            if node.path == path {
                node.expanded = expanded;
                return;
            }
            if node.is_dir {
                Self::set_expanded(&mut node.children, path, expanded);
            }
        }
    }

    pub fn selected_file_path(&self) -> Option<PathBuf> {
        self.flat_list
            .get(self.selected_index)
            .and_then(|n| if n.is_dir { None } else { Some(n.path.clone()) })
    }

    /// Returns the selected path (file or folder)
    pub fn selected_path(&self) -> Option<(PathBuf, bool)> {
        self.flat_list
            .get(self.selected_index)
            .map(|n| (n.path.clone(), n.is_dir))
    }

    /// Get all file paths under a folder (recursively)
    pub fn files_under_path(&self, folder_path: &Path) -> Vec<PathBuf> {
        self.file_statuses
            .keys()
            .filter(|p| p.starts_with(folder_path))
            .cloned()
            .collect()
    }

    pub fn get_file_status(&self, path: &Path) -> Option<FileStatus> {
        self.file_statuses.get(path).copied()
    }

    // === Horizontal navigation methods ===

    /// Build the rows for horizontal tree display
    pub fn get_horizontal_rows(&self) -> Vec<HorizontalRow> {
        let Some(selected) = self.flat_list.get(self.selected_index) else {
            return Vec::new();
        };

        let selected_path = &selected.path;

        // Collect all ancestor paths (including selected)
        let mut path_ancestors: Vec<PathBuf> = Vec::new();
        let mut current = selected_path.clone();
        path_ancestors.push(current.clone());
        while let Some(parent) = current.parent() {
            if parent.as_os_str().is_empty() {
                break;
            }
            path_ancestors.push(parent.to_path_buf());
            current = parent.to_path_buf();
        }
        path_ancestors.reverse(); // root to selected

        // Build rows by walking the tree
        let mut rows: Vec<HorizontalRow> = Vec::new();
        self.build_horizontal_rows(&self.root, &path_ancestors, selected_path, &mut rows);

        rows
    }

    fn build_horizontal_rows(
        &self,
        nodes: &[TreeNode],
        path_ancestors: &[PathBuf],
        selected_path: &Path,
        rows: &mut Vec<HorizontalRow>,
    ) {
        if nodes.is_empty() {
            return;
        }

        // Current depth = number of rows already built
        let depth = rows.len();

        // Find which node at this level is on the path (if any)
        let path_node = path_ancestors.get(depth);

        let mut items: Vec<HorizontalItem> = Vec::new();
        let mut active_index = 0;
        let mut next_level_nodes: Option<&[TreeNode]> = None;

        for (i, node) in nodes.iter().enumerate() {
            let is_on_path = path_node.is_some_and(|p| *p == node.path);
            let is_selected = node.path == selected_path;

            if is_on_path {
                active_index = i;
                // If this is a folder on the path, we'll recurse into its children
                if node.is_dir && !node.children.is_empty() {
                    next_level_nodes = Some(&node.children);
                }
            }

            items.push(HorizontalItem {
                name: node.name.clone(),
                path: node.path.clone(),
                is_dir: node.is_dir,
                status: node.status,
                is_on_path,
                is_selected,
            });
        }

        rows.push(HorizontalRow {
            items,
            active_index,
        });

        // Recurse into the next level if we have a folder on the path
        if let Some(children) = next_level_nodes {
            self.build_horizontal_rows(children, path_ancestors, selected_path, rows);
        }
    }

    /// Move to parent directory (k in horizontal mode)
    /// Remembers current position so move_to_child can return here
    pub fn move_to_parent(&mut self) {
        if let Some(node) = self.flat_list.get(self.selected_index)
            && let Some(parent) = node.path.parent()
            && !parent.as_os_str().is_empty()
            && let Some(idx) = self.flat_list.iter().position(|n| n.path == parent)
        {
            // Remember this child for when we come back down
            self.last_visited_child
                .insert(parent.to_path_buf(), node.path.clone());
            self.selected_index = idx;
        }
    }

    /// Move to child (j in horizontal mode, only works on expanded folders)
    /// Uses remembered child if available, otherwise first child
    pub fn move_to_child(&mut self) {
        if let Some(node) = self.flat_list.get(self.selected_index)
            && node.is_dir
            && node.expanded
        {
            let current_path = node.path.clone();
            let current_depth = node.depth;

            // Check if we have a remembered child for this folder
            if let Some(remembered) = self.last_visited_child.get(&current_path)
                && let Some(idx) = self.flat_list.iter().position(|n| n.path == *remembered)
            {
                self.selected_index = idx;
                return;
            }

            // Otherwise go to first child
            if self.selected_index + 1 < self.flat_list.len() {
                let next = &self.flat_list[self.selected_index + 1];
                if next.depth > current_depth {
                    self.selected_index += 1;
                }
            }
        }
    }

    /// Move to previous sibling (h in horizontal mode)
    /// If at first sibling, jump to last child of previous uncle (cousin navigation)
    pub fn move_to_prev_sibling(&mut self) {
        if let Some(node) = self.flat_list.get(self.selected_index) {
            let target_depth = node.depth;
            let parent_path = node.path.parent().map(|p| p.to_path_buf());

            // Search backwards for a sibling at same depth with same parent
            for i in (0..self.selected_index).rev() {
                let candidate = &self.flat_list[i];
                if candidate.depth == target_depth {
                    let candidate_parent = candidate.path.parent().map(|p| p.to_path_buf());
                    if candidate_parent == parent_path {
                        self.selected_index = i;
                        return;
                    }
                }
                // If we hit a shallower node, stop searching for direct siblings
                if candidate.depth < target_depth {
                    break;
                }
            }

            // No previous sibling found - try cousin navigation
            // Find the previous uncle (parent's previous sibling) and go to its last child
            if let Some(parent) = &parent_path
                && let Some(parent_idx) = self.flat_list.iter().position(|n| n.path == *parent)
            {
                // Find parent's previous sibling
                let parent_depth = self.flat_list[parent_idx].depth;
                let grandparent = parent.parent().map(|p| p.to_path_buf());

                for i in (0..parent_idx).rev() {
                    let candidate = &self.flat_list[i];
                    if candidate.depth == parent_depth && candidate.is_dir && candidate.expanded {
                        let candidate_parent = candidate.path.parent().map(|p| p.to_path_buf());
                        if candidate_parent == grandparent {
                            // Found previous uncle - go to its last child at target_depth
                            if let Some(last_child) = self.find_last_child_at_depth(i, target_depth)
                            {
                                self.selected_index = last_child;
                                return;
                            }
                        }
                    }
                    if candidate.depth < parent_depth {
                        break;
                    }
                }
            }
        }
    }

    /// Move to next sibling (l in horizontal mode)
    /// If at last sibling, jump to first child of next uncle (cousin navigation)
    pub fn move_to_next_sibling(&mut self) {
        if let Some(node) = self.flat_list.get(self.selected_index) {
            let target_depth = node.depth;
            let parent_path = node.path.parent().map(|p| p.to_path_buf());

            // Search forwards for a sibling at same depth with same parent
            for i in (self.selected_index + 1)..self.flat_list.len() {
                let candidate = &self.flat_list[i];
                if candidate.depth == target_depth {
                    let candidate_parent = candidate.path.parent().map(|p| p.to_path_buf());
                    if candidate_parent == parent_path {
                        self.selected_index = i;
                        return;
                    }
                }
                // If we hit a shallower node, stop searching for direct siblings
                if candidate.depth < target_depth {
                    break;
                }
            }

            // No next sibling found - try cousin navigation
            // Find the next uncle (parent's next sibling) and go to its first child
            if let Some(parent) = &parent_path
                && let Some(parent_idx) = self.flat_list.iter().position(|n| n.path == *parent)
            {
                // Find parent's next sibling
                let parent_depth = self.flat_list[parent_idx].depth;
                let grandparent = parent.parent().map(|p| p.to_path_buf());

                for i in (parent_idx + 1)..self.flat_list.len() {
                    let candidate = &self.flat_list[i];
                    if candidate.depth == parent_depth && candidate.is_dir && candidate.expanded {
                        let candidate_parent = candidate.path.parent().map(|p| p.to_path_buf());
                        if candidate_parent == grandparent {
                            // Found next uncle - go to its first child at target_depth
                            if let Some(first_child) =
                                self.find_first_child_at_depth(i, target_depth)
                            {
                                self.selected_index = first_child;
                                return;
                            }
                        }
                    }
                    if candidate.depth < parent_depth {
                        break;
                    }
                }
            }
        }
    }

    /// Find the first descendant at a specific depth
    fn find_first_child_at_depth(&self, parent_idx: usize, target_depth: usize) -> Option<usize> {
        let parent_depth = self.flat_list[parent_idx].depth;
        for i in (parent_idx + 1)..self.flat_list.len() {
            let node = &self.flat_list[i];
            if node.depth <= parent_depth {
                break;
            }
            if node.depth == target_depth {
                return Some(i);
            }
        }
        None
    }

    /// Find the last descendant at a specific depth
    fn find_last_child_at_depth(&self, parent_idx: usize, target_depth: usize) -> Option<usize> {
        let parent_depth = self.flat_list[parent_idx].depth;
        let mut last_found = None;
        for i in (parent_idx + 1)..self.flat_list.len() {
            let node = &self.flat_list[i];
            if node.depth <= parent_depth {
                break;
            }
            if node.depth == target_depth {
                last_found = Some(i);
            }
        }
        last_found
    }
}
