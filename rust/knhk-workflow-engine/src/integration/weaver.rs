//! Weaver Live-Check Integration for Workflow Engine
//!
//! Provides Weaver live-check validation for all workflow engine telemetry.
//! Ensures all OTEL spans and metrics conform to semantic conventions.

use crate::error::WorkflowResult;
#[cfg(feature = "std")]
use knhk_otel::validation::{validate_weaver_live_check, ValidationError};
use knhk_otel::WeaverLiveCheck;
use std::path::PathBuf;
use std::process::Child;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// Weaver integration for workflow engine
pub struct WeaverIntegration {
    live_check: Arc<RwLock<Option<WeaverLiveCheck>>>,
    process: Arc<RwLock<Option<Child>>>,
    registry_path: PathBuf,
    otlp_grpc_port: u16,
    admin_port: u16,
    enabled: bool,
}

impl WeaverIntegration {
    /// Create new Weaver integration
    pub fn new(registry_path: PathBuf) -> Self {
        Self {
            live_check: Arc::new(RwLock::new(None)),
            process: Arc::new(RwLock::new(None)),
            registry_path,
            otlp_grpc_port: 4317,
            admin_port: 8080,
            enabled: false,
        }
    }

    /// Create Weaver integration with custom configuration
    pub fn with_config(registry_path: PathBuf, otlp_grpc_port: u16, admin_port: u16) -> Self {
        Self {
            live_check: Arc::new(RwLock::new(None)),
            process: Arc::new(RwLock::new(None)),
            registry_path,
            otlp_grpc_port,
            admin_port,
            enabled: false,
        }
    }

    /// Enable Weaver integration
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable Weaver integration
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Check if Weaver binary is available
    pub fn check_weaver_available() -> WorkflowResult<()> {
        WeaverLiveCheck::check_weaver_available().map_err(|e| {
            crate::error::WorkflowError::Internal(format!("Weaver check failed: {}", e))
        })
    }

    /// Start Weaver live-check
    pub async fn start(&self) -> WorkflowResult<()> {
        if !self.enabled {
            return Ok(());
        }

        // Check Weaver binary availability
        Self::check_weaver_available()?;

        // Verify registry path exists
        if !self.registry_path.exists() {
            return Err(crate::error::WorkflowError::Internal(format!(
                "Registry path does not exist: {}",
                self.registry_path.display()
            )));
        }

        // Create Weaver live-check instance
        let live_check = WeaverLiveCheck::new()
            .with_registry(
                self.registry_path
                    .to_str()
                    .ok_or_else(|| {
                        crate::error::WorkflowError::Internal(
                            "Registry path is not valid UTF-8".to_string(),
                        )
                    })?
                    .to_string(),
            )
            .with_otlp_port(self.otlp_grpc_port)
            .with_admin_port(self.admin_port)
            .with_inactivity_timeout(300) // 5 minutes
            .with_format("json".to_string())
            .with_output("./weaver-reports".to_string());

        // Start Weaver live-check process
        match live_check.start() {
            Ok(process) => {
                let mut guard = self.process.write().await;
                *guard = Some(process);
                let mut live_check_guard = self.live_check.write().await;
                *live_check_guard = Some(live_check);

                info!(
                    registry_path = %self.registry_path.display(),
                    otlp_port = self.otlp_grpc_port,
                    admin_port = self.admin_port,
                    "Weaver live-check started"
                );

                // Wait a bit for Weaver to start
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                // Verify Weaver is running
                if let Some(ref lc) = *live_check_guard {
                    match lc.check_health() {
                        Ok(true) => {
                            info!("Weaver live-check is healthy and ready");

                            // Use validation helper to verify Weaver setup
                            #[cfg(feature = "std")]
                            {
                                if let Err(e) = validate_weaver_live_check(&self.registry_path) {
                                    warn!("Weaver validation check failed: {}", e);
                                }
                            }
                        }
                        Ok(false) => {
                            warn!("Weaver live-check started but health check failed");
                        }
                        Err(e) => {
                            warn!("Weaver health check error: {}", e);
                        }
                    }
                }

                Ok(())
            }
            Err(e) => {
                error!("Failed to start Weaver live-check: {}", e);
                Err(crate::error::WorkflowError::Internal(format!(
                    "Failed to start Weaver: {}",
                    e
                )))
            }
        }
    }

    /// Stop Weaver live-check
    pub async fn stop(&self) -> WorkflowResult<()> {
        let mut process_guard = self.process.write().await;
        let mut live_check_guard = self.live_check.write().await;

        if let Some(ref mut process) = *process_guard {
            // Try graceful shutdown via admin endpoint first
            if let Some(ref lc) = *live_check_guard {
                if let Err(e) = lc.stop() {
                    warn!("Failed to stop Weaver via admin endpoint: {}", e);
                    // Fall back to killing the process
                    if let Err(e) = process.kill() {
                        error!("Failed to kill Weaver process: {}", e);
                    }
                } else {
                    info!("Weaver live-check stopped gracefully");
                }
            } else {
                // No live-check instance, just kill the process
                if let Err(e) = process.kill() {
                    error!("Failed to kill Weaver process: {}", e);
                }
            }

            // Wait for process to exit
            let _ = process.wait();
        }

        *process_guard = None;
        *live_check_guard = None;

        Ok(())
    }

    /// Get OTLP gRPC endpoint for telemetry export
    pub fn otlp_endpoint(&self) -> String {
        format!("http://127.0.0.1:{}", self.otlp_grpc_port)
    }

    /// Check if Weaver is running
    pub async fn is_running(&self) -> bool {
        let mut guard = self.process.write().await;
        guard.as_mut().map_or(false, |p| {
            p.try_wait().map_or(true, |status| status.is_none())
        })
    }

    /// Get Weaver health status
    pub async fn health_check(&self) -> WorkflowResult<bool> {
        let guard = self.live_check.read().await;
        if let Some(ref lc) = *guard {
            lc.check_health().map_err(|e| {
                crate::error::WorkflowError::Internal(format!("Health check failed: {}", e))
            })
        } else {
            Ok(false)
        }
    }

    /// Get validation results from Weaver reports directory
    /// Returns the path to the reports directory if available
    pub fn reports_directory(&self) -> PathBuf {
        PathBuf::from("./weaver-reports")
    }

    /// Check if validation reports exist
    pub async fn has_reports(&self) -> bool {
        let reports_dir = self.reports_directory();
        reports_dir.exists() && reports_dir.is_dir()
    }
}

impl Default for WeaverIntegration {
    fn default() -> Self {
        Self::new(PathBuf::from("./registry"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_weaver_integration_creation() {
        let temp_dir = TempDir::new().unwrap();
        let registry_path = temp_dir.path().join("registry");
        std::fs::create_dir_all(&registry_path).unwrap();

        let weaver = WeaverIntegration::new(registry_path.clone());
        assert_eq!(weaver.registry_path, registry_path);
        assert_eq!(weaver.otlp_grpc_port, 4317);
        assert_eq!(weaver.admin_port, 8080);
        assert!(!weaver.enabled);
    }

    #[tokio::test]
    async fn test_weaver_integration_with_config() {
        let temp_dir = TempDir::new().unwrap();
        let registry_path = temp_dir.path().join("registry");
        std::fs::create_dir_all(&registry_path).unwrap();

        let weaver = WeaverIntegration::with_config(registry_path, 4318, 8081);
        assert_eq!(weaver.otlp_grpc_port, 4318);
        assert_eq!(weaver.admin_port, 8081);
    }

    #[tokio::test]
    async fn test_weaver_enable_disable() {
        let temp_dir = TempDir::new().unwrap();
        let registry_path = temp_dir.path().join("registry");
        std::fs::create_dir_all(&registry_path).unwrap();

        let mut weaver = WeaverIntegration::new(registry_path);
        assert!(!weaver.enabled);

        weaver.enable();
        assert!(weaver.enabled);

        weaver.disable();
        assert!(!weaver.enabled);
    }

    #[tokio::test]
    async fn test_otlp_endpoint() {
        let temp_dir = TempDir::new().unwrap();
        let registry_path = temp_dir.path().join("registry");
        std::fs::create_dir_all(&registry_path).unwrap();

        let weaver = WeaverIntegration::with_config(registry_path, 4318, 8081);
        assert_eq!(weaver.otlp_endpoint(), "http://127.0.0.1:4318");
    }
}
