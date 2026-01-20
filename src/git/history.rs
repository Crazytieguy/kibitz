use crate::model::{CommitInfo, FileStatus};
use anyhow::Result;
use git2::Repository;
use std::path::{Path, PathBuf};

/// Get commit at offset from HEAD (0 = HEAD, 1 = HEAD~1, etc.)
/// Returns None if the offset is beyond available history.
pub fn get_commit_at(repo_path: &Path, offset: usize) -> Result<Option<CommitInfo>> {
    let repo = Repository::open(repo_path)?;

    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;

    let oid = match revwalk.nth(offset) {
        Some(Ok(oid)) => oid,
        _ => return Ok(None),
    };

    let commit = repo.find_commit(oid)?;
    let message = commit
        .message()
        .unwrap_or("")
        .lines()
        .next()
        .unwrap_or("")
        .to_string();

    Ok(Some(CommitInfo {
        oid: format!("{:.7}", oid),
        oid_full: oid.to_string(),
        message,
    }))
}

/// Get files changed in a commit (comparing to its parent).
/// Returns a list of (path, status) pairs suitable for building a FileTree.
pub fn get_commit_files(repo_path: &Path, oid: &str) -> Result<Vec<(PathBuf, FileStatus)>> {
    let repo = Repository::open(repo_path)?;
    let oid = git2::Oid::from_str(oid)?;
    let commit = repo.find_commit(oid)?;

    let tree = commit.tree()?;
    let parent_tree = commit.parent(0).ok().and_then(|p| p.tree().ok());

    let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None)?;

    let mut files = Vec::new();

    for delta in diff.deltas() {
        let path = delta
            .new_file()
            .path()
            .or_else(|| delta.old_file().path())
            .map(|p| p.to_path_buf())
            .unwrap_or_default();

        let status = match delta.status() {
            git2::Delta::Added => FileStatus::Added,
            git2::Delta::Deleted => FileStatus::Deleted,
            git2::Delta::Renamed => FileStatus::Renamed,
            _ => FileStatus::Modified,
        };

        files.push((path, status));
    }

    Ok(files)
}
