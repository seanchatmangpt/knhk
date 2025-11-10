//! File watcher for monitoring code changes
//!
//! Uses `notify` crate to watch for changes to Rust source files.
//! Triggers rebuild when code changes are detected.

use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use tokio::sync::mpsc as tokio_mpsc;
use crate::TestCacheError;

/// File watcher that monitors Rust source files for changes
pub struct FileWatcher {
    /// Watcher instance
    watcher: RecommendedWatcher,
    /// Event receiver
    receiver: mpsc::Receiver<notify::Result<Event>>,
    /// Watch directory
    watch_dir: PathBuf,
}

impl FileWatcher {
    /// Create a new file watcher
    pub fn new(watch_dir: PathBuf) -> Result<Self, TestCacheError> {
        let (tx, rx) = mpsc::channel();
        
        let config = Config::default()
            .with_poll_interval(std::time::Duration::from_millis(500))
            .with_compare_contents(true);
        
        let mut watcher = RecommendedWatcher::new(tx, config)?;
        watcher.watch(&watch_dir, RecursiveMode::Recursive)?;

        Ok(Self {
            watcher,
            receiver: rx,
            watch_dir,
        })
    }

    /// Watch for changes and send events to async channel
    pub fn watch_async(
        &mut self,
        event_tx: tokio_mpsc::UnboundedSender<PathBuf>,
    ) -> Result<(), TestCacheError> {
        // Debounce: collect events for 1 second before processing
        let mut pending_changes = Vec::new();
        let mut last_event_time = std::time::Instant::now();
        let debounce_duration = std::time::Duration::from_secs(1);

        loop {
            // Check for events with timeout
            match self.receiver.recv_timeout(std::time::Duration::from_millis(100)) {
                Ok(Ok(event)) => {
                    if self.is_rust_file_change(&event) {
                        pending_changes.push(event);
                        last_event_time = std::time::Instant::now();
                    }
                }
                Ok(Err(e)) => {
                    return Err(TestCacheError::WatcherError(e));
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // Check if debounce period has elapsed
                    if !pending_changes.is_empty() && last_event_time.elapsed() >= debounce_duration {
                        // Process pending changes
                        let changed_files = self.extract_changed_files(&pending_changes);
                        for file in changed_files {
                            let _ = event_tx.send(file);
                        }
                        pending_changes.clear();
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    break;
                }
            }
        }

        Ok(())
    }

    /// Check if event is a Rust file change
    fn is_rust_file_change(&self, event: &Event) -> bool {
        match &event.kind {
            EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                event.paths.iter().any(|p| {
                    p.extension()
                        .and_then(|s| s.to_str())
                        .map(|ext| ext == "rs")
                        .unwrap_or(false)
                })
            }
            _ => false,
        }
    }

    /// Extract changed file paths from events
    fn extract_changed_files(&self, events: &[Event]) -> Vec<PathBuf> {
        let mut files = Vec::new();
        
        for event in events {
            for path in &event.paths {
                // Skip excluded paths
                if self.is_excluded(path) {
                    continue;
                }
                
                if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                    files.push(path.clone());
                }
            }
        }
        
        files
    }

    /// Check if path should be excluded from watching
    fn is_excluded(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        path_str.contains("target/")
            || path_str.contains(".git/")
            || path_str.contains(".test-cache/")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    use std::time::Duration;
    use tokio::time::timeout;

    #[tokio::test]
    async fn test_watcher_detects_file_changes() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir_all(&src_dir).unwrap();

        let mut watcher = FileWatcher::new(temp_dir.path().to_path_buf()).unwrap();
        let (tx, mut rx) = tokio_mpsc::unbounded_channel();

        // Start watching in background
        let watch_dir = temp_dir.path().to_path_buf();
        tokio::spawn(async move {
            let _ = watcher.watch_async(tx);
        });

        // Wait a bit for watcher to initialize
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Create a Rust file
        fs::write(src_dir.join("lib.rs"), "pub fn test() {}").unwrap();

        // Wait for event (with timeout)
        let result = timeout(Duration::from_secs(2), rx.recv()).await;
        assert!(result.is_ok(), "Should detect file creation");
    }
}

