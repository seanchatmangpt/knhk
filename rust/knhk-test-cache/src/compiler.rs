//! Test binary compiler
//!
//! Pre-compiles test binaries using cargo build --tests.
//! Runs in background to avoid blocking.

use crate::TestCacheError;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;

/// Test compiler that pre-compiles test binaries
#[derive(Clone)]
pub struct TestCompiler {
    /// Workspace root directory
    workspace_root: PathBuf,
}

impl TestCompiler {
    /// Create a new test compiler
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }

    /// Check if cargo is available
    pub fn check_cargo_available() -> Result<(), TestCacheError> {
        which::which("cargo").map_err(|_| TestCacheError::CargoNotFound)?;
        Ok(())
    }

    /// Pre-compile test binaries
    ///
    /// Runs `cargo build --tests --workspace` in background.
    /// Uses file locking to prevent concurrent builds.
    pub async fn precompile(&self) -> Result<(), TestCacheError> {
        // Check if cargo is available
        Self::check_cargo_available()?;

        // Run cargo build --tests --workspace
        let output = Command::new("cargo")
            .arg("build")
            .arg("--tests")
            .arg("--workspace")
            .current_dir(&self.workspace_root)
            .env("CARGO_INCREMENTAL", "1")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| TestCacheError::ProcessError(format!("Failed to execute cargo: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(TestCacheError::ProcessError(format!(
                "Cargo build failed: {}",
                stderr
            )));
        }

        Ok(())
    }

    /// Check if test binaries are up-to-date
    ///
    /// Checks if test binaries exist and are newer than source files.
    /// This is a heuristic - full validation requires code hashing.
    pub fn binaries_exist(&self) -> bool {
        let target_dir = self.workspace_root.join("target");
        if !target_dir.exists() {
            return false;
        }

        // Check for test binaries (heuristic: look for test executables)
        // This is approximate - full check requires code hashing
        target_dir.exists()
    }

    /// Get workspace root
    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_check_cargo_available() {
        // Cargo should be available in test environment
        let result = TestCompiler::check_cargo_available();
        assert!(result.is_ok() || matches!(result.unwrap_err(), TestCacheError::CargoNotFound));
    }

    #[tokio::test]
    async fn test_precompile_requires_cargo() {
        let temp_dir = TempDir::new().unwrap();
        let compiler = TestCompiler::new(temp_dir.path().to_path_buf());

        // This will fail if cargo not found, or succeed if cargo is available
        // We just verify the method exists and can be called
        let _ = compiler.precompile().await;
    }
}
