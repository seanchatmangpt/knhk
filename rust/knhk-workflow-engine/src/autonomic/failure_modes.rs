// rust/knhk-workflow-engine/src/autonomic/failure_modes.rs
//! Doctrine-Aware Failure Modes for MAPE-K Loop
//!
//! Implements safe ceiling behavior when the autonomic loop itself degrades.
//! The system automatically limits its scope of action based on health signals.
//!
//! **Philosophy**:
//! - Do less, not more, when uncertain
//! - Prefer safety over throughput when degraded
//! - Explicit failure > silent incorrect behavior
//! - Mode changes are observable and receipted
//!
//! **Mode Hierarchy**:
//! - `Normal`: Full MAPE-K, all adaptations allowed
//! - `Conservative`: Runtime tuning only, limited actions
//! - `Frozen`: Read-only, observation and alerting only
//!
//! **Design Principles**:
//! 1. Self-limiting behavior under degradation
//! 2. Graceful degradation with clear semantics
//! 3. Operator visibility into mode changes
//! 4. Health signal aggregation from all MAPE components
//! 5. Automatic and manual mode transitions

use crate::error::{WorkflowError, WorkflowResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// Autonomic operating mode
///
/// Determines what actions are permitted in the autonomic loop.
/// Mode degradation is a safety mechanism - when we can't trust
/// our own decision-making, we limit what we're allowed to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AutonomicMode {
    /// Normal operation - all adaptations allowed
    ///
    /// - Full MAPE-K loop operational
    /// - Complex actions permitted (scaling, migration, etc.)
    /// - Overlays and ΔΣ changes allowed
    /// - All action types available
    Normal,

    /// Conservative operation - limited safe actions only
    ///
    /// - Runtime tuning only
    /// - No structural changes (no scaling, migration)
    /// - No ΔΣ modifications
    /// - Limited to low-risk actions
    /// - Alerting on degradation
    Conservative,

    /// Frozen operation - read-only, no actions
    ///
    /// - Observation only
    /// - Alerting and receipt emission
    /// - No adaptations of any kind
    /// - Fail-safe mode when loop health is critical
    Frozen,
}

impl AutonomicMode {
    /// Check if this mode allows a more permissive mode
    pub fn allows(&self, other: AutonomicMode) -> bool {
        use AutonomicMode::*;
        match (self, other) {
            (Normal, _) => true,
            (Conservative, Conservative) | (Conservative, Frozen) => true,
            (Frozen, Frozen) => true,
            _ => false,
        }
    }

    /// Get mode severity (higher = more restrictive)
    pub const fn severity(&self) -> u8 {
        match self {
            AutonomicMode::Normal => 0,
            AutonomicMode::Conservative => 1,
            AutonomicMode::Frozen => 2,
        }
    }

    /// Get human-readable description
    pub const fn description(&self) -> &'static str {
        match self {
            AutonomicMode::Normal => "Full autonomic operation with all adaptations enabled",
            AutonomicMode::Conservative => "Limited operation with only safe runtime tuning",
            AutonomicMode::Frozen => "Read-only observation mode with no adaptations",
        }
    }
}

impl Default for AutonomicMode {
    fn default() -> Self {
        AutonomicMode::Normal
    }
}

impl std::fmt::Display for AutonomicMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AutonomicMode::Normal => write!(f, "Normal"),
            AutonomicMode::Conservative => write!(f, "Conservative"),
            AutonomicMode::Frozen => write!(f, "Frozen"),
        }
    }
}

/// Health signal from a MAPE-K component
///
/// Each component reports its own health, and the ModeManager
/// aggregates these to determine overall system mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSignal {
    /// Component name (monitor, analyzer, planner, executor, knowledge)
    pub component: ComponentType,
    /// Health score (0.0 = critical, 1.0 = perfect)
    pub score: f64,
    /// Timestamp when signal was generated
    pub timestamp_ms: u64,
    /// Optional details about health issues
    pub details: Option<String>,
}

impl HealthSignal {
    pub fn new(component: ComponentType, score: f64) -> Self {
        Self {
            component,
            score: score.clamp(0.0, 1.0),
            timestamp_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
            details: None,
        }
    }

    pub fn with_details(mut self, details: String) -> Self {
        self.details = Some(details);
        self
    }

    /// Check if signal is stale (older than threshold)
    pub fn is_stale(&self, max_age_ms: u64) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        now.saturating_sub(self.timestamp_ms) > max_age_ms
    }
}

/// MAPE-K component type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComponentType {
    Monitor,
    Analyzer,
    Planner,
    Executor,
    Knowledge,
}

impl std::fmt::Display for ComponentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComponentType::Monitor => write!(f, "Monitor"),
            ComponentType::Analyzer => write!(f, "Analyzer"),
            ComponentType::Planner => write!(f, "Planner"),
            ComponentType::Executor => write!(f, "Executor"),
            ComponentType::Knowledge => write!(f, "Knowledge"),
        }
    }
}

/// Health metrics for mode determination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    /// Monitor completeness (% of expected metrics)
    pub monitor_completeness: f64,
    /// Analyzer confidence (quality of anomaly detection)
    pub analyzer_confidence: f64,
    /// Planner viability (can generate valid plans)
    pub planner_viability: f64,
    /// Executor reliability (action success rate)
    pub executor_reliability: f64,
    /// Knowledge staleness (how old is data)
    pub knowledge_staleness_ms: u64,
    /// Overall health score (0.0-1.0)
    pub overall_score: f64,
}

impl HealthMetrics {
    /// Calculate overall health score from component signals
    pub fn from_signals(signals: &HashMap<ComponentType, HealthSignal>) -> Self {
        let monitor_completeness = signals
            .get(&ComponentType::Monitor)
            .map(|s| s.score)
            .unwrap_or(0.0);

        let analyzer_confidence = signals
            .get(&ComponentType::Analyzer)
            .map(|s| s.score)
            .unwrap_or(0.0);

        let planner_viability = signals
            .get(&ComponentType::Planner)
            .map(|s| s.score)
            .unwrap_or(0.0);

        let executor_reliability = signals
            .get(&ComponentType::Executor)
            .map(|s| s.score)
            .unwrap_or(1.0);

        let knowledge_staleness_ms = signals
            .get(&ComponentType::Knowledge)
            .and_then(|s| {
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .ok()
                    .map(|now| now.as_millis() as u64 - s.timestamp_ms)
            })
            .unwrap_or(0);

        // Calculate weighted overall score
        // Monitor and Analyzer are critical for decision-making
        let overall_score = (monitor_completeness * 0.3
            + analyzer_confidence * 0.3
            + planner_viability * 0.2
            + executor_reliability * 0.2)
            .clamp(0.0, 1.0);

        Self {
            monitor_completeness,
            analyzer_confidence,
            planner_viability,
            executor_reliability,
            knowledge_staleness_ms,
            overall_score,
        }
    }

    /// Determine appropriate mode based on health metrics
    pub fn determine_mode(&self) -> AutonomicMode {
        // Critical thresholds for mode degradation
        const FROZEN_THRESHOLD: f64 = 0.3;
        const CONSERVATIVE_THRESHOLD: f64 = 0.6;

        // If Monitor or Analyzer are severely degraded, freeze
        if self.monitor_completeness < FROZEN_THRESHOLD
            || self.analyzer_confidence < FROZEN_THRESHOLD
        {
            return AutonomicMode::Frozen;
        }

        // If overall health is critical, freeze
        if self.overall_score < FROZEN_THRESHOLD {
            return AutonomicMode::Frozen;
        }

        // If health is degraded, go conservative
        if self.overall_score < CONSERVATIVE_THRESHOLD {
            return AutonomicMode::Conservative;
        }

        // If Monitor or Analyzer are degraded, go conservative
        if self.monitor_completeness < CONSERVATIVE_THRESHOLD
            || self.analyzer_confidence < CONSERVATIVE_THRESHOLD
        {
            return AutonomicMode::Conservative;
        }

        AutonomicMode::Normal
    }
}

/// Mode change event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeChangeEvent {
    /// Previous mode
    pub from: AutonomicMode,
    /// New mode
    pub to: AutonomicMode,
    /// Reason for change
    pub reason: String,
    /// Health metrics at time of change
    pub metrics: HealthMetrics,
    /// Timestamp
    pub timestamp_ms: u64,
    /// Whether change was manual override
    pub manual_override: bool,
}

impl ModeChangeEvent {
    pub fn new(
        from: AutonomicMode,
        to: AutonomicMode,
        reason: String,
        metrics: HealthMetrics,
        manual_override: bool,
    ) -> Self {
        Self {
            from,
            to,
            reason,
            metrics,
            timestamp_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
            manual_override,
        }
    }
}

/// Mode manager for autonomic loop
///
/// Aggregates health signals from all MAPE-K components and determines
/// the appropriate operating mode. Emits telemetry on mode changes.
pub struct ModeManager {
    /// Current mode
    current_mode: Arc<RwLock<AutonomicMode>>,
    /// Health signals from components
    signals: Arc<RwLock<HashMap<ComponentType, HealthSignal>>>,
    /// Mode change history
    history: Arc<RwLock<Vec<ModeChangeEvent>>>,
    /// Maximum age for health signals (ms)
    signal_max_age_ms: u64,
    /// Manual override (if set, ignores health signals)
    manual_override: Arc<RwLock<Option<AutonomicMode>>>,
}

impl ModeManager {
    /// Create new mode manager
    pub fn new() -> Self {
        Self {
            current_mode: Arc::new(RwLock::new(AutonomicMode::Normal)),
            signals: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(RwLock::new(Vec::new())),
            signal_max_age_ms: 60_000, // 60 seconds
            manual_override: Arc::new(RwLock::new(None)),
        }
    }

    /// Get current mode
    pub async fn current_mode(&self) -> AutonomicMode {
        *self.current_mode.read().await
    }

    /// Update health signal from a component
    pub async fn update_health(&self, signal: HealthSignal) -> WorkflowResult<()> {
        let mut signals = self.signals.write().await;
        signals.insert(signal.component, signal);
        drop(signals);

        // Re-evaluate mode based on new signal
        self.evaluate_mode().await
    }

    /// Evaluate and potentially change mode based on health signals
    pub async fn evaluate_mode(&self) -> WorkflowResult<()> {
        // Check for manual override first
        let manual = *self.manual_override.read().await;
        if let Some(override_mode) = manual {
            let current = *self.current_mode.read().await;
            if current != override_mode {
                self.change_mode(
                    override_mode,
                    format!("Manual override to {}", override_mode),
                    true,
                )
                .await?;
            }
            return Ok(());
        }

        // Get current signals
        let signals = self.signals.read().await;

        // Remove stale signals
        let fresh_signals: HashMap<ComponentType, HealthSignal> = signals
            .iter()
            .filter(|(_, signal)| !signal.is_stale(self.signal_max_age_ms))
            .map(|(k, v)| (*k, v.clone()))
            .collect();

        drop(signals);

        // Calculate health metrics
        let metrics = HealthMetrics::from_signals(&fresh_signals);

        // Determine appropriate mode
        let target_mode = metrics.determine_mode();
        let current_mode = *self.current_mode.read().await;

        // Change mode if needed
        if target_mode != current_mode {
            let reason = format!(
                "Health-driven mode change (score: {:.2}, monitor: {:.2}, analyzer: {:.2})",
                metrics.overall_score, metrics.monitor_completeness, metrics.analyzer_confidence
            );
            self.change_mode_with_metrics(target_mode, reason, metrics, false)
                .await?;
        }

        Ok(())
    }

    /// Change mode with full context
    async fn change_mode_with_metrics(
        &self,
        new_mode: AutonomicMode,
        reason: String,
        metrics: HealthMetrics,
        manual: bool,
    ) -> WorkflowResult<()> {
        let mut current = self.current_mode.write().await;
        let old_mode = *current;

        if old_mode == new_mode {
            return Ok(());
        }

        // Create mode change event
        let event = ModeChangeEvent::new(old_mode, new_mode, reason.clone(), metrics, manual);

        // Update mode
        *current = new_mode;
        drop(current);

        // Record in history
        let mut history = self.history.write().await;
        history.push(event.clone());
        drop(history);

        // Emit telemetry
        match new_mode.severity().cmp(&old_mode.severity()) {
            std::cmp::Ordering::Greater => {
                warn!(
                    mode.from = %old_mode,
                    mode.to = %new_mode,
                    mode.reason = %reason,
                    mode.manual = manual,
                    "Autonomic mode degraded"
                );
            }
            std::cmp::Ordering::Less => {
                info!(
                    mode.from = %old_mode,
                    mode.to = %new_mode,
                    mode.reason = %reason,
                    mode.manual = manual,
                    "Autonomic mode improved"
                );
            }
            std::cmp::Ordering::Equal => {
                info!(
                    mode.from = %old_mode,
                    mode.to = %new_mode,
                    mode.reason = %reason,
                    mode.manual = manual,
                    "Autonomic mode changed"
                );
            }
        }

        Ok(())
    }

    /// Change mode (without metrics)
    async fn change_mode(
        &self,
        new_mode: AutonomicMode,
        reason: String,
        manual: bool,
    ) -> WorkflowResult<()> {
        let signals = self.signals.read().await;
        let metrics = HealthMetrics::from_signals(&signals);
        drop(signals);

        self.change_mode_with_metrics(new_mode, reason, metrics, manual)
            .await
    }

    /// Manually override mode
    pub async fn set_manual_override(&self, mode: AutonomicMode) -> WorkflowResult<()> {
        let mut override_guard = self.manual_override.write().await;
        *override_guard = Some(mode);
        drop(override_guard);

        self.change_mode(
            mode,
            format!("Manual override to {}", mode),
            true,
        )
        .await
    }

    /// Clear manual override
    pub async fn clear_manual_override(&self) -> WorkflowResult<()> {
        let mut override_guard = self.manual_override.write().await;
        *override_guard = None;
        drop(override_guard);

        info!("Manual mode override cleared, resuming automatic mode management");

        // Re-evaluate based on health
        self.evaluate_mode().await
    }

    /// Get health metrics
    pub async fn get_health_metrics(&self) -> HealthMetrics {
        let signals = self.signals.read().await;
        HealthMetrics::from_signals(&signals)
    }

    /// Get mode change history
    pub async fn get_history(&self) -> Vec<ModeChangeEvent> {
        let history = self.history.read().await;
        history.clone()
    }

    /// Get current health signals
    pub async fn get_signals(&self) -> HashMap<ComponentType, HealthSignal> {
        let signals = self.signals.read().await;
        signals.clone()
    }
}

impl Default for ModeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mode_hierarchy() {
        assert!(AutonomicMode::Normal.allows(AutonomicMode::Normal));
        assert!(AutonomicMode::Normal.allows(AutonomicMode::Conservative));
        assert!(AutonomicMode::Normal.allows(AutonomicMode::Frozen));

        assert!(!AutonomicMode::Conservative.allows(AutonomicMode::Normal));
        assert!(AutonomicMode::Conservative.allows(AutonomicMode::Conservative));
        assert!(AutonomicMode::Conservative.allows(AutonomicMode::Frozen));

        assert!(!AutonomicMode::Frozen.allows(AutonomicMode::Normal));
        assert!(!AutonomicMode::Frozen.allows(AutonomicMode::Conservative));
        assert!(AutonomicMode::Frozen.allows(AutonomicMode::Frozen));
    }

    #[tokio::test]
    async fn test_health_signal() {
        let signal = HealthSignal::new(ComponentType::Monitor, 0.8);
        assert_eq!(signal.component, ComponentType::Monitor);
        assert!((signal.score - 0.8).abs() < 0.01);
        assert!(!signal.is_stale(60_000));
    }

    #[tokio::test]
    async fn test_health_metrics_normal() {
        let mut signals = HashMap::new();
        signals.insert(
            ComponentType::Monitor,
            HealthSignal::new(ComponentType::Monitor, 0.9),
        );
        signals.insert(
            ComponentType::Analyzer,
            HealthSignal::new(ComponentType::Analyzer, 0.85),
        );
        signals.insert(
            ComponentType::Planner,
            HealthSignal::new(ComponentType::Planner, 0.8),
        );
        signals.insert(
            ComponentType::Executor,
            HealthSignal::new(ComponentType::Executor, 0.95),
        );

        let metrics = HealthMetrics::from_signals(&signals);
        assert!(metrics.overall_score > 0.6);

        let mode = metrics.determine_mode();
        assert_eq!(mode, AutonomicMode::Normal);
    }

    #[tokio::test]
    async fn test_health_metrics_conservative() {
        let mut signals = HashMap::new();
        signals.insert(
            ComponentType::Monitor,
            HealthSignal::new(ComponentType::Monitor, 0.55), // Below 0.6
        );
        signals.insert(
            ComponentType::Analyzer,
            HealthSignal::new(ComponentType::Analyzer, 0.7),
        );

        let metrics = HealthMetrics::from_signals(&signals);
        let mode = metrics.determine_mode();
        assert_eq!(mode, AutonomicMode::Conservative);
    }

    #[tokio::test]
    async fn test_health_metrics_frozen() {
        let mut signals = HashMap::new();
        signals.insert(
            ComponentType::Monitor,
            HealthSignal::new(ComponentType::Monitor, 0.2), // Below 0.3
        );
        signals.insert(
            ComponentType::Analyzer,
            HealthSignal::new(ComponentType::Analyzer, 0.5),
        );

        let metrics = HealthMetrics::from_signals(&signals);
        let mode = metrics.determine_mode();
        assert_eq!(mode, AutonomicMode::Frozen);
    }

    #[tokio::test]
    async fn test_mode_manager_automatic_degradation() {
        let manager = ModeManager::new();

        // Start in normal mode
        assert_eq!(manager.current_mode().await, AutonomicMode::Normal);

        // Update with degraded monitor signal
        let signal = HealthSignal::new(ComponentType::Monitor, 0.5);
        manager.update_health(signal).await.unwrap();

        // Should degrade to conservative
        let mode = manager.current_mode().await;
        assert_eq!(mode, AutonomicMode::Conservative);
    }

    #[tokio::test]
    async fn test_mode_manager_manual_override() {
        let manager = ModeManager::new();

        // Set manual override to frozen
        manager
            .set_manual_override(AutonomicMode::Frozen)
            .await
            .unwrap();
        assert_eq!(manager.current_mode().await, AutonomicMode::Frozen);

        // Even with good health, should stay frozen
        let signal = HealthSignal::new(ComponentType::Monitor, 0.95);
        manager.update_health(signal).await.unwrap();
        assert_eq!(manager.current_mode().await, AutonomicMode::Frozen);

        // Clear override
        manager.clear_manual_override().await.unwrap();

        // Should return to normal with good health
        let signal = HealthSignal::new(ComponentType::Monitor, 0.95);
        manager.update_health(signal).await.unwrap();
        assert_eq!(manager.current_mode().await, AutonomicMode::Normal);
    }

    #[tokio::test]
    async fn test_mode_manager_recovery() {
        let manager = ModeManager::new();

        // Degrade to frozen
        manager
            .update_health(HealthSignal::new(ComponentType::Monitor, 0.2))
            .await
            .unwrap();
        assert_eq!(manager.current_mode().await, AutonomicMode::Frozen);

        // Improve to conservative
        manager
            .update_health(HealthSignal::new(ComponentType::Monitor, 0.55))
            .await
            .unwrap();
        manager
            .update_health(HealthSignal::new(ComponentType::Analyzer, 0.6))
            .await
            .unwrap();
        assert_eq!(manager.current_mode().await, AutonomicMode::Conservative);

        // Recover to normal
        manager
            .update_health(HealthSignal::new(ComponentType::Monitor, 0.9))
            .await
            .unwrap();
        manager
            .update_health(HealthSignal::new(ComponentType::Analyzer, 0.85))
            .await
            .unwrap();
        assert_eq!(manager.current_mode().await, AutonomicMode::Normal);
    }

    #[tokio::test]
    async fn test_mode_change_history() {
        let manager = ModeManager::new();

        // Make some mode changes
        manager
            .update_health(HealthSignal::new(ComponentType::Monitor, 0.5))
            .await
            .unwrap();
        manager
            .update_health(HealthSignal::new(ComponentType::Monitor, 0.2))
            .await
            .unwrap();

        let history = manager.get_history().await;
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].to, AutonomicMode::Conservative);
        assert_eq!(history[1].to, AutonomicMode::Frozen);
    }
}
