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

/// Spawn async diff loading, returns a receiver for the result
pub fn load_diff_async(req: DiffRequest) -> mpsc::Receiver<DiffState> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let result = get_diff_sync(&req);
        let _ = tx.send(result.unwrap_or_default());
    });

    rx
}

fn get_diff_sync(req: &DiffRequest) -> Result<DiffState> {
    let diff_cmd = build_diff_command(req);
    let has_both = req.status.is_some_and(|s| s.has_both());
    run_diff_command(&req.repo_path, &diff_cmd, req.width, has_both, req.staged)
}

/// Filter out control characters and script artifacts from output
fn filter_control_chars(input: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(input.len());
    let mut i = 0;

    // Skip leading ^D and backspaces that script adds
    while i < input.len() {
        if input[i] == b'^' && i + 1 < input.len() && input[i + 1] == b'D' {
            i += 2;
            continue;
        }
        if input[i] == 0x08 {
            // backspace
            i += 1;
            continue;
        }
        break;
    }

    while i < input.len() {
        // Skip OSC sequences: ESC ] ... (terminated by BEL or ST)
        if i + 1 < input.len() && input[i] == 0x1b && input[i + 1] == b']' {
            i += 2;
            while i < input.len() {
                if input[i] == 0x07 {
                    i += 1;
                    break;
                }
                if i + 1 < input.len() && input[i] == 0x1b && input[i + 1] == b'\\' {
                    i += 2;
                    break;
                }
                i += 1;
            }
            continue;
        }

        // Skip CSI device attribute queries: ESC [ ... c
        if i + 1 < input.len() && input[i] == 0x1b && input[i + 1] == b'[' {
            let start = i;
            i += 2;
            while i < input.len() && (input[i] < 0x40 || input[i] > 0x7e) {
                i += 1;
            }
            if i < input.len() {
                if input[i] == b'c' {
                    i += 1;
                    continue;
                }
                result.extend_from_slice(&input[start..=i]);
                i += 1;
                continue;
            }
        }

        result.push(input[i]);
        i += 1;
    }

    result
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
    let (tx, rx) = mpsc::channel();
    let repo_path = repo_path.to_path_buf();
    let file_paths: Vec<_> = file_paths.to_vec();

    thread::spawn(move || {
        let result = get_multi_diff_sync(&repo_path, &file_paths, width, delta_args.as_deref());
        let _ = tx.send(result.unwrap_or_default());
    });

    rx
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
    let files_arg: Vec<_> = file_paths
        .iter()
        .map(|p| format!("'{}'", p.to_string_lossy()))
        .collect();

    let diff_cmd = format!(
        "git diff --color=always -- {} | delta --paging=never {}",
        files_arg.join(" "),
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
    let output = Command::new("script")
        .args(["-q", "/dev/null", "sh", "-c", diff_cmd])
        .current_dir(repo_path)
        .env("TERM", "xterm-256color")
        .env("COLUMNS", width.to_string())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()?;

    let stdout = filter_control_chars(&output.stdout);
    let content = stdout.into_text().unwrap_or_default();
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

/// Get diff for a specific commit (comparing to its parent)
pub fn get_commit_diff(
    repo_path: &Path,
    oid: &str,
    width: usize,
    delta_args: Option<String>,
) -> mpsc::Receiver<DiffState> {
    let (tx, rx) = mpsc::channel();
    let repo_path = repo_path.to_path_buf();
    let oid = oid.to_string();

    thread::spawn(move || {
        let result = get_commit_diff_sync(&repo_path, &oid, width, delta_args.as_deref());
        let _ = tx.send(result.unwrap_or_default());
    });

    rx
}

fn get_commit_diff_sync(
    repo_path: &Path,
    oid: &str,
    width: usize,
    delta_args: Option<&str>,
) -> Result<DiffState> {
    let user_args = delta_args.unwrap_or("");
    let diff_cmd = format!(
        "git show --format='' --color=always {} | delta --paging=never {}",
        oid, user_args
    );
    run_diff_command(repo_path, &diff_cmd, width, false, false)
}

/// Get diff for a specific file within a commit
pub fn get_commit_file_diff(
    repo_path: &Path,
    oid: &str,
    file_path: &Path,
    width: usize,
    delta_args: Option<String>,
) -> mpsc::Receiver<DiffState> {
    let (tx, rx) = mpsc::channel();
    let repo_path = repo_path.to_path_buf();
    let oid = oid.to_string();
    let file_path = file_path.to_path_buf();

    thread::spawn(move || {
        let result = get_commit_file_diff_sync(&repo_path, &oid, &file_path, width, delta_args.as_deref());
        let _ = tx.send(result.unwrap_or_default());
    });

    rx
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
