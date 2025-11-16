//! Main orchestrator for the Autonomous Evolution Loop

use crate::config::AutonomousLoopConfig;
use crate::cycle::EvolutionCycle;
use crate::dependencies::LoopDependencies;
use crate::health::{HealthStats, LoopHealth};
use crate::telemetry::LoopTelemetry;
use crate::LegacyResult as Result;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

/// Main orchestrator for the autonomous evolution loop
pub struct LoopEngine {
    /// Configuration
    config: Arc<RwLock<AutonomousLoopConfig>>,

    /// Dependencies (snapshot store, pattern miner, etc.)
    dependencies: Arc<LoopDependencies>,

    /// Evolution history (last 100 cycles)
    history: Arc<RwLock<VecDeque<EvolutionCycle>>>,

    /// Health statistics
    stats: Arc<RwLock<HealthStats>>,

    /// Is loop currently running?
    running: Arc<AtomicBool>,

    /// Current health status
    health: Arc<RwLock<LoopHealth>>,

    /// Telemetry integration
    telemetry: Arc<LoopTelemetry>,

    /// Loop start time
    start_time: SystemTime,
}

impl LoopEngine {
    /// Create a new loop engine
    pub fn new(
        config: AutonomousLoopConfig,
        dependencies: LoopDependencies,
    ) -> Result<Self> {
        // Validate configuration
        config.validate()?;

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            dependencies: Arc::new(dependencies),
            history: Arc::new(RwLock::new(VecDeque::with_capacity(100))),
            stats: Arc::new(RwLock::new(HealthStats::default())),
            running: Arc::new(AtomicBool::new(false)),
            health: Arc::new(RwLock::new(LoopHealth::Running)),
            telemetry: Arc::new(LoopTelemetry::new(
                "knhk-autonomous-loop".to_string(),
            )),
            start_time: SystemTime::now(),
        })
    }

    /// Start the autonomous evolution loop
    ///
    /// This runs forever until stopped. Each cycle:
    /// 1. Checks health status
    /// 2. Executes evolution cycle
    /// 3. Updates statistics
    /// 4. Emits telemetry
    /// 5. Sleeps until next cycle
    pub async fn run(self: Arc<Self>) -> Result<()> {
        self.running.store(true, Ordering::SeqCst);

        tracing::info!("Starting autonomous evolution loop");

        // Set health to running
        *self.health.write().await = LoopHealth::Running;

        loop {
            // Check if we should stop
            if !self.running.load(Ordering::SeqCst) {
                tracing::info!("Autonomous loop stopped");
                *self.health.write().await = LoopHealth::Stopped;
                break;
            }

            // Check health status
            let health_status = self.health.read().await.clone();

            match health_status {
                LoopHealth::Paused { ref reason } => {
                    tracing::warn!(reason = %reason, "Loop is paused");
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
                LoopHealth::Stopped => {
                    tracing::info!("Loop is stopped");
                    break;
                }
                LoopHealth::Error {
                    ref error,
                    retry_count,
                    ..
                } => {
                    let config = self.config.read().await;

                    if retry_count >= config.max_retries {
                        tracing::error!(
                            error = %error,
                            retry_count,
                            "Max retries exceeded, pausing loop"
                        );

                        *self.health.write().await = LoopHealth::Paused {
                            reason: format!(
                                "Max retries exceeded: {}",
                                error
                            ),
                        };
                        continue;
                    }

                    tracing::warn!(
                        error = %error,
                        retry_count,
                        "Retrying after error"
                    );

                    tokio::time::sleep(config.retry_backoff).await;
                }
                LoopHealth::Running => {
                    // Continue normally
                }
            }

            // Get configuration
            let config = self.config.read().await.clone();

            // Execute evolution cycle
            match EvolutionCycle::execute(&config, &self.dependencies).await {
                Ok(cycle) => {
                    let duration_ms = cycle.duration_ms();

                    tracing::info!(
                        cycle_id = cycle.cycle_id,
                        duration_ms,
                        result = ?cycle.result,
                        "Cycle completed"
                    );

                    // Update statistics
                    self.stats
                        .write()
                        .await
                        .record_cycle_result(&cycle.result, duration_ms);

                    // Check if we should pause due to error rate
                    let stats = self.stats.read().await;
                    if stats.exceeds_error_threshold(config.pause_on_error_rate) {
                        tracing::error!(
                            error_rate = stats.error_rate,
                            threshold = ?config.pause_on_error_rate,
                            "Error rate threshold exceeded, pausing loop"
                        );

                        *self.health.write().await = LoopHealth::Paused {
                            reason: format!(
                                "Error rate {:.1}% exceeds threshold",
                                stats.error_rate
                            ),
                        };
                    } else {
                        *self.health.write().await = LoopHealth::Running;
                    }
                    drop(stats);

                    // Store in history
                    let mut history = self.history.write().await;
                    history.push_back(cycle.clone());
                    if history.len() > 100 {
                        history.pop_front();
                    }
                    drop(history);

                    // Emit telemetry
                    if let Err(e) = self.telemetry.emit_cycle_complete(&cycle).await {
                        tracing::warn!(error = %e, "Failed to emit telemetry");
                    }
                }
                Err(e) => {
                    tracing::error!(error = %e, "Cycle execution failed");

                    // Update health with error
                    let mut health = self.health.write().await;
                    let retry_count = match &*health {
                        LoopHealth::Error { retry_count, .. } => retry_count + 1,
                        _ => 1,
                    };

                    *health = LoopHealth::Error {
                        error: e.to_string(),
                        retry_count,
                        last_error_time: Some(SystemTime::now()),
                    };
                    drop(health);

                    // Emit error telemetry
                    if let Err(e) = self.telemetry.emit_cycle_error(&e).await {
                        tracing::warn!(error = %e, "Failed to emit error telemetry");
                    }
                }
            }

            // Update uptime
            if let Ok(elapsed) = self.start_time.elapsed() {
                self.stats.write().await.uptime_seconds = elapsed.as_secs();
            }

            // Wait for next cycle
            tokio::time::sleep(config.cycle_interval).await;
        }

        tracing::info!("Autonomous evolution loop exited");
        Ok(())
    }

    /// Stop the loop gracefully
    pub fn stop(&self) {
        tracing::info!("Stopping autonomous loop");
        self.running.store(false, Ordering::SeqCst);
    }

    /// Pause the loop
    pub async fn pause(&self, reason: String) {
        tracing::warn!(reason = %reason, "Pausing autonomous loop");
        *self.health.write().await = LoopHealth::Paused { reason };
    }

    /// Resume the loop
    pub async fn resume(&self) {
        tracing::info!("Resuming autonomous loop");
        *self.health.write().await = LoopHealth::Running;
    }

    /// Get cycle history
    pub async fn get_history(&self) -> Vec<EvolutionCycle> {
        self.history.read().await.iter().cloned().collect()
    }

    /// Get current health status
    pub async fn get_health(&self) -> LoopHealth {
        self.health.read().await.clone()
    }

    /// Get health statistics
    pub async fn get_stats(&self) -> HealthStats {
        self.stats.read().await.clone()
    }

    /// Get current configuration
    pub async fn get_config(&self) -> AutonomousLoopConfig {
        self.config.read().await.clone()
    }

    /// Update configuration (takes effect on next cycle)
    pub async fn update_config(&self, new_config: AutonomousLoopConfig) -> Result<()> {
        new_config.validate()?;
        *self.config.write().await = new_config;
        tracing::info!("Configuration updated");
        Ok(())
    }

    /// Check if loop is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

/// Handle to a running loop
pub struct LoopHandle {
    engine: Arc<LoopEngine>,
    task: JoinHandle<Result<()>>,
}

impl LoopHandle {
    /// Create new loop handle
    pub fn new(engine: Arc<LoopEngine>, task: JoinHandle<Result<()>>) -> Self {
        Self { engine, task }
    }

    /// Stop the loop and wait for completion
    pub async fn stop(self) -> Result<()> {
        self.engine.stop();
        self.task.await.map_err(|e| {
            crate::EvolutionError::Unknown(format!("Task join error: {}", e))
        })?
    }

    /// Get reference to engine
    pub fn engine(&self) -> &Arc<LoopEngine> {
        &self.engine
    }
}

/// Start the autonomous loop in a background task
pub fn start_autonomous_loop(
    config: AutonomousLoopConfig,
    dependencies: LoopDependencies,
) -> Result<LoopHandle> {
    let engine = Arc::new(LoopEngine::new(config, dependencies)?);
    let engine_clone = Arc::clone(&engine);

    let task = tokio::spawn(async move { engine_clone.run().await });

    Ok(LoopHandle::new(engine, task))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loop_engine_creation() {
        // This test requires mock dependencies
        // In practice, you would use the mock implementations
    }

    #[tokio::test]
    async fn test_stop_loop() {
        // Similar to above - requires mocks
    }
}
