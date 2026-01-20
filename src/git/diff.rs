use crate::model::{DiffState, FileStatus};
use ansi_to_tui::IntoText;
use anyhow::Result;
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
    // Build the diff command
    let diff_cmd = build_diff_command(req);

    // Use script to fake a TTY for delta's color output
    // script -q /dev/null runs the command in a pseudo-terminal
    let output = Command::new("script")
        .args(["-q", "/dev/null", "sh", "-c", &diff_cmd])
        .current_dir(&req.repo_path)
        .env("TERM", "xterm-256color")
        .env("COLUMNS", req.width.to_string())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()?;

    // Filter out control characters that script adds (like ^D and terminal queries)
    let stdout = filter_control_chars(&output.stdout);

    // Convert ANSI to ratatui Text
    let content = stdout.into_text().unwrap_or_default();
    let total_lines = content.lines.len();

    let has_both = req.status.map(|s| s.has_both()).unwrap_or(false);

    Ok(DiffState {
        content,
        scroll_offset: 0,
        hunk_positions: Vec::new(),
        current_hunk: 0,
        total_lines,
        has_both,
        showing_staged: req.staged,
    })
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

fn build_diff_command(req: &DiffRequest) -> String {
    let file_path = req.file_path.to_string_lossy();

    match req.status {
        Some(FileStatus::Untracked) => {
            // For untracked files, show content as new file
            format!(
                "git diff --no-index --color=always -- /dev/null '{}' 2>/dev/null | delta --paging=never || cat '{}'",
                file_path, file_path
            )
        }
        Some(s) if s.has_staged() && req.staged => {
            format!(
                "git diff --cached --color=always -- '{}' | delta --paging=never",
                file_path
            )
        }
        _ => {
            format!(
                "git diff --color=always -- '{}' | delta --paging=never",
                file_path
            )
        }
    }
}

pub fn get_diff(
    repo_path: &Path,
    file_path: &Path,
    status: Option<FileStatus>,
    width: usize,
) -> mpsc::Receiver<DiffState> {
    let staged = match status {
        Some(s) => {
            if s.has_both() {
                false
            } else {
                s.has_staged()
            }
        }
        None => false,
    };

    load_diff_async(DiffRequest {
        repo_path: repo_path.to_path_buf(),
        file_path: file_path.to_path_buf(),
        status,
        width,
        staged,
    })
}

pub fn get_diff_staged(
    repo_path: &Path,
    file_path: &Path,
    status: Option<FileStatus>,
    width: usize,
    staged: bool,
) -> mpsc::Receiver<DiffState> {
    load_diff_async(DiffRequest {
        repo_path: repo_path.to_path_buf(),
        file_path: file_path.to_path_buf(),
        status,
        width,
        staged,
    })
}
