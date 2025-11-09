//! Configuration management
//!
//! Provides configuration loading, validation, and environment-based overrides.

use crate::error::{WorkflowError, WorkflowResult};
use serde::{Deserialize, Serialize};
// Configuration module - HashMap will be used when implementing config loading
use std::path::Path;

/// Configuration loader
pub struct ConfigLoader;

impl ConfigLoader {
    /// Load configuration from file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> WorkflowResult<AppConfig> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| WorkflowError::Internal(format!("Failed to read config: {}", e)))?;
        Self::load_from_str(&content)
    }

    /// Load configuration from string
    pub fn load_from_str(_content: &str) -> WorkflowResult<AppConfig> {
        // TOML parsing is not yet implemented
        // Return error instead of false positive (claiming to load config when we return default)
        Err(WorkflowError::Internal(
            "Configuration loading from string requires TOML parsing - TOML parsing not yet implemented. Use load_from_env() or AppConfig::default() instead.".to_string()
        ))
    }

    /// Load configuration from environment variables
    pub fn load_from_env() -> WorkflowResult<AppConfig> {
        let mut config = AppConfig::default();

        // Override with environment variables
        if let Ok(val) = std::env::var("KNHK_LOG_LEVEL") {
            config.logging.level = val;
        }
        if let Ok(val) = std::env::var("KNHK_SERVICE_NAME") {
            config.service.name = val;
        }
        if let Ok(val) = std::env::var("KNHK_SERVICE_VERSION") {
            config.service.version = val;
        }

        config.validate()?;
        Ok(config)
    }
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    /// Service configuration
    pub service: ServiceConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
    /// Performance configuration
    pub performance: PerformanceConfig,
    /// Security configuration
    pub security: SecurityConfig,
}

impl AppConfig {
    /// Validate configuration
    pub fn validate(&self) -> WorkflowResult<()> {
        self.service.validate()?;
        self.logging.validate()?;
        self.performance.validate()?;
        self.security.validate()?;
        Ok(())
    }
}

/// Service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// Service name
    pub name: String,
    /// Service version
    pub version: String,
    /// Service environment
    pub environment: String,
}

impl ServiceConfig {
    /// Validate service configuration
    pub fn validate(&self) -> WorkflowResult<()> {
        if self.name.is_empty() {
            return Err(WorkflowError::Validation(
                "Service name cannot be empty".to_string(),
            ));
        }
        Ok(())
    }
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            name: "knhk-workflow-engine".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            environment: "development".to_string(),
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level
    pub level: String,
    /// Enable structured logging
    pub structured: bool,
    /// Enable JSON output
    pub json: bool,
}

impl LoggingConfig {
    /// Validate logging configuration
    pub fn validate(&self) -> WorkflowResult<()> {
        let valid_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_levels.contains(&self.level.as_str()) {
            return Err(WorkflowError::Validation(format!(
                "Invalid log level: {}",
                self.level
            )));
        }
        Ok(())
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            structured: true,
            json: false,
        }
    }
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Hot path tick budget
    pub hot_path_ticks: u32,
    /// Enable SIMD
    pub enable_simd: bool,
    /// Enable caching
    pub enable_caching: bool,
}

impl PerformanceConfig {
    /// Validate performance configuration
    pub fn validate(&self) -> WorkflowResult<()> {
        if self.hot_path_ticks > 8 {
            return Err(WorkflowError::Validation(format!(
                "Hot path ticks {} exceeds maximum 8",
                self.hot_path_ticks
            )));
        }
        Ok(())
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            hot_path_ticks: 8,
            enable_simd: true,
            enable_caching: true,
        }
    }
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable input validation
    pub enable_validation: bool,
    /// Enable sanitization
    pub enable_sanitization: bool,
    /// Allowed origins
    pub allowed_origins: Vec<String>,
}

impl SecurityConfig {
    /// Validate security configuration
    pub fn validate(&self) -> WorkflowResult<()> {
        // Validate allowed origins format (must be valid URIs or "*")
        for origin in &self.allowed_origins {
            if origin != "*" && !origin.starts_with("http://") && !origin.starts_with("https://") {
                return Err(WorkflowError::Validation(format!(
                    "Invalid origin format: {} (must be '*' or start with 'http://' or 'https://')",
                    origin
                )));
            }
        }

        // Validate that if validation is disabled, sanitization should also be disabled
        // (security best practice: don't disable validation without sanitization)
        if !self.enable_validation && !self.enable_sanitization {
            return Err(WorkflowError::Validation(
                "Security configuration invalid: validation and sanitization cannot both be disabled".to_string()
            ));
        }

        Ok(())
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_validation: true,
            enable_sanitization: true,
            allowed_origins: Vec::new(),
        }
    }
}
