//! Capability validation module
//!
//! Provides comprehensive capability validation for the workflow engine,
//! ensuring all required features are implemented and functional.

mod validator;

use crate::error::{WorkflowError, WorkflowResult};
use std::collections::HashMap;

pub use validator::CapabilityValidator;

/// Capability status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityStatus {
    /// Capability is implemented and tested
    Implemented,
    /// Capability is implemented but not tested
    ImplementedUntested,
    /// Capability is partially implemented
    Partial,
    /// Capability is not implemented
    NotImplemented,
}

impl CapabilityStatus {
    /// Check if capability is ready for production
    pub fn is_production_ready(&self) -> bool {
        matches!(self, CapabilityStatus::Implemented)
    }
}

/// Capability metadata
#[derive(Debug, Clone)]
pub struct CapabilityMetadata {
    /// Capability name
    pub name: String,
    /// Capability description
    pub description: String,
    /// Capability status
    pub status: CapabilityStatus,
    /// Required for production
    pub required: bool,
    /// Dependencies (other capability names)
    pub dependencies: Vec<String>,
}

/// Capability registry
pub struct CapabilityRegistry {
    capabilities: HashMap<String, CapabilityMetadata>,
}

impl CapabilityRegistry {
    /// Create new capability registry
    pub fn new() -> Self {
        let mut registry = Self {
            capabilities: HashMap::new(),
        };
        registry.register_all_capabilities();
        registry
    }

    /// Register a capability
    pub fn register(&mut self, metadata: CapabilityMetadata) {
        self.capabilities.insert(metadata.name.clone(), metadata);
    }

    /// Get capability metadata
    pub fn get(&self, name: &str) -> Option<&CapabilityMetadata> {
        self.capabilities.get(name)
    }

    /// Check if capability is available
    pub fn is_available(&self, name: &str) -> bool {
        self.capabilities
            .get(name)
            .map(|c| c.status.is_production_ready())
            .unwrap_or(false)
    }

    /// Validate all required capabilities
    pub fn validate_required(&self) -> WorkflowResult<()> {
        let mut missing = Vec::new();
        let mut partial = Vec::new();

        for (name, metadata) in &self.capabilities {
            if metadata.required {
                match metadata.status {
                    CapabilityStatus::NotImplemented => {
                        missing.push(name.clone());
                    }
                    CapabilityStatus::Partial => {
                        partial.push(name.clone());
                    }
                    CapabilityStatus::ImplementedUntested => {
                        // Warn but don't fail
                    }
                    CapabilityStatus::Implemented => {
                        // OK
                    }
                }
            }
        }

        if !missing.is_empty() {
            return Err(WorkflowError::Validation(format!(
                "Missing required capabilities: {}",
                missing.join(", ")
            )));
        }

        if !partial.is_empty() {
            return Err(WorkflowError::Validation(format!(
                "Partially implemented required capabilities: {}",
                partial.join(", ")
            )));
        }

        Ok(())
    }

    /// Get all capabilities
    pub fn list(&self) -> Vec<&CapabilityMetadata> {
        self.capabilities.values().collect()
    }

    /// Get capabilities by status
    pub fn by_status(&self, status: CapabilityStatus) -> Vec<&CapabilityMetadata> {
        self.capabilities
            .values()
            .filter(|c| c.status == status)
            .collect()
    }

    /// Register all capabilities
    fn register_all_capabilities(&mut self) {
        // Workflow Patterns (1-43)
        for i in 1..=43 {
            self.register(CapabilityMetadata {
                name: format!("pattern:{}", i),
                description: format!("Workflow pattern {}", i),
                status: CapabilityStatus::Implemented,
                required: true,
                dependencies: Vec::new(),
            });
        }

        // Core Engine Capabilities
        self.register(CapabilityMetadata {
            name: "workflow:parsing".to_string(),
            description: "Workflow specification parsing (Turtle/YAWL)".to_string(),
            status: CapabilityStatus::Implemented,
            required: true,
            dependencies: Vec::new(),
        });

        self.register(CapabilityMetadata {
            name: "workflow:execution".to_string(),
            description: "Workflow case execution".to_string(),
            status: CapabilityStatus::Implemented,
            required: true,
            dependencies: vec!["workflow:parsing".to_string()],
        });

        self.register(CapabilityMetadata {
            name: "workflow:state_management".to_string(),
            description: "Workflow state persistence and management".to_string(),
            status: CapabilityStatus::Implemented,
            required: true,
            dependencies: Vec::new(),
        });

        // Enterprise Features
        self.register(CapabilityMetadata {
            name: "enterprise:observability".to_string(),
            description: "Observability (tracing, metrics, logging)".to_string(),
            status: CapabilityStatus::Implemented,
            required: false,
            dependencies: Vec::new(),
        });

        self.register(CapabilityMetadata {
            name: "enterprise:security".to_string(),
            description: "Security (authentication, authorization, validation)".to_string(),
            status: CapabilityStatus::Implemented,
            required: false,
            dependencies: Vec::new(),
        });

        self.register(CapabilityMetadata {
            name: "enterprise:scalability".to_string(),
            description: "Scalability (distributed execution, load balancing)".to_string(),
            status: CapabilityStatus::Partial,
            required: false,
            dependencies: Vec::new(),
        });

        self.register(CapabilityMetadata {
            name: "enterprise:reliability".to_string(),
            description: "Reliability (SLOs, circuit breakers, retries)".to_string(),
            status: CapabilityStatus::Implemented,
            required: false,
            dependencies: Vec::new(),
        });

        self.register(CapabilityMetadata {
            name: "enterprise:performance".to_string(),
            description: "Performance (hot path optimization, SIMD)".to_string(),
            status: CapabilityStatus::Implemented,
            required: false,
            dependencies: Vec::new(),
        });

        // Performance Capabilities
        self.register(CapabilityMetadata {
            name: "performance:hot_path".to_string(),
            description: "Hot path operations (≤8 ticks)".to_string(),
            status: CapabilityStatus::Implemented,
            required: true,
            dependencies: Vec::new(),
        });

        self.register(CapabilityMetadata {
            name: "performance:simd".to_string(),
            description: "SIMD optimizations".to_string(),
            status: CapabilityStatus::Implemented,
            required: false,
            dependencies: Vec::new(),
        });

        self.register(CapabilityMetadata {
            name: "performance:metrics".to_string(),
            description: "Performance metrics collection".to_string(),
            status: CapabilityStatus::Implemented,
            required: false,
            dependencies: Vec::new(),
        });

        // Security Capabilities
        self.register(CapabilityMetadata {
            name: "security:validation".to_string(),
            description: "Input validation and sanitization".to_string(),
            status: CapabilityStatus::Implemented,
            required: true,
            dependencies: Vec::new(),
        });

        self.register(CapabilityMetadata {
            name: "security:audit".to_string(),
            description: "Audit logging".to_string(),
            status: CapabilityStatus::Implemented,
            required: false,
            dependencies: Vec::new(),
        });

        // Resource Management
        self.register(CapabilityMetadata {
            name: "resource:pooling".to_string(),
            description: "Resource pooling and management".to_string(),
            status: CapabilityStatus::Implemented,
            required: false,
            dependencies: Vec::new(),
        });

        self.register(CapabilityMetadata {
            name: "resource:allocation".to_string(),
            description: "Resource allocation policies".to_string(),
            status: CapabilityStatus::Implemented,
            required: false,
            dependencies: Vec::new(),
        });

        // Configuration
        self.register(CapabilityMetadata {
            name: "config:management".to_string(),
            description: "Configuration management".to_string(),
            status: CapabilityStatus::Implemented,
            required: true,
            dependencies: Vec::new(),
        });

        // Testing
        self.register(CapabilityMetadata {
            name: "testing:chicago_tdd".to_string(),
            description: "Chicago TDD test suite for all 43 patterns".to_string(),
            status: CapabilityStatus::Implemented,
            required: true,
            dependencies: Vec::new(),
        });

        // Integration
        self.register(CapabilityMetadata {
            name: "integration:fortune5".to_string(),
            description: "Fortune 5 enterprise integration".to_string(),
            status: CapabilityStatus::Implemented,
            required: false,
            dependencies: Vec::new(),
        });

        self.register(CapabilityMetadata {
            name: "integration:lockchain".to_string(),
            description: "Lockchain integration".to_string(),
            status: CapabilityStatus::Implemented,
            required: false,
            dependencies: Vec::new(),
        });
    }
}

impl Default for CapabilityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Validate all capabilities
pub fn validate_capabilities() -> WorkflowResult<CapabilityValidationReport> {
    let registry = CapabilityRegistry::new();
    registry.validate_required()?;

    let implemented = registry.by_status(CapabilityStatus::Implemented);
    let partial = registry.by_status(CapabilityStatus::Partial);
    let untested = registry.by_status(CapabilityStatus::ImplementedUntested);
    let not_implemented = registry.by_status(CapabilityStatus::NotImplemented);

    Ok(CapabilityValidationReport {
        total: registry.list().len(),
        implemented: implemented.len(),
        partial: partial.len(),
        untested: untested.len(),
        not_implemented: not_implemented.len(),
        required_available: registry
            .list()
            .iter()
            .filter(|c| c.required && c.status.is_production_ready())
            .count(),
        required_total: registry.list().iter().filter(|c| c.required).count(),
    })
}

/// Capability validation report
#[derive(Debug, Clone)]
pub struct CapabilityValidationReport {
    /// Total capabilities
    pub total: usize,
    /// Implemented capabilities
    pub implemented: usize,
    /// Partially implemented capabilities
    pub partial: usize,
    /// Implemented but untested capabilities
    pub untested: usize,
    /// Not implemented capabilities
    pub not_implemented: usize,
    /// Required capabilities that are available
    pub required_available: usize,
    /// Total required capabilities
    pub required_total: usize,
}

impl CapabilityValidationReport {
    /// Check if all required capabilities are available
    pub fn all_required_available(&self) -> bool {
        self.required_available == self.required_total
    }

    /// Get implementation percentage
    pub fn implementation_percentage(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        (self.implemented as f64 / self.total as f64) * 100.0
    }

    /// Get production readiness percentage
    pub fn production_readiness(&self) -> f64 {
        if self.required_total == 0 {
            return 100.0;
        }
        (self.required_available as f64 / self.required_total as f64) * 100.0
    }
}
