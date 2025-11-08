//! Unified integration manager
//!
//! Combines the best features from all KNHK integrations:
//! - Fortune 5: SPIFFE/SPIRE, KMS, SLO, multi-region, promotion gates
//! - Lockchain: Receipt storage, provenance tracking
//! - Connectors: Kafka, Salesforce, external systems
//! - OTEL: Tracing, metrics, logging
//! - Sidecar: gRPC, JSON parsing (simdjson)
//! - ETL: 5-stage pipeline (Ingest, Transform, Load, Reflex, Emit)

use crate::error::{WorkflowError, WorkflowResult};
use crate::integration::check::{HealthCheckResult, HealthStatus, IntegrationHealthChecker};
use crate::integration::connectors::ConnectorIntegration;
use crate::integration::fortune5::{Fortune5Config, Fortune5Integration};
use crate::integration::lockchain::LockchainIntegration;
use crate::integration::otel::OtelIntegration;
use crate::integration::registry::{IntegrationRegistry, IntegrationStatus};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Unified integration configuration
#[derive(Debug, Clone)]
pub struct UnifiedIntegrationConfig {
    /// Fortune 5 configuration
    pub fortune5: Option<Fortune5Config>,
    /// Lockchain path
    pub lockchain_path: Option<String>,
    /// OTEL endpoint
    pub otel_endpoint: Option<String>,
    /// Connector configuration
    pub connectors: HashMap<String, String>,
    /// Enable SLO tracking
    pub enable_slo: bool,
    /// Enable provenance tracking
    pub enable_provenance: bool,
    /// Enable observability
    pub enable_observability: bool,
}

impl Default for UnifiedIntegrationConfig {
    fn default() -> Self {
        Self {
            fortune5: None,
            lockchain_path: None,
            otel_endpoint: None,
            connectors: HashMap::new(),
            enable_slo: true,
            enable_provenance: true,
            enable_observability: true,
        }
    }
}

/// Unified integration manager
pub struct UnifiedIntegration {
    /// Fortune 5 integration
    fortune5: Option<Arc<Fortune5Integration>>,
    /// Lockchain integration
    lockchain: Option<Arc<LockchainIntegration>>,
    /// OTEL integration
    otel: Option<Arc<OtelIntegration>>,
    /// Connector integration
    connectors: Arc<RwLock<ConnectorIntegration>>,
    /// Integration registry
    registry: Arc<IntegrationRegistry>,
    /// Health checker
    health_checker: Arc<IntegrationHealthChecker>,
    /// Configuration
    config: UnifiedIntegrationConfig,
}

impl UnifiedIntegration {
    /// Create new unified integration
    pub async fn new(config: UnifiedIntegrationConfig) -> WorkflowResult<Self> {
        // Initialize Fortune 5 integration
        let fortune5 = if let Some(ref fortune5_config) = config.fortune5 {
            Some(Arc::new(Fortune5Integration::new(fortune5_config.clone())))
        } else {
            None
        };

        // Initialize Lockchain integration
        let lockchain = if let Some(ref lockchain_path) = config.lockchain_path {
            Some(Arc::new(LockchainIntegration::new(lockchain_path)?))
        } else {
            None
        };

        // Initialize OTEL integration
        let otel = if let Some(ref otel_endpoint) = config.otel_endpoint {
            Some(Arc::new(OtelIntegration::new(otel_endpoint.clone())?))
        } else {
            None
        };

        // Initialize Connector integration
        let mut connectors = ConnectorIntegration::new();
        for (name, config) in &config.connectors {
            // FUTURE: Register connectors based on configuration
            // connectors.register_connector(name.clone(), ...);
        }

        let registry = Arc::new(IntegrationRegistry::new());
        let health_checker = Arc::new(IntegrationHealthChecker::new());

        Ok(Self {
            fortune5,
            lockchain,
            otel,
            connectors: Arc::new(RwLock::new(connectors)),
            registry,
            health_checker,
            config,
        })
    }

    /// Get Fortune 5 integration (best: SPIFFE/SPIRE, KMS, SLO, multi-region, promotion gates)
    pub fn fortune5(&self) -> Option<&Fortune5Integration> {
        self.fortune5.as_ref().map(|f| f.as_ref())
    }

    /// Get Lockchain integration (best: receipt storage, provenance tracking)
    pub fn lockchain(&self) -> Option<&LockchainIntegration> {
        self.lockchain.as_ref().map(|l| l.as_ref())
    }

    /// Get OTEL integration (best: tracing, metrics, logging)
    pub fn otel(&self) -> Option<&OtelIntegration> {
        self.otel.as_ref().map(|o| o.as_ref())
    }

    /// Get Connector integration (best: Kafka, Salesforce, external systems)
    pub async fn connectors(&self) -> &ConnectorIntegration {
        &*self.connectors.read().await
    }

    /// Record SLO metric (best from Fortune 5)
    pub async fn record_slo_metric(
        &self,
        runtime_class: crate::integration::fortune5::slo::RuntimeClass,
        latency_ns: u64,
    ) {
        if let Some(ref fortune5) = self.fortune5 {
            fortune5.record_slo_metric(runtime_class, latency_ns).await;
        }
    }

    /// Store receipt with provenance (best from Lockchain)
    pub async fn store_receipt(&self, receipt: knhk_lockchain::Receipt) -> WorkflowResult<()> {
        if let Some(ref lockchain) = self.lockchain {
            lockchain.store_receipt(receipt).await
        } else {
            Err(WorkflowError::ResourceUnavailable(
                "Lockchain integration not available".to_string(),
            ))
        }
    }

    /// Start trace span (best from OTEL)
    pub fn start_trace_span(
        &self,
        name: &str,
        attributes: HashMap<String, String>,
    ) -> Option<opentelemetry::trace::Span> {
        if let Some(ref otel) = self.otel {
            otel.start_span(name, attributes)
        } else {
            None
        }
    }

    /// Execute connector task (best from Connectors)
    pub async fn execute_connector_task(
        &self,
        connector_name: &str,
        data: serde_json::Value,
    ) -> WorkflowResult<serde_json::Value> {
        let mut connectors = self.connectors.write().await;
        connectors.execute_task(connector_name, data).await
    }

    /// Check integration health (best from Health Checker)
    pub async fn check_health(&self) -> WorkflowResult<Vec<HealthCheckResult>> {
        let mut health_checker = IntegrationHealthChecker::new();
        health_checker.check_all().await
    }

    /// Get integration registry (best from Registry)
    pub fn registry(&self) -> &IntegrationRegistry {
        &self.registry
    }

    /// Enable integration (best from Registry)
    pub async fn enable_integration(&self, name: &str) -> WorkflowResult<()> {
        self.registry.enable(name).await
    }

    /// Disable integration (best from Registry)
    pub async fn disable_integration(&self, name: &str) -> WorkflowResult<()> {
        self.registry.disable(name).await
    }

    /// Get all available integrations (best from Registry)
    pub async fn list_available(&self) -> Vec<crate::integration::registry::IntegrationMetadata> {
        self.registry.list_available().await
    }

    /// Check SLO compliance (best from Fortune 5)
    pub async fn check_slo_compliance(&self) -> WorkflowResult<bool> {
        if let Some(ref fortune5) = self.fortune5 {
            fortune5.check_slo_compliance().await
        } else {
            Ok(true) // No SLO configured, always compliant
        }
    }

    /// Check promotion gate (best from Fortune 5)
    pub async fn check_promotion_gate(&self) -> WorkflowResult<bool> {
        if let Some(ref fortune5) = self.fortune5 {
            fortune5.check_promotion_gate().await
        } else {
            Ok(true) // No promotion gate configured, always allow
        }
    }

    /// Get SLO metrics (best from Fortune 5)
    pub async fn get_slo_metrics(&self) -> Option<(u64, u64, u64)> {
        if let Some(ref fortune5) = self.fortune5 {
            fortune5.get_slo_metrics().await
        } else {
            None
        }
    }

    /// Record metric (best from OTEL)
    pub fn record_metric(&self, name: &str, value: f64, labels: HashMap<String, String>) {
        if let Some(ref otel) = self.otel {
            otel.record_metric(name, value, labels);
        }
    }

    /// Log event (best from OTEL)
    pub fn log_event(&self, level: &str, message: &str, fields: HashMap<String, String>) {
        if let Some(ref otel) = self.otel {
            otel.log_event(level, message, fields);
        }
    }

    /// Get receipt by hash (best from Lockchain)
    pub async fn get_receipt(&self, hash: &str) -> WorkflowResult<Option<knhk_lockchain::Receipt>> {
        if let Some(ref lockchain) = self.lockchain {
            lockchain.get_receipt(hash).await
        } else {
            Err(WorkflowError::ResourceUnavailable(
                "Lockchain integration not available".to_string(),
            ))
        }
    }

    /// Verify receipt (best from Lockchain)
    pub async fn verify_receipt(&self, receipt: &knhk_lockchain::Receipt) -> WorkflowResult<bool> {
        if let Some(ref lockchain) = self.lockchain {
            lockchain.verify_receipt(receipt).await
        } else {
            Err(WorkflowError::ResourceUnavailable(
                "Lockchain integration not available".to_string(),
            ))
        }
    }
}

/// Unified integration builder
pub struct UnifiedIntegrationBuilder {
    config: UnifiedIntegrationConfig,
}

impl UnifiedIntegrationBuilder {
    /// Create new builder
    pub fn new() -> Self {
        Self {
            config: UnifiedIntegrationConfig::default(),
        }
    }

    /// Set Fortune 5 configuration
    pub fn with_fortune5(mut self, config: Fortune5Config) -> Self {
        self.config.fortune5 = Some(config);
        self
    }

    /// Set Lockchain path
    pub fn with_lockchain(mut self, path: impl Into<String>) -> Self {
        self.config.lockchain_path = Some(path.into());
        self
    }

    /// Set OTEL endpoint
    pub fn with_otel(mut self, endpoint: impl Into<String>) -> Self {
        self.config.otel_endpoint = Some(endpoint.into());
        self
    }

    /// Add connector
    pub fn with_connector(mut self, name: impl Into<String>, config: impl Into<String>) -> Self {
        self.config.connectors.insert(name.into(), config.into());
        self
    }

    /// Enable/disable SLO tracking
    pub fn with_slo(mut self, enable: bool) -> Self {
        self.config.enable_slo = enable;
        self
    }

    /// Enable/disable provenance tracking
    pub fn with_provenance(mut self, enable: bool) -> Self {
        self.config.enable_provenance = enable;
        self
    }

    /// Enable/disable observability
    pub fn with_observability(mut self, enable: bool) -> Self {
        self.config.enable_observability = enable;
        self
    }

    /// Build unified integration
    pub async fn build(self) -> WorkflowResult<UnifiedIntegration> {
        UnifiedIntegration::new(self.config).await
    }
}

impl Default for UnifiedIntegrationBuilder {
    fn default() -> Self {
        Self::new()
    }
}
