use crate::model::{DiffState, FileStatus};
use ansi_to_tui::IntoText;
use anyhow::Result;
use ratatui::text::Text;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;

pub fn delta_available() -> bool {
    Command::new("delta")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Request to load a diff asynchronously
pub struct DiffRequest {
    pub repo_path: std::path::PathBuf,
    pub file_path: std::path::PathBuf,
    pub status: Option<FileStatus>,
    pub width: usize,
    pub staged: bool,
    pub delta_args: Option<String>,
}

/// Spawn a function on a thread and return a receiver for the result
fn spawn_diff<F>(f: F) -> mpsc::Receiver<DiffState>
where
    F: FnOnce() -> Result<DiffState> + Send + 'static,
{
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let _ = tx.send(f().unwrap_or_default());
    });
    rx
}

/// Spawn async diff loading, returns a receiver for the result
pub fn load_diff_async(req: DiffRequest) -> mpsc::Receiver<DiffState> {
    spawn_diff(move || get_diff_sync(&req))
}

fn get_diff_sync(req: &DiffRequest) -> Result<DiffState> {
    let diff_cmd = build_diff_command(req);
    let has_both = req.status.is_some_and(|s| s.has_both());
    run_diff_command(&req.repo_path, &diff_cmd, req.width, has_both, req.staged)
}

/// Find hunk positions in delta output by looking for file headers (Δ) or hunk markers (•)
fn find_hunk_positions(content: &Text) -> Vec<usize> {
    let mut positions = Vec::new();

    for (i, line) in content.lines.iter().enumerate() {
        // Get the raw text content of the line
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        let trimmed = text.trim_start();

        // Delta uses "Δ" (U+0394) for file headers and "•" (U+2022) for hunk markers
        if trimmed.starts_with('Δ') || trimmed.starts_with('•') {
            positions.push(i);
        }
    }

    positions
}

fn build_diff_command(req: &DiffRequest) -> String {
    let file_path = req.file_path.to_string_lossy();
    let user_args = req.delta_args.as_deref().unwrap_or("");

    match req.status {
        Some(FileStatus::Untracked) => {
            // For untracked files, show content as new file
            format!(
                "git diff --no-index --color=always -- /dev/null '{}' 2>/dev/null | delta --paging=never {} || cat '{}'",
                file_path, user_args, file_path
            )
        }
        Some(s) if s.has_staged() && req.staged => {
            format!(
                "git diff --cached --color=always -- '{}' | delta --paging=never {}",
                file_path, user_args
            )
        }
        _ => {
            format!(
                "git diff --color=always -- '{}' | delta --paging=never {}",
                file_path, user_args
            )
        }
    }
}

pub fn get_diff(
    repo_path: &Path,
    file_path: &Path,
    status: Option<FileStatus>,
    width: usize,
    delta_args: Option<String>,
) -> mpsc::Receiver<DiffState> {
    // Default: show unstaged if file has both, otherwise show staged if only staged
    let staged = status.is_some_and(|s| !s.has_both() && s.has_staged());
    get_diff_staged(repo_path, file_path, status, width, staged, delta_args)
}

pub fn get_diff_staged(
    repo_path: &Path,
    file_path: &Path,
    status: Option<FileStatus>,
    width: usize,
    staged: bool,
    delta_args: Option<String>,
) -> mpsc::Receiver<DiffState> {
    load_diff_async(DiffRequest {
        repo_path: repo_path.to_path_buf(),
        file_path: file_path.to_path_buf(),
        status,
        width,
        staged,
        delta_args,
    })
}

/// Get combined diff for multiple files (used for folder diffs)
pub fn get_diff_for_paths(
    repo_path: &Path,
    file_paths: &[std::path::PathBuf],
    width: usize,
    delta_args: Option<String>,
) -> mpsc::Receiver<DiffState> {
    let repo_path = repo_path.to_path_buf();
    let file_paths = file_paths.to_vec();
    spawn_diff(move || get_multi_diff_sync(&repo_path, &file_paths, width, delta_args.as_deref()))
}

/// Build shell-quoted file path arguments
fn quote_paths(paths: &[std::path::PathBuf]) -> String {
    paths
        .iter()
        .map(|p| format!("'{}'", p.to_string_lossy()))
        .collect::<Vec<_>>()
        .join(" ")
}

fn get_multi_diff_sync(
    repo_path: &Path,
    file_paths: &[std::path::PathBuf],
    width: usize,
    delta_args: Option<&str>,
) -> Result<DiffState> {
    if file_paths.is_empty() {
        return Ok(DiffState::new());
    }

    let user_args = delta_args.unwrap_or("");
    let diff_cmd = format!(
        "git diff --color=always -- {} | delta --paging=never {}",
        quote_paths(file_paths),
        user_args
    );

    run_diff_command(repo_path, &diff_cmd, width, false, false)
}

/// Run a diff command and convert output to DiffState
fn run_diff_command(
    repo_path: &Path,
    diff_cmd: &str,
    width: usize,
    has_both: bool,
    showing_staged: bool,
) -> Result<DiffState> {
    let output = Command::new("sh")
        .args(["-c", diff_cmd])
        .current_dir(repo_path)
        .env("TERM", "xterm-256color")
        .env("COLUMNS", width.to_string())
        .env("FORCE_COLOR", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()?;

    let content = output.stdout.into_text().unwrap_or_default();
    let total_lines = content.lines.len();
    let hunk_positions = find_hunk_positions(&content);

    Ok(DiffState {
        content,
        scroll_offset: 0,
        hunk_positions,
        current_hunk: 0,
        total_lines,
        has_both,
        showing_staged,
    })
}

/// Get diff for a specific file within a commit
pub fn get_commit_file_diff(
    repo_path: &Path,
    oid: &str,
    file_path: &Path,
    width: usize,
    delta_args: Option<String>,
) -> mpsc::Receiver<DiffState> {
    let repo_path = repo_path.to_path_buf();
    let oid = oid.to_string();
    let file_path = file_path.to_path_buf();
    spawn_diff(move || get_commit_file_diff_sync(&repo_path, &oid, &file_path, width, delta_args.as_deref()))
}

fn get_commit_file_diff_sync(
    repo_path: &Path,
    oid: &str,
    file_path: &Path,
    width: usize,
    delta_args: Option<&str>,
) -> Result<DiffState> {
    let user_args = delta_args.unwrap_or("");
    let diff_cmd = format!(
        "git show --format='' --color=always {} -- '{}' | delta --paging=never {}",
        oid,
        file_path.to_string_lossy(),
        user_args
    );
    run_diff_command(repo_path, &diff_cmd, width, false, false)
}

/// Get diff for multiple files within a commit (used for folder diffs in history)
pub fn get_commit_files_diff(
    repo_path: &Path,
    oid: &str,
    file_paths: &[std::path::PathBuf],
    width: usize,
    delta_args: Option<String>,
) -> mpsc::Receiver<DiffState> {
    let repo_path = repo_path.to_path_buf();
    let oid = oid.to_string();
    let file_paths = file_paths.to_vec();
    spawn_diff(move || get_commit_files_diff_sync(&repo_path, &oid, &file_paths, width, delta_args.as_deref()))
}

fn get_commit_files_diff_sync(
    repo_path: &Path,
    oid: &str,
    file_paths: &[std::path::PathBuf],
    width: usize,
    delta_args: Option<&str>,
) -> Result<DiffState> {
    if file_paths.is_empty() {
        return Ok(DiffState::new());
    }

    let user_args = delta_args.unwrap_or("");
    let diff_cmd = format!(
        "git show --format='' --color=always {} -- {} | delta --paging=never {}",
        oid,
        quote_paths(file_paths),
        user_args
    );
    run_diff_command(repo_path, &diff_cmd, width, false, false)
}
