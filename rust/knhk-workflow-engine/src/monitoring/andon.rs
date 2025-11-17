//! Andon System - Visual Control and Alert System
//!
//! Implements Toyota Production System's Andon concept for workflow monitoring:
//! - Visual indicators (Green/Yellow/Red) for workflow health
//! - Real-time alerting when issues are detected
//! - Ability to "stop the line" (pause workflows) when critical issues occur
//! - Integration with OTEL for observability
//!
//! # Andon States
//!
//! - **Green**: All workflows healthy, no issues
//! - **Yellow**: Warning conditions detected, monitoring
//! - **Red**: Critical issue detected, workflow paused/stopped
//!
//! # TRIZ Principles Applied
//!
//! - **Principle 10: Prior Action**: Pre-configured alert thresholds
//! - **Principle 15: Dynamics**: Real-time state updates
//! - **Principle 24: Intermediary**: Andon as intermediary between execution and operators

use crate::case::CaseId;
use crate::error::{WorkflowError, WorkflowResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{error, info, warn};

/// Andon state (visual indicator)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AndonState {
    /// Green: All systems operational
    Green,
    /// Yellow: Warning conditions present
    Yellow,
    /// Red: Critical issue, workflow stopped
    Red,
}

impl AndonState {
    /// Get priority level (higher = more critical)
    pub fn priority(&self) -> u8 {
        match self {
            AndonState::Green => 0,
            AndonState::Yellow => 1,
            AndonState::Red => 2,
        }
    }

    /// Check if workflow should be stopped
    pub fn should_stop(&self) -> bool {
        matches!(self, AndonState::Red)
    }
}

impl std::fmt::Display for AndonState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AndonState::Green => write!(f, "🟢 GREEN"),
            AndonState::Yellow => write!(f, "🟡 YELLOW"),
            AndonState::Red => write!(f, "🔴 RED"),
        }
    }
}

/// Andon alert type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AndonAlertType {
    /// Performance degradation (latency > threshold)
    PerformanceDegradation,
    /// Error rate exceeded threshold
    HighErrorRate,
    /// Guard violation detected
    GuardViolation,
    /// Tick budget exceeded
    TickBudgetExceeded,
    /// Resource exhaustion
    ResourceExhaustion,
    /// Workflow deadlock detected
    DeadlockDetected,
    /// Data corruption detected
    DataCorruption,
    /// External system failure
    ExternalSystemFailure,
}

/// Andon alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AndonAlert {
    /// Alert type
    pub alert_type: AndonAlertType,
    /// Alert severity
    pub severity: AndonState,
    /// Alert message
    pub message: String,
    /// Workflow/case ID (if applicable)
    pub workflow_id: Option<String>,
    pub case_id: Option<CaseId>,
    /// Timestamp (Unix epoch milliseconds)
    pub timestamp_ms: u64,
    /// Additional context data
    pub context: HashMap<String, String>,
}

impl AndonAlert {
    /// Create new alert
    pub fn new(alert_type: AndonAlertType, severity: AndonState, message: String) -> Self {
        Self {
            alert_type,
            severity,
            message,
            workflow_id: None,
            case_id: None,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
            context: HashMap::new(),
        }
    }

    /// Add workflow context
    pub fn with_workflow(mut self, workflow_id: String) -> Self {
        self.workflow_id = Some(workflow_id);
        self
    }

    /// Add case context
    pub fn with_case(mut self, case_id: CaseId) -> Self {
        self.case_id = Some(case_id);
        self
    }

    /// Add context data
    pub fn with_context(mut self, key: String, value: String) -> Self {
        self.context.insert(key, value);
        self
    }
}

/// Andon system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AndonConfig {
    /// Maximum allowed latency (microseconds) before yellow alert
    pub max_latency_us: u64,
    /// Critical latency threshold (microseconds) for red alert
    pub critical_latency_us: u64,
    /// Maximum error rate (errors per second) before alert
    pub max_error_rate: f64,
    /// Maximum tick budget before alert
    pub max_tick_budget: u32,
    /// Enable automatic workflow stopping on red alerts
    pub auto_stop_on_red: bool,
    /// Alert retention period (seconds)
    pub alert_retention_secs: u64,
}

impl Default for AndonConfig {
    fn default() -> Self {
        Self {
            max_latency_us: 500_000,        // 500ms
            critical_latency_us: 1_000_000, // 1s
            max_error_rate: 0.1,            // 10% error rate
            max_tick_budget: 8,             // Chatman Constant
            auto_stop_on_red: true,
            alert_retention_secs: 3600, // 1 hour
        }
    }
}

/// Andon system - visual control and alerting
pub struct AndonSystem {
    /// Current andon state
    state: Arc<RwLock<AndonState>>,
    /// Active alerts
    alerts: Arc<RwLock<Vec<AndonAlert>>>,
    /// Configuration
    config: Arc<AndonConfig>,
    /// Alert broadcast channel
    alert_tx: broadcast::Sender<AndonAlert>,
    /// Statistics
    stats: Arc<AndonStats>,
    /// Whether system is enabled
    enabled: Arc<AtomicBool>,
}

/// Andon statistics
#[derive(Debug, Default)]
struct AndonStats {
    /// Total alerts raised
    total_alerts: AtomicU64,
    /// Green state duration (milliseconds)
    green_duration_ms: AtomicU64,
    /// Yellow state duration (milliseconds)
    yellow_duration_ms: AtomicU64,
    /// Red state duration (milliseconds)
    red_duration_ms: AtomicU64,
    /// State transitions
    state_transitions: AtomicU64,
}

impl AndonSystem {
    /// Create new andon system
    pub fn new(config: AndonConfig) -> Self {
        let (tx, _) = broadcast::channel(1000);
        Self {
            state: Arc::new(RwLock::new(AndonState::Green)),
            alerts: Arc::new(RwLock::new(Vec::new())),
            config: Arc::new(config),
            alert_tx: tx,
            stats: Arc::new(AndonStats::default()),
            enabled: Arc::new(AtomicBool::new(true)),
        }
    }

    /// Get current andon state
    pub async fn state(&self) -> AndonState {
        *self.state.read().await
    }

    /// Check if workflows should be stopped
    pub async fn should_stop_workflows(&self) -> bool {
        if !self.enabled.load(Ordering::Relaxed) {
            return false;
        }
        self.state().await.should_stop()
    }

    /// Raise an alert
    pub async fn raise_alert(&self, alert: AndonAlert) -> WorkflowResult<()> {
        if !self.enabled.load(Ordering::Relaxed) {
            return Ok(()); // System disabled, ignore alerts
        }

        // Update state based on alert severity
        let new_state = match alert.severity {
            AndonState::Red => {
                if self.config.auto_stop_on_red {
                    warn!("🔴 ANDON RED: {}", alert.message);
                    error!("Workflow execution STOPPED due to critical alert");
                } else {
                    warn!("🔴 ANDON RED: {}", alert.message);
                }
                AndonState::Red
            }
            AndonState::Yellow => {
                warn!("🟡 ANDON YELLOW: {}", alert.message);
                AndonState::Yellow
            }
            AndonState::Green => {
                info!("🟢 ANDON GREEN: {}", alert.message);
                AndonState::Green
            }
        };

        // Update state if new state is more severe
        {
            let mut current_state = self.state.write().await;
            if new_state.priority() > current_state.priority() {
                self.stats.state_transitions.fetch_add(1, Ordering::Relaxed);
                *current_state = new_state;
            }
        }

        // Store alert
        {
            let mut alerts = self.alerts.write().await;
            alerts.push(alert.clone());
            self.stats.total_alerts.fetch_add(1, Ordering::Relaxed);

            // Cleanup old alerts
            let retention_ms = self.config.alert_retention_secs * 1000;
            let cutoff = alert.timestamp_ms.saturating_sub(retention_ms);
            alerts.retain(|a| a.timestamp_ms >= cutoff);
        }

        // Broadcast alert
        let _ = self.alert_tx.send(alert);

        Ok(())
    }

    /// Check performance metrics and raise alerts if needed
    pub async fn check_performance(
        &self,
        latency_us: u64,
        error_count: u64,
        total_requests: u64,
    ) -> WorkflowResult<()> {
        // Check latency
        if latency_us > self.config.critical_latency_us {
            self.raise_alert(
                AndonAlert::new(
                    AndonAlertType::PerformanceDegradation,
                    AndonState::Red,
                    format!(
                        "Critical latency: {}μs (threshold: {}μs)",
                        latency_us, self.config.critical_latency_us
                    ),
                )
                .with_context("latency_us".to_string(), latency_us.to_string())
                .with_context(
                    "threshold_us".to_string(),
                    self.config.critical_latency_us.to_string(),
                ),
            )
            .await?;
        } else if latency_us > self.config.max_latency_us {
            self.raise_alert(
                AndonAlert::new(
                    AndonAlertType::PerformanceDegradation,
                    AndonState::Yellow,
                    format!(
                        "High latency: {}μs (threshold: {}μs)",
                        latency_us, self.config.max_latency_us
                    ),
                )
                .with_context("latency_us".to_string(), latency_us.to_string())
                .with_context(
                    "threshold_us".to_string(),
                    self.config.max_latency_us.to_string(),
                ),
            )
            .await?;
        }

        // Check error rate
        if total_requests > 0 {
            let error_rate = error_count as f64 / total_requests as f64;
            if error_rate > self.config.max_error_rate {
                self.raise_alert(
                    AndonAlert::new(
                        AndonAlertType::HighErrorRate,
                        AndonState::Yellow,
                        format!(
                            "High error rate: {:.2}% (threshold: {:.2}%)",
                            error_rate * 100.0,
                            self.config.max_error_rate * 100.0
                        ),
                    )
                    .with_context("error_rate".to_string(), error_rate.to_string())
                    .with_context("error_count".to_string(), error_count.to_string())
                    .with_context("total_requests".to_string(), total_requests.to_string()),
                )
                .await?;
            }
        }

        Ok(())
    }

    /// Check tick budget and raise alert if exceeded
    pub async fn check_tick_budget(&self, ticks_used: u32) -> WorkflowResult<()> {
        if ticks_used > self.config.max_tick_budget {
            self.raise_alert(
                AndonAlert::new(
                    AndonAlertType::TickBudgetExceeded,
                    AndonState::Red,
                    format!(
                        "Tick budget exceeded: {} ticks (limit: {} ticks)",
                        ticks_used, self.config.max_tick_budget
                    ),
                )
                .with_context("ticks_used".to_string(), ticks_used.to_string())
                .with_context(
                    "max_ticks".to_string(),
                    self.config.max_tick_budget.to_string(),
                ),
            )
            .await?;
        }
        Ok(())
    }

    /// Check guard violation and raise alert
    pub async fn check_guard_violation(
        &self,
        guard_name: String,
        workflow_id: Option<String>,
        case_id: Option<CaseId>,
    ) -> WorkflowResult<()> {
        let mut alert = AndonAlert::new(
            AndonAlertType::GuardViolation,
            AndonState::Red,
            format!("Guard violation: {}", guard_name),
        )
        .with_context("guard_name".to_string(), guard_name);

        if let Some(wf_id) = workflow_id {
            alert = alert.with_workflow(wf_id);
        }
        if let Some(c_id) = case_id {
            alert = alert.with_case(c_id);
        }

        self.raise_alert(alert).await
    }

    /// Clear alert and return to green (manual reset)
    pub async fn clear_alert(&self) -> WorkflowResult<()> {
        let mut state = self.state.write().await;
        if *state != AndonState::Green {
            self.stats.state_transitions.fetch_add(1, Ordering::Relaxed);
            *state = AndonState::Green;
            info!("🟢 ANDON cleared - returning to GREEN");
        }
        Ok(())
    }

    /// Get active alerts
    pub async fn get_alerts(&self) -> Vec<AndonAlert> {
        self.alerts.read().await.clone()
    }

    /// Get alerts by severity
    pub async fn get_alerts_by_severity(&self, severity: AndonState) -> Vec<AndonAlert> {
        self.alerts
            .read()
            .await
            .iter()
            .filter(|a| a.severity == severity)
            .cloned()
            .collect()
    }

    /// Subscribe to alert stream
    pub fn subscribe_alerts(&self) -> broadcast::Receiver<AndonAlert> {
        self.alert_tx.subscribe()
    }

    /// Enable/disable andon system
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Relaxed);
        if enabled {
            info!("🟢 ANDON system ENABLED");
        } else {
            warn!("⚫ ANDON system DISABLED");
        }
    }

    /// Get statistics
    pub fn get_stats(&self) -> AndonStatsSnapshot {
        AndonStatsSnapshot {
            total_alerts: self.stats.total_alerts.load(Ordering::Relaxed),
            state_transitions: self.stats.state_transitions.load(Ordering::Relaxed),
            enabled: self.enabled.load(Ordering::Relaxed),
        }
    }
}

/// Andon statistics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AndonStatsSnapshot {
    /// Total alerts raised
    pub total_alerts: u64,
    /// State transitions
    pub state_transitions: u64,
    /// Whether system is enabled
    pub enabled: bool,
}
