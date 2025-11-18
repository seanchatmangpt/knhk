//! Guardian Agent - Validates decisions and enforces invariants

use super::{Agent, AgentConfig};
use crate::error::{SwarmError, SwarmResult};
use crate::types::{AgentId, AgentRole, AgentState};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, instrument, warn};

/// Invariant violation detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvariantViolation {
    pub invariant_name: String,
    pub description: String,
    pub severity: ViolationSeverity,
}

/// Severity of violation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// The Guardian Agent
pub struct GuardianAgent {
    id: AgentId,
    config: AgentConfig,
    state: Arc<RwLock<AgentState>>,
    violations: Arc<RwLock<Vec<InvariantViolation>>>,
}

impl GuardianAgent {
    /// Create a new Guardian agent
    #[instrument]
    pub fn new(id: AgentId, config: AgentConfig) -> SwarmResult<Self> {
        info!("Creating Guardian agent: {}", id);

        Ok(Self {
            id,
            config,
            state: Arc::new(RwLock::new(AgentState::Working)),
            violations: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// Validate a decision
    pub async fn validate_decision(&self, decision: &str) -> SwarmResult<bool> {
        // Placeholder validation logic
        // In production, this would check against defined invariants

        if decision.contains("unsafe") {
            let violation = InvariantViolation {
                invariant_name: "safety_constraint".into(),
                description: "Decision contains unsafe operation".into(),
                severity: ViolationSeverity::High,
            };

            self.report_violation(violation).await;
            return Ok(false);
        }

        Ok(true)
    }

    /// Report an invariant violation
    pub async fn report_violation(&self, violation: InvariantViolation) {
        warn!(
            "Guardian {} detected violation: {} (severity: {:?})",
            self.id, violation.invariant_name, violation.severity
        );

        self.violations.write().await.push(violation);
    }

    /// Get all violations
    pub async fn get_violations(&self) -> Vec<InvariantViolation> {
        self.violations.read().await.clone()
    }

    /// Clear violations
    pub async fn clear_violations(&self) {
        self.violations.write().await.clear();
    }
}

impl Agent for GuardianAgent {
    fn id(&self) -> AgentId {
        self.id
    }

    fn role(&self) -> AgentRole {
        self.config.role
    }

    fn state(&self) -> AgentState {
        AgentState::Working
    }

    async fn start(&mut self) {
        *self.state.write().await = AgentState::Working;
        info!("Guardian agent {} started", self.id);
    }

    async fn stop(&mut self) {
        *self.state.write().await = AgentState::Idle;
        info!("Guardian agent {} stopped", self.id);
    }

    async fn health_check(&self) -> bool {
        let state = self.state.read().await;
        !matches!(*state, AgentState::Failed)
    }
}
