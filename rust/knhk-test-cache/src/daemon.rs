//! Daemon management for test cache
//!
//! Manages daemon lifecycle: start, stop, status, rebuild.
//! Uses PID file for process management.

use crate::{Cache, CodeHasher, FileWatcher, TestCacheError, TestCompiler};
use std::path::PathBuf;
use std::process;
use std::sync::Arc;
use tokio::process::Command;
use tokio::signal;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

/// Daemon command
#[derive(Debug, Clone)]
pub enum DaemonCommand {
    Start,
    Stop,
    Status,
    Rebuild,
}

/// Test cache daemon
pub struct Daemon {
    /// Project root directory
    project_root: PathBuf,
    /// Cache directory
    cache_dir: PathBuf,
    /// PID file path
    pid_file: PathBuf,
    /// Log file path
    log_file: PathBuf,
    /// Code hasher
    hasher: CodeHasher,
    /// Cache manager
    cache: Cache,
    /// Test compiler
    compiler: TestCompiler,
}

impl Daemon {
    /// Create a new daemon instance
    pub fn new(project_root: PathBuf) -> Self {
        let cache_dir = project_root.join(".test-cache");
        std::fs::create_dir_all(&cache_dir).ok();

        let rust_dir = project_root.join("rust");
        let hasher = CodeHasher::new(rust_dir.clone());
        let cache = Cache::new(cache_dir.clone());
        let compiler = TestCompiler::new(rust_dir);

        Self {
            project_root,
            cache_dir: cache_dir.clone(),
            pid_file: cache_dir.join("daemon.pid"),
            log_file: cache_dir.join("daemon.log"),
            hasher,
            cache,
            compiler,
        }
    }

    /// Check if daemon is running
    pub fn is_running(&self) -> bool {
        if !self.pid_file.exists() {
            return false;
        }

        if let Ok(pid_str) = std::fs::read_to_string(&self.pid_file) {
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                // Check if process exists
                return process::id() == pid || self.process_exists(pid);
            }
        }

        false
    }

    /// Check if process exists
    fn process_exists(&self, pid: u32) -> bool {
        #[cfg(unix)]
        {
            use std::process::Command;
            Command::new("kill")
                .arg("-0")
                .arg(pid.to_string())
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        }

        #[cfg(windows)]
        {
            use std::process::Command;
            Command::new("tasklist")
                .arg("/FI")
                .arg(format!("PID eq {}", pid))
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        }
    }

    /// Start daemon
    pub async fn start(&self) -> Result<(), TestCacheError> {
        if self.is_running() {
            return Err(TestCacheError::DaemonRunning(
                std::fs::read_to_string(&self.pid_file)
                    .ok()
                    .and_then(|s| s.trim().parse().ok())
                    .unwrap_or(0),
            ));
        }

        info!("Starting test cache daemon...");

        // Initial code hash
        let current_hash = self.hasher.hash()?;
        info!("Current code hash: {}", current_hash);

        // Pre-compile test binaries if needed
        if !self.compiler.binaries_exist() {
            info!("Pre-compiling test binaries...");
            self.compiler.precompile().await?;
            info!("Test binaries pre-compiled");
        } else {
            info!("Test binaries already exist");
        }

        // Start file watcher
        let rust_dir = self.project_root.join("rust");
        let mut watcher = FileWatcher::new(rust_dir)?;

        let (event_tx, mut event_rx) = mpsc::unbounded_channel();

        // Spawn file watcher task
        let watcher_task = {
            let mut watcher = watcher;
            tokio::spawn(async move {
                if let Err(e) = watcher.watch_async(event_tx) {
                    error!("File watcher error: {}", e);
                }
            })
        };

        // Spawn rebuild task
        let rebuild_task = {
            let compiler = Arc::new(self.compiler.clone());
            let cache = Arc::new(self.cache.clone());
            let hasher = Arc::new(self.hasher.clone());

            tokio::spawn(async move {
                while let Some(_changed_file) = event_rx.recv().await {
                    // Debounce: wait a bit for more changes
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                    // Check if code actually changed
                    let new_hash = match hasher.hash() {
                        Ok(h) => h,
                        Err(e) => {
                            warn!("Failed to hash code: {}", e);
                            continue;
                        }
                    };

                    // Check cache
                    match cache.get(&new_hash) {
                        Ok(Some(_)) => {
                            info!("Code hash unchanged, skipping rebuild");
                            continue;
                        }
                        Ok(None) => {
                            // Cache miss, proceed with rebuild
                        }
                        Err(e) => {
                            warn!("Failed to check cache: {}", e);
                            // Proceed with rebuild on cache error
                        }
                    }

                    info!("Code changed, pre-compiling test binaries...");
                    if let Err(e) = compiler.precompile().await {
                        error!("Failed to pre-compile test binaries: {}", e);
                    } else {
                        info!("Test binaries pre-compiled successfully");
                    }
                }
            })
        };

        // Write PID file
        let pid = process::id();
        std::fs::write(&self.pid_file, pid.to_string())?;

        info!("Daemon started (PID: {})", pid);
        info!("Watching for code changes...");

        // Wait for shutdown signal
        #[cfg(unix)]
        let mut shutdown = signal::unix::signal(signal::unix::SignalKind::interrupt())?;

        #[cfg(unix)]
        shutdown.recv().await;

        #[cfg(windows)]
        {
            let _ = signal::ctrl_c().await;
        }

        info!("Shutting down daemon...");

        // Cleanup
        watcher_task.abort();
        rebuild_task.abort();
        std::fs::remove_file(&self.pid_file).ok();

        Ok(())
    }

    /// Stop daemon
    pub async fn stop(&self) -> Result<(), TestCacheError> {
        if !self.is_running() {
            return Err(TestCacheError::DaemonNotRunning);
        }

        let pid_str = std::fs::read_to_string(&self.pid_file)?;
        let pid: u32 = pid_str
            .trim()
            .parse()
            .map_err(|_| TestCacheError::InvalidConfig("Invalid PID".to_string()))?;

        info!("Stopping daemon (PID: {})...", pid);

        #[cfg(unix)]
        {
            Command::new("kill")
                .arg(pid.to_string())
                .output()
                .await
                .ok();
        }

        #[cfg(windows)]
        {
            Command::new("taskkill")
                .arg("/PID")
                .arg(pid.to_string())
                .output()
                .await
                .ok();
        }

        // Wait a bit for process to exit
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        std::fs::remove_file(&self.pid_file).ok();
        info!("Daemon stopped");

        Ok(())
    }

    /// Get daemon status
    pub fn status(&self) -> Result<DaemonStatus, TestCacheError> {
        let running = self.is_running();
        let stats = self.cache.stats()?;
        let current_hash = self.hasher.hash().ok();

        Ok(DaemonStatus {
            running,
            cache_stats: stats,
            current_code_hash: current_hash,
            pid_file: self.pid_file.clone(),
            log_file: self.log_file.clone(),
        })
    }

    /// Force rebuild of test binaries
    pub async fn rebuild(&self) -> Result<(), TestCacheError> {
        info!("Forcing rebuild of test binaries...");
        self.compiler.precompile().await?;
        info!("Rebuild complete");
        Ok(())
    }
}

/// Daemon status information
#[derive(Debug)]
pub struct DaemonStatus {
    pub running: bool,
    pub cache_stats: crate::cache::CacheStats,
    pub current_code_hash: Option<String>,
    pub pid_file: PathBuf,
    pub log_file: PathBuf,
}
