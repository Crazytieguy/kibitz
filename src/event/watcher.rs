use anyhow::Result;
use notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{DebouncedEvent, DebouncedEventKind, Debouncer, new_debouncer};
use std::path::Path;
use std::sync::mpsc::Sender;
use std::time::Duration;

pub struct FileWatcher {
    _debouncer: Debouncer<RecommendedWatcher>,
}

impl FileWatcher {
    pub fn new(repo_path: &Path, tx: Sender<()>) -> Result<Self> {
        let debouncer = new_debouncer(
            Duration::from_millis(200),
            move |res: Result<Vec<DebouncedEvent>, notify::Error>| {
                if let Ok(events) = res {
                    // Filter for relevant events
                    let has_relevant = events.iter().any(|e| {
                        matches!(e.kind, DebouncedEventKind::Any)
                    });
                    if has_relevant {
                        let _ = tx.send(());
                    }
                }
            },
        )?;

        let git_dir = repo_path.join(".git");
        let mut watcher = debouncer;

        // Watch .git directory for index changes
        watcher
            .watcher()
            .watch(&git_dir, RecursiveMode::Recursive)?;

        // Watch working directory for file changes (non-recursive to avoid perf issues)
        watcher
            .watcher()
            .watch(repo_path, RecursiveMode::NonRecursive)?;

        Ok(Self {
            _debouncer: watcher,
        })
    }
}
