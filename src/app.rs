use crate::config::Config;
use crate::event::{self, watcher::FileWatcher};
use crate::git;
use crate::model::{DiffState, FileTree};
use crate::ui;
use anyhow::Result;
use crossterm::event::{self as ct_event, Event};
use ratatui::prelude::*;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

pub struct App {
    pub file_tree: FileTree,
    pub diff_state: DiffState,
    pub show_tree: bool,
    pub repo_path: PathBuf,
    pub config: Config,
    #[allow(dead_code)]
    file_watcher: FileWatcher,
    watcher_rx: mpsc::Receiver<()>,
    terminal_size: (u16, u16),
    pending_diff: Option<mpsc::Receiver<DiffState>>,
}

impl App {
    pub fn new() -> Result<Self> {
        let repo_path = git::status::find_repo_root()?;
        let config = Config::load(&repo_path);
        let file_tree = FileTree::from_git_status(&repo_path)?;

        let (tx, rx) = mpsc::channel();
        let watcher = FileWatcher::new(&repo_path, tx)?;

        let app = Self {
            file_tree,
            diff_state: DiffState::new(),
            show_tree: true,
            repo_path,
            config,
            file_watcher: watcher,
            watcher_rx: rx,
            terminal_size: (0, 0),
            pending_diff: None,
        };

        Ok(app)
    }

    pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
        // Get initial size and load first diff
        let size = terminal.size()?;
        self.terminal_size = (size.width, size.height);
        self.request_diff();

        loop {
            // Check for completed async diff
            if let Some(ref rx) = self.pending_diff
                && let Ok(diff) = rx.try_recv()
            {
                self.diff_state = diff;
                self.pending_diff = None;
            }

            // Check for file system changes
            if self.watcher_rx.try_recv().is_ok() {
                self.refresh()?;
            }

            // Check for resize
            let size = terminal.size()?;
            if (size.width, size.height) != self.terminal_size {
                self.terminal_size = (size.width, size.height);
                self.request_diff();
            }

            terminal.draw(|frame| ui::render(frame, self))?;

            // Short poll timeout for responsive UI
            if ct_event::poll(Duration::from_millis(16))? {
                match ct_event::read()? {
                    Event::Key(key) => {
                        if event::handle_key(self, key)? {
                            break;
                        }
                    }
                    Event::Mouse(mouse) => {
                        event::handle_mouse(self, mouse)?;
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.file_tree = FileTree::from_git_status(&self.repo_path)?;
        self.request_diff();
        Ok(())
    }

    fn get_diff_width(&self) -> usize {
        if self.show_tree {
            // Estimate based on typical tree width
            self.terminal_size.0.saturating_sub(35) as usize
        } else {
            self.terminal_size.0 as usize
        }
    }

    pub fn request_diff(&mut self) {
        let diff_width = self.get_diff_width();

        if let Some((path, is_dir)) = self.file_tree.selected_path() {
            if is_dir {
                // Folder selected - get all files under it
                let files = self.file_tree.files_under_path(&path);
                if files.is_empty() {
                    self.diff_state = DiffState::new();
                    self.pending_diff = None;
                } else {
                    let rx = git::diff::get_diff_for_paths(
                        &self.repo_path,
                        &files,
                        diff_width,
                        self.config.delta.args.clone(),
                    );
                    self.pending_diff = Some(rx);
                }
            } else {
                // Single file
                let status = self.file_tree.get_file_status(&path);
                let rx = git::diff::get_diff(
                    &self.repo_path,
                    &path,
                    status,
                    diff_width,
                    self.config.delta.args.clone(),
                );
                self.pending_diff = Some(rx);
            }
        } else {
            self.diff_state = DiffState::new();
            self.pending_diff = None;
        }
    }

    pub fn request_diff_staged(&mut self, staged: bool) {
        if let Some(path) = self.file_tree.selected_file_path() {
            let status = self.file_tree.get_file_status(&path);
            let diff_width = self.get_diff_width();

            let rx = git::diff::get_diff_staged(
                &self.repo_path,
                &path,
                status,
                diff_width,
                staged,
                self.config.delta.args.clone(),
            );
            self.pending_diff = Some(rx);
        }
    }

    pub fn toggle_tree(&mut self) {
        self.show_tree = !self.show_tree;
        self.request_diff();
    }

    pub fn toggle_staged(&mut self) {
        if self.diff_state.has_both {
            let new_staged = !self.diff_state.showing_staged;
            self.diff_state.showing_staged = new_staged;
            self.request_diff_staged(new_staged);
        }
    }

    pub fn navigate_tree(&mut self, navigate_fn: impl FnOnce(&mut FileTree)) {
        let prev_path = self.file_tree.selected_path();
        navigate_fn(&mut self.file_tree);
        if self.file_tree.selected_path() != prev_path {
            self.request_diff();
        }
    }
}
