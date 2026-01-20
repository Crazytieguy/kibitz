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
}

#[derive(Debug, Clone)]
struct FlatNode {
    pub path: PathBuf,
    pub depth: usize,
    pub is_dir: bool,
    pub expanded: bool,
    pub name: String,
    pub status: Option<FileStatus>,
}

impl FileTree {
    pub fn from_git_status(repo_path: &Path) -> Result<Self> {
        let (files, file_statuses) = crate::git::status::get_status(repo_path)?;

        let mut root = Vec::new();

        for (path, status) in &files {
            Self::insert_path(&mut root, path, *status);
        }

        Self::sort_tree(&mut root);

        let mut tree = Self {
            root,
            selected_index: 0,
            flat_list: Vec::new(),
            file_statuses,
        };

        tree.rebuild_flat_list();

        Ok(tree)
    }

    fn insert_path(nodes: &mut Vec<TreeNode>, path: &Path, status: FileStatus) {
        let components: Vec<_> = path.components().collect();
        if components.is_empty() {
            return;
        }

        let mut current = nodes;
        let mut current_path = PathBuf::new();

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
        nodes.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
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

    pub fn visible_items(&self) -> Vec<(String, usize, bool, bool, Option<FileStatus>)> {
        self.flat_list
            .iter()
            .map(|n| (n.name.clone(), n.depth, n.is_dir, n.expanded, n.status))
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
        if let Some(node) = self.flat_list.get(self.selected_index)
            && node.is_dir
            && !node.expanded
        {
            let path = node.path.clone();
            Self::set_expanded(&mut self.root, &path, true);
            self.rebuild_flat_list();
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
        self.flat_list.get(self.selected_index).and_then(|n| {
            if n.is_dir {
                None
            } else {
                Some(n.path.clone())
            }
        })
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
}
