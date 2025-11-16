// rust/knhk-workflow-engine/src/autonomic/loop_controller.rs
//! MAPE-K Loop Controller
//!
//! Orchestrates the Monitor-Analyze-Plan-Execute cycle with shared Knowledge.

use super::analyze::Analyzer;
use super::execute::Executor;
use super::failure_modes::{AutonomicMode, ComponentType, HealthSignal, ModeManager};
use super::knowledge::KnowledgeBase;
use super::mode_policy::ModePolicyFilter;
use super::monitor::Monitor;
use super::plan::Planner;
use super::{AutonomicManager, AutonomicProperty, CycleStats};
use crate::error::{WorkflowError, WorkflowResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Controller state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControllerState {
    /// Controller is stopped
    Stopped,
    /// Controller is starting
    Starting,
    /// Controller is running
    Running,
    /// Controller is stopping
    Stopping,
    /// Controller encountered an error
    Error,
}

/// Controller configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerConfig {
    /// MAPE-K cycle interval
    pub cycle_interval: Duration,
    /// Enable self-healing
    pub self_healing: bool,
    /// Enable self-optimization
    pub self_optimization: bool,
    /// Enable self-configuration
    pub self_configuration: bool,
    /// Enable self-protection
    pub self_protection: bool,
}

impl Default for ControllerConfig {
    fn default() -> Self {
        Self {
            cycle_interval: Duration::from_secs(30), // 30s MAPE-K cycle
            self_healing: true,
            self_optimization: true,
            self_configuration: false,
            self_protection: false,
        }
    }
}

/// MAPE-K controller
pub struct MapeKController {
    /// Configuration
    config: ControllerConfig,
    /// Knowledge base (shared across all components)
    knowledge: Arc<KnowledgeBase>,
    /// Monitor component
    monitor: Arc<Monitor>,
    /// Analyzer component
    analyzer: Arc<Analyzer>,
    /// Planner component
    planner: Arc<Planner>,
    /// Executor component
    executor: Arc<Executor>,
    /// Controller state
    state: Arc<RwLock<ControllerState>>,
    /// Cycle statistics
    stats: Arc<RwLock<CycleStats>>,
    /// Mode manager (failure modes)
    mode_manager: Arc<ModeManager>,
    /// Mode-aware policy filter
    policy_filter: Arc<ModePolicyFilter>,
}

impl MapeKController {
    /// Create new MAPE-K controller
    pub fn new(config: ControllerConfig, monitor: Monitor) -> Self {
        let knowledge = Arc::new(KnowledgeBase::new());
        let analyzer = Arc::new(Analyzer::new(knowledge.clone()));
        let planner = Arc::new(Planner::new(knowledge.clone()));
        let executor = Arc::new(Executor::new());
        let mode_manager = Arc::new(ModeManager::new());
        let policy_filter = Arc::new(ModePolicyFilter::new());

        Self {
            config,
            knowledge: knowledge.clone(),
            monitor: Arc::new(monitor),
            analyzer,
            planner,
            executor,
            state: Arc::new(RwLock::new(ControllerState::Stopped)),
            stats: Arc::new(RwLock::new(CycleStats::default())),
            mode_manager,
            policy_filter,
        }
    }

    /// Get knowledge base
    pub fn knowledge(&self) -> Arc<KnowledgeBase> {
        self.knowledge.clone()
    }

    /// Get monitor
    pub fn monitor(&self) -> Arc<Monitor> {
        self.monitor.clone()
    }

    /// Get mode manager
    pub fn mode_manager(&self) -> Arc<ModeManager> {
        self.mode_manager.clone()
    }

    /// Get current state
    pub async fn state(&self) -> ControllerState {
        *self.state.read().await
    }

    /// Get current autonomic mode
    pub async fn autonomic_mode(&self) -> AutonomicMode {
        self.mode_manager.current_mode().await
    }

    /// Start MAPE-K loop
    pub async fn start_loop(&self) -> WorkflowResult<()> {
        let mut state = self.state.write().await;
        if *state == ControllerState::Running {
            return Err(WorkflowError::Internal(
                "Controller already running".to_string(),
            ));
        }

        *state = ControllerState::Starting;
        drop(state);

        // Start monitor
        self.monitor.start().await?;

        let mut state = self.state.write().await;
        *state = ControllerState::Running;
        drop(state);

        // Start MAPE-K cycle loop
        let config = self.config.clone();
        let knowledge = self.knowledge.clone();
        let monitor = self.monitor.clone();
        let analyzer = self.analyzer.clone();
        let planner = self.planner.clone();
        let executor = self.executor.clone();
        let mode_manager = self.mode_manager.clone();
        let policy_filter = self.policy_filter.clone();
        let state_ref = self.state.clone();
        let stats_ref = self.stats.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(config.cycle_interval);

            loop {
                let current_state = *state_ref.read().await;
                if current_state != ControllerState::Running {
                    break;
                }

                interval.tick().await;

                let cycle_start = std::time::Instant::now();

                // MAPE-K Cycle with mode management
                match Self::execute_cycle(
                    &monitor,
                    &analyzer,
                    &planner,
                    &executor,
                    &mode_manager,
                    &policy_filter,
                    &config,
                ).await {
                    Ok(cycle_result) => {
                        let mut stats = stats_ref.write().await;
                        stats.total_cycles += 1;

                        if cycle_result.adapted {
                            stats.successful_adaptations += 1;
                        }

                        stats.anomalies_detected += cycle_result.anomalies as u64;
                        stats.plans_generated += if cycle_result.plan_generated { 1 } else { 0 };
                        stats.actions_executed += cycle_result.actions_executed as u64;

                        // Update average cycle duration
                        let duration_ms = cycle_start.elapsed().as_millis() as f64;
                        stats.avg_cycle_duration_ms =
                            (stats.avg_cycle_duration_ms * (stats.total_cycles - 1) as f64 + duration_ms)
                                / stats.total_cycles as f64;
                    }
                    Err(e) => {
                        tracing::error!("MAPE-K cycle error: {}", e);
                        let mut stats = stats_ref.write().await;
                        stats.failed_adaptations += 1;
                    }
                }
            }
        });

        Ok(())
    }

    /// Execute single MAPE-K cycle with mode management
    async fn execute_cycle(
        monitor: &Monitor,
        analyzer: &Analyzer,
        planner: &Planner,
        executor: &Executor,
        mode_manager: &ModeManager,
        policy_filter: &ModePolicyFilter,
        config: &ControllerConfig,
    ) -> WorkflowResult<CycleResult> {
        let mut result = CycleResult::default();

        // Get current mode
        let current_mode = mode_manager.current_mode().await;
        result.mode = current_mode;

        // In Frozen mode, only observe (no adaptations)
        if current_mode == AutonomicMode::Frozen {
            tracing::info!(
                mode = ?current_mode,
                "MAPE-K cycle running in Frozen mode (observation only)"
            );
            return Ok(result);
        }

        // MONITOR: Collect metrics (already running)
        // Emit monitor health signal
        let monitor_health = if monitor.is_running().await {
            HealthSignal::new(ComponentType::Monitor, 0.9) // TODO: Calculate actual health
        } else {
            HealthSignal::new(ComponentType::Monitor, 0.0)
        };
        mode_manager.update_health(monitor_health).await?;

        // ANALYZE: Analyze current state
        let analysis = analyzer.analyze().await?;
        result.anomalies = analysis.anomalies.len();

        // Emit analyzer health signal based on analysis quality
        let analyzer_confidence = match analysis.health {
            super::analyze::HealthStatus::Healthy => 1.0,
            super::analyze::HealthStatus::Degraded => 0.7,
            super::analyze::HealthStatus::Unhealthy => 0.4,
            super::analyze::HealthStatus::Critical => 0.2,
        };
        mode_manager
            .update_health(HealthSignal::new(ComponentType::Analyzer, analyzer_confidence))
            .await?;

        if !analysis.adaptation_needed {
            return Ok(result);
        }

        // PLAN: Generate adaptation plan
        if let Some(plan) = planner.plan(&analysis).await? {
            result.plan_generated = true;

            // Emit planner health signal (1.0 if plan generated successfully)
            mode_manager
                .update_health(HealthSignal::new(ComponentType::Planner, 1.0))
                .await?;

            // Get current mode (may have changed after health updates)
            let execution_mode = mode_manager.current_mode().await;
            result.mode = execution_mode;

            // Filter actions based on current mode
            let (allowed_actions, rejected_actions) =
                policy_filter.filter_with_rejected(&plan.actions, execution_mode);

            result.actions_executed = allowed_actions.len();
            result.actions_rejected = rejected_actions.len();

            // Log rejected actions
            for rejected in &rejected_actions {
                tracing::warn!(
                    action_id = ?rejected.action.id,
                    action_type = ?rejected.action.action_type,
                    current_mode = ?rejected.current_mode,
                    required_mode = ?rejected.required_mode,
                    reason = %rejected.reason,
                    "Action rejected by mode policy"
                );
            }

            // EXECUTE: Apply allowed actions
            if config.self_healing || config.self_optimization {
                if !allowed_actions.is_empty() {
                    // Create filtered plan with only allowed actions
                    let mut filtered_plan = plan.clone();
                    filtered_plan.actions = allowed_actions;

                    let exec_results = executor.execute(&filtered_plan).await?;
                    result.adapted = exec_results.iter().any(|r| r.success);

                    // Emit executor health signal based on success rate
                    let success_rate = exec_results.iter().filter(|r| r.success).count() as f64
                        / exec_results.len().max(1) as f64;
                    mode_manager
                        .update_health(HealthSignal::new(ComponentType::Executor, success_rate))
                        .await?;
                } else {
                    tracing::info!(
                        mode = ?execution_mode,
                        total_actions = plan.actions.len(),
                        "All actions rejected by mode policy"
                    );
                }
            }
        } else {
            // Planner could not generate a plan - degrade health
            mode_manager
                .update_health(HealthSignal::new(ComponentType::Planner, 0.5))
                .await?;
        }

        Ok(result)
    }

    /// Stop MAPE-K loop
    pub async fn stop_loop(&self) -> WorkflowResult<()> {
        let mut state = self.state.write().await;
        *state = ControllerState::Stopping;
        drop(state);

        // Stop monitor
        self.monitor.stop().await?;

        let mut state = self.state.write().await;
        *state = ControllerState::Stopped;

        Ok(())
    }

    /// Get cycle statistics
    pub async fn get_stats(&self) -> CycleStats {
        let stats = self.stats.read().await;
        stats.clone()
    }
}

impl AutonomicManager for MapeKController {
    fn properties(&self) -> Vec<AutonomicProperty> {
        let mut props = Vec::new();

        if self.config.self_configuration {
            props.push(AutonomicProperty::SelfConfiguration);
        }
        if self.config.self_healing {
            props.push(AutonomicProperty::SelfHealing);
        }
        if self.config.self_optimization {
            props.push(AutonomicProperty::SelfOptimization);
        }
        if self.config.self_protection {
            props.push(AutonomicProperty::SelfProtection);
        }

        props
    }

    fn start(&mut self) -> WorkflowResult<()> {
        futures::executor::block_on(self.start_loop())
    }

    fn stop(&mut self) -> WorkflowResult<()> {
        futures::executor::block_on(self.stop_loop())
    }

    fn stats(&self) -> CycleStats {
        futures::executor::block_on(self.get_stats())
    }
}

/// MAPE-K cycle result
#[derive(Debug, Clone)]
struct CycleResult {
    anomalies: usize,
    plan_generated: bool,
    actions_executed: usize,
    actions_rejected: usize,
    adapted: bool,
    mode: AutonomicMode,
}

impl Default for CycleResult {
    fn default() -> Self {
        Self {
            anomalies: 0,
            plan_generated: false,
            actions_executed: 0,
            actions_rejected: 0,
            adapted: false,
            mode: AutonomicMode::Normal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::autonomic::knowledge::{Fact, Goal, GoalType};
    use crate::autonomic::monitor::WorkflowMetricsCollector;

    #[tokio::test]
    async fn test_mape_k_controller() {
        let config = ControllerConfig::default();
        let knowledge = Arc::new(KnowledgeBase::new());

        // Add goal
        let goal = Goal::new(
            "latency".to_string(),
            GoalType::Performance,
            "avg_latency_ms".to_string(),
            100.0,
        );
        knowledge.add_goal(goal).await.unwrap();

        // Create monitor with collector
        let mut monitor = Monitor::new(knowledge.clone());
        let collector = Arc::new(WorkflowMetricsCollector::new());
        monitor.add_collector(collector.clone());

        // Create controller
        let controller = MapeKController::new(config, monitor);

        // Add violating fact
        knowledge
            .add_fact(Fact::new(
                "avg_latency_ms".to_string(),
                150.0,
                "test".to_string(),
            ))
            .await
            .unwrap();

        // Start loop
        controller.start_loop().await.unwrap();

        // Wait for at least one cycle
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Stop loop
        controller.stop_loop().await.unwrap();

        // Check stats
        let stats = controller.get_stats().await;
        assert!(stats.total_cycles > 0 || stats.anomalies_detected > 0);
    }
}
