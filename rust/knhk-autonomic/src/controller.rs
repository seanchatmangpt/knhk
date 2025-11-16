//! # Autonomic Controller - MAPE-K Orchestration
//!
//! **Covenant 3**: Feedback loops run at machine speed
//!
//! The Autonomic Controller orchestrates the complete MAPE-K feedback loop.
//! It coordinates Monitor, Analyze, Plan, Execute, and Knowledge components
//! to create a self-managing system.
//!
//! ## Workflow
//!
//! 1. **Monitor**: Collect metrics and detect anomalies
//! 2. **Analyze**: Match patterns and identify root causes
//! 3. **Plan**: Evaluate policies and select actions
//! 4. **Execute**: Run actions and capture feedback
//! 5. **Knowledge**: Learn from results and update patterns
//!
//! ## Example
//!
//! ```rust,no_run
//! use knhk_autonomic::{AutonomicController, Config};
//!
//! # async fn example() -> anyhow::Result<()> {
//! let config = Config::default()
//!     .with_loop_frequency(std::time::Duration::from_secs(5));
//!
//! let mut controller = AutonomicController::new(config).await?;
//!
//! // Start MAPE-K loop
//! controller.start().await?;
//! # Ok(())
//! # }
//! ```

use crate::{
    monitor::MonitoringComponent,
    analyze::AnalysisComponent,
    planner::PlanningComponent,
    execute::ExecutionComponent,
    knowledge::KnowledgeBase,
    hooks::{HookRegistry, HookType, HookContext},
    types::{Config, FeedbackCycle},
    error::{AutonomicError, Result},
};
use chrono::Utc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{instrument, info, debug, warn, error};

/// Autonomic controller orchestrating MAPE-K loop
pub struct AutonomicController {
    /// Configuration
    config: Config,

    /// Monitor component
    monitor: Arc<RwLock<MonitoringComponent>>,

    /// Analyze component
    analyzer: Arc<RwLock<AnalysisComponent>>,

    /// Planning component
    planner: Arc<RwLock<PlanningComponent>>,

    /// Execution component
    executor: Arc<RwLock<ExecutionComponent>>,

    /// Knowledge base
    knowledge: Arc<RwLock<KnowledgeBase>>,

    /// Hook registry
    hooks: Arc<RwLock<HookRegistry>>,

    /// Whether loop is running
    running: Arc<AtomicBool>,

    /// Current cycle number
    cycle_number: Arc<AtomicU64>,
}

impl AutonomicController {
    /// Create a new autonomic controller
    #[instrument(skip(config))]
    pub async fn new(config: Config) -> Result<Self> {
        info!("Initializing autonomic controller");

        let knowledge = KnowledgeBase::new(&config.knowledge_path).await?;

        let controller = Self {
            monitor: Arc::new(RwLock::new(MonitoringComponent::new())),
            analyzer: Arc::new(RwLock::new(AnalysisComponent::new())),
            planner: Arc::new(RwLock::new(PlanningComponent::new())),
            executor: Arc::new(RwLock::new(ExecutionComponent::new())),
            knowledge: Arc::new(RwLock::new(knowledge)),
            hooks: Arc::new(RwLock::new(HookRegistry::new())),
            running: Arc::new(AtomicBool::new(false)),
            cycle_number: Arc::new(AtomicU64::new(0)),
            config,
        };

        info!("Autonomic controller initialized");
        Ok(controller)
    }

    /// Start the MAPE-K feedback loop
    #[instrument(skip(self))]
    pub async fn start(&mut self) -> Result<()> {
        if self.running.swap(true, Ordering::SeqCst) {
            return Err(AutonomicError::Config("Controller already running".to_string()));
        }

        info!("Starting MAPE-K feedback loop (frequency: {:?})", self.config.loop_frequency);

        let mut ticker = interval(self.config.loop_frequency);

        loop {
            ticker.tick().await;

            if !self.running.load(Ordering::SeqCst) {
                break;
            }

            // Execute one MAPE-K cycle
            if let Err(e) = self.execute_cycle().await {
                error!("MAPE-K cycle failed: {}", e);
                // Continue running despite errors
            }
        }

        info!("MAPE-K feedback loop stopped");
        Ok(())
    }

    /// Stop the MAPE-K feedback loop
    pub async fn stop(&self) {
        info!("Stopping MAPE-K feedback loop");
        self.running.store(false, Ordering::SeqCst);
    }

    /// Execute one complete MAPE-K cycle
    #[instrument(skip(self))]
    async fn execute_cycle(&self) -> Result<()> {
        let cycle_num = self.cycle_number.fetch_add(1, Ordering::SeqCst) + 1;
        let start_time = Utc::now();

        debug!("Starting MAPE-K cycle {}", cycle_num);

        // === MONITOR PHASE ===
        let mut ctx = HookContext::new();
        ctx.set("cycle_number", cycle_num)?;

        {
            let hooks = self.hooks.read().await;
            hooks.execute(HookType::PreMonitor, &ctx).await?;
        }

        let metrics = {
            let monitor = self.monitor.read().await;
            monitor.collect_metrics().await?
        };

        let observations = {
            let monitor = self.monitor.read().await;
            monitor.detect_anomalies(&metrics).await?
        };

        ctx.set("metrics_count", metrics.len())?;
        ctx.set("observations_count", observations.len())?;

        {
            let hooks = self.hooks.read().await;
            hooks.execute(HookType::PostMonitor, &ctx).await?;
        }

        debug!("Monitor: {} metrics, {} observations", metrics.len(), observations.len());

        // === ANALYZE PHASE ===
        if observations.is_empty() {
            debug!("No anomalies detected, skipping analysis");
            return Ok(());
        }

        {
            let hooks = self.hooks.read().await;
            hooks.execute(HookType::PreAnalyze, &ctx).await?;
        }

        let analyses = {
            let analyzer = self.analyzer.read().await;
            analyzer.analyze(&observations, &metrics).await?
        };

        ctx.set("analyses_count", analyses.len())?;

        {
            let hooks = self.hooks.read().await;
            hooks.execute(HookType::PostAnalyze, &ctx).await?;
        }

        debug!("Analyze: {} analyses generated", analyses.len());

        if analyses.is_empty() {
            debug!("No problems identified, continuing monitoring");
            return Ok(());
        }

        // === PLAN PHASE ===
        {
            let hooks = self.hooks.read().await;
            hooks.execute(HookType::PrePlan, &ctx).await?;
        }

        let success_rates = {
            let kb = self.knowledge.read().await;
            kb.get_all_success_rates().await?
        };

        let mut plans = Vec::new();
        for analysis in &analyses {
            let planner = self.planner.read().await;
            if let Some(plan) = planner.create_plan(analysis, &success_rates).await? {
                plans.push(plan);
            }
        }

        ctx.set("plans_count", plans.len())?;

        {
            let hooks = self.hooks.read().await;
            hooks.execute(HookType::PostPlan, &ctx).await?;
        }

        debug!("Plan: {} plans created", plans.len());

        if plans.is_empty() {
            debug!("No viable plans, continuing monitoring");
            return Ok(());
        }

        // === EXECUTE PHASE ===
        {
            let hooks = self.hooks.read().await;
            hooks.execute(HookType::PreExecute, &ctx).await?;
        }

        let mut all_executions = Vec::new();
        for plan in &plans {
            // Get actions for plan
            let actions = {
                let planner = self.planner.read().await;
                let mut action_list = Vec::new();
                for action_id in &plan.actions {
                    if let Some(action) = planner.get_action(action_id).await? {
                        action_list.push(action);
                    }
                }
                action_list
            };

            let executions = {
                let mut executor = self.executor.write().await;
                executor.execute_plan(plan, &actions).await?
            };

            all_executions.extend(executions);
        }

        ctx.set("executions_count", all_executions.len())?;

        {
            let hooks = self.hooks.read().await;
            hooks.execute(HookType::PostExecute, &ctx).await?;
        }

        debug!("Execute: {} actions executed", all_executions.len());

        // === KNOWLEDGE/FEEDBACK PHASE ===
        {
            let hooks = self.hooks.read().await;
            hooks.execute(HookType::PreFeedback, &ctx).await?;
        }

        // Record feedback cycle
        let end_time = Utc::now();
        let effectiveness = self.calculate_effectiveness(&all_executions);

        let cycle = FeedbackCycle {
            cycle_number: cycle_num,
            start_time,
            end_time,
            observations,
            analysis: analyses.first().cloned(),
            plan: plans.first().cloned(),
            executions: all_executions.clone(),
            outcome: format!("Executed {} actions", all_executions.len()),
            effectiveness,
        };

        {
            let mut kb = self.knowledge.write().await;
            kb.record_cycle(cycle).await?;

            // Record successes/failures
            for execution in &all_executions {
                let success = execution.status == crate::types::ExecutionStatus::Successful;
                let situation = analyses.first()
                    .map(|a| a.problem.clone())
                    .unwrap_or_else(|| "Unknown".to_string());

                kb.record_success(situation, execution.action_id, success).await?;
            }
        }

        {
            let hooks = self.hooks.read().await;
            hooks.execute(HookType::PostFeedback, &ctx).await?;
        }

        debug!("Knowledge: Cycle {} recorded (effectiveness: {:.2})", cycle_num, effectiveness);

        Ok(())
    }

    /// Calculate effectiveness of a cycle
    fn calculate_effectiveness(&self, executions: &[crate::types::ActionExecution]) -> f64 {
        if executions.is_empty() {
            return 0.0;
        }

        let successes = executions.iter()
            .filter(|e| e.status == crate::types::ExecutionStatus::Successful)
            .count();

        successes as f64 / executions.len() as f64
    }

    /// Get access to monitor component
    pub fn monitor(&self) -> Arc<RwLock<MonitoringComponent>> {
        Arc::clone(&self.monitor)
    }

    /// Get access to analyzer component
    pub fn analyzer(&self) -> Arc<RwLock<AnalysisComponent>> {
        Arc::clone(&self.analyzer)
    }

    /// Get access to planner component
    pub fn planner(&self) -> Arc<RwLock<PlanningComponent>> {
        Arc::clone(&self.planner)
    }

    /// Get access to knowledge base
    pub fn knowledge(&self) -> Arc<RwLock<KnowledgeBase>> {
        Arc::clone(&self.knowledge)
    }

    /// Get access to hook registry
    pub fn hooks(&self) -> Arc<RwLock<HookRegistry>> {
        Arc::clone(&self.hooks)
    }
}
