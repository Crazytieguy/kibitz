use crate::model::FileStatus;
use anyhow::{Context, Result};
use git2::{Repository, StatusOptions};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub type GitStatusResult = (Vec<(PathBuf, FileStatus)>, HashMap<PathBuf, FileStatus>);

pub fn find_repo_root() -> Result<PathBuf> {
    let current_dir = std::env::current_dir()?;
    let repo = Repository::discover(&current_dir)
        .context("Not a git repository (or any parent up to mount point)")?;
    let workdir = repo
        .workdir()
        .context("Repository has no working directory")?;
    Ok(workdir.to_path_buf())
}

pub fn get_status(repo_path: &Path) -> Result<GitStatusResult> {
    let repo = Repository::open(repo_path)?;

    let mut opts = StatusOptions::new();
    opts.include_untracked(true)
        .recurse_untracked_dirs(true)
        .include_ignored(false);

    let statuses = repo.statuses(Some(&mut opts))?;

    let mut files = Vec::new();
    let mut file_map = HashMap::new();

    for entry in statuses.iter() {
        let path = match entry.path() {
            Some(p) => PathBuf::from(p),
            None => continue,
        };

        let status = entry.status();
        let file_status = convert_status(status);

        if let Some(fs) = file_status {
            files.push((path.clone(), fs));
            file_map.insert(path, fs);
        }
    }

    Ok((files, file_map))
}

fn convert_status(status: git2::Status) -> Option<FileStatus> {
    let has_index_change = status.intersects(
        git2::Status::INDEX_NEW
            | git2::Status::INDEX_MODIFIED
            | git2::Status::INDEX_DELETED
            | git2::Status::INDEX_RENAMED,
    );

    let has_worktree_change = status.intersects(
        git2::Status::WT_NEW
            | git2::Status::WT_MODIFIED
            | git2::Status::WT_DELETED
            | git2::Status::WT_RENAMED,
    );

    if has_index_change && has_worktree_change {
        Some(FileStatus::StagedModified)
    } else if has_index_change {
        Some(FileStatus::Staged)
    } else if status.contains(git2::Status::WT_NEW) {
        Some(FileStatus::Untracked)
    } else if status.contains(git2::Status::WT_MODIFIED) {
        Some(FileStatus::Modified)
    } else if status.contains(git2::Status::WT_DELETED) {
        Some(FileStatus::Deleted)
    } else if status.contains(git2::Status::WT_RENAMED) {
        Some(FileStatus::Renamed)
    } else {
        None
    }
}
