//! Adaptive strategy - learns from success/failure to optimize change frequency

use crate::{LoopConfig, LoopState, Result};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument};

/// Learns from success/failure patterns to adapt the loop's behavior
///
/// This system:
/// 1. Tracks recent cycle outcomes
/// 2. Calculates success rates
/// 3. Adjusts cycle interval based on performance
/// 4. Optimizes change frequency over time
pub struct AdaptiveStrategy {
    /// Reference to loop state
    state: Arc<RwLock<LoopState>>,

    /// History of recent cycles
    history: Arc<RwLock<StrategyHistory>>,
}

/// History of cycle outcomes
#[derive(Debug, Clone)]
pub struct StrategyHistory {
    /// Recent cycle outcomes (bounded)
    pub recent_cycles: VecDeque<CycleOutcome>,

    /// Maximum history size
    pub max_history: usize,
}

impl Default for StrategyHistory {
    fn default() -> Self {
        Self {
            recent_cycles: VecDeque::new(),
            max_history: 100,
        }
    }
}

/// Outcome of a single cycle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CycleOutcome {
    /// Cycle succeeded completely
    Success {
        proposals: usize,
        promoted: usize,
    },

    /// Cycle partially succeeded
    PartialSuccess {
        proposals: usize,
        promoted: usize,
        failed: usize,
    },

    /// Cycle failed
    Failure { reason: String },
}

impl AdaptiveStrategy {
    /// Create a new adaptive strategy
    pub fn new(state: Arc<RwLock<LoopState>>) -> Self {
        Self {
            state,
            history: Arc::new(RwLock::new(StrategyHistory::default())),
        }
    }

    /// Record a cycle outcome
    #[instrument(skip(self, outcome))]
    pub async fn record_outcome(&self, outcome: CycleOutcome) -> Result<()> {
        let mut history = self.history.write().await;

        history.recent_cycles.push_back(outcome);

        // Maintain bounded history
        while history.recent_cycles.len() > history.max_history {
            history.recent_cycles.pop_front();
        }

        debug!(
            history_size = history.recent_cycles.len(),
            "Recorded cycle outcome"
        );

        Ok(())
    }

    /// Adjust cycle interval based on recent success rate
    #[instrument(skip(self, config))]
    pub async fn adjust_interval(&self, config: &mut LoopConfig) -> Result<()> {
        let history = self.history.read().await;

        if history.recent_cycles.len() < 5 {
            // Not enough data yet
            return Ok(());
        }

        // Calculate success rate from last 10 cycles
        let window_size = 10.min(history.recent_cycles.len());
        let recent = history
            .recent_cycles
            .iter()
            .rev()
            .take(window_size)
            .collect::<Vec<_>>();

        let success_count = recent
            .iter()
            .filter(|outcome| matches!(outcome, CycleOutcome::Success { .. }))
            .count();

        let success_rate = success_count as f64 / window_size as f64;

        debug!(
            success_rate,
            window_size, "Calculated success rate"
        );

        // Adjust interval based on success rate
        let old_interval = config.cycle_interval;

        if success_rate > 0.8 {
            // High success rate - speed up (but not below 10 seconds)
            let reduction = Duration::from_millis(500);
            config.cycle_interval = config
                .cycle_interval
                .saturating_sub(reduction)
                .max(Duration::from_secs(10));

            info!(
                success_rate,
                old_interval_ms = old_interval.as_millis(),
                new_interval_ms = config.cycle_interval.as_millis(),
                "Speeding up cycle (high success rate)"
            );
        } else if success_rate < 0.5 {
            // Low success rate - slow down
            let increase = Duration::from_secs(5);
            config.cycle_interval = config
                .cycle_interval
                .saturating_add(increase)
                .min(Duration::from_secs(600)); // Max 10 minutes

            info!(
                success_rate,
                old_interval_ms = old_interval.as_millis(),
                new_interval_ms = config.cycle_interval.as_millis(),
                "Slowing down cycle (low success rate)"
            );
        }

        Ok(())
    }

    /// Calculate current success rate
    #[instrument(skip(self))]
    pub async fn success_rate(&self) -> Result<f64> {
        let history = self.history.read().await;

        if history.recent_cycles.is_empty() {
            return Ok(0.0);
        }

        let total = history.recent_cycles.len();
        let successful = history
            .recent_cycles
            .iter()
            .filter(|outcome| matches!(outcome, CycleOutcome::Success { .. }))
            .count();

        Ok(successful as f64 / total as f64)
    }

    /// Calculate average proposals per cycle
    pub async fn average_proposals(&self) -> Result<f64> {
        let history = self.history.read().await;

        if history.recent_cycles.is_empty() {
            return Ok(0.0);
        }

        let total_proposals: usize = history
            .recent_cycles
            .iter()
            .map(|outcome| match outcome {
                CycleOutcome::Success { proposals, .. } => *proposals,
                CycleOutcome::PartialSuccess { proposals, .. } => *proposals,
                CycleOutcome::Failure { .. } => 0,
            })
            .sum();

        Ok(total_proposals as f64 / history.recent_cycles.len() as f64)
    }

    /// Calculate average promotions per cycle
    pub async fn average_promotions(&self) -> Result<f64> {
        let history = self.history.read().await;

        if history.recent_cycles.is_empty() {
            return Ok(0.0);
        }

        let total_promoted: usize = history
            .recent_cycles
            .iter()
            .map(|outcome| match outcome {
                CycleOutcome::Success { promoted, .. } => *promoted,
                CycleOutcome::PartialSuccess { promoted, .. } => *promoted,
                CycleOutcome::Failure { .. } => 0,
            })
            .sum();

        Ok(total_promoted as f64 / history.recent_cycles.len() as f64)
    }

    /// Get recent cycle outcomes (for monitoring)
    pub async fn recent_outcomes(&self, limit: usize) -> Vec<CycleOutcome> {
        let history = self.history.read().await;
        history
            .recent_cycles
            .iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// Clear all history
    #[instrument(skip(self))]
    pub async fn clear_history(&self) -> Result<()> {
        let mut history = self.history.write().await;
        history.recent_cycles.clear();
        info!("Cleared strategy history");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_record_outcome() {
        let state = Arc::new(RwLock::new(LoopState::default()));
        let strategy = AdaptiveStrategy::new(state);

        strategy
            .record_outcome(CycleOutcome::Success {
                proposals: 5,
                promoted: 3,
            })
            .await
            .unwrap();

        let history = strategy.history.read().await;
        assert_eq!(history.recent_cycles.len(), 1);
    }

    #[tokio::test]
    async fn test_bounded_history() {
        let state = Arc::new(RwLock::new(LoopState::default()));
        let strategy = AdaptiveStrategy::new(state);

        // Record more than max_history
        for i in 0..150 {
            strategy
                .record_outcome(CycleOutcome::Success {
                    proposals: i,
                    promoted: i,
                })
                .await
                .unwrap();
        }

        let history = strategy.history.read().await;
        assert_eq!(history.recent_cycles.len(), history.max_history);
    }

    #[tokio::test]
    async fn test_success_rate() {
        let state = Arc::new(RwLock::new(LoopState::default()));
        let strategy = AdaptiveStrategy::new(state);

        // 8 successes, 2 failures
        for _ in 0..8 {
            strategy
                .record_outcome(CycleOutcome::Success {
                    proposals: 5,
                    promoted: 3,
                })
                .await
                .unwrap();
        }
        for _ in 0..2 {
            strategy
                .record_outcome(CycleOutcome::Failure {
                    reason: "test".to_string(),
                })
                .await
                .unwrap();
        }

        let rate = strategy.success_rate().await.unwrap();
        assert!((rate - 0.8).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_adjust_interval_speedup() {
        let state = Arc::new(RwLock::new(LoopState::default()));
        let strategy = AdaptiveStrategy::new(state);

        // Record high success rate
        for _ in 0..10 {
            strategy
                .record_outcome(CycleOutcome::Success {
                    proposals: 5,
                    promoted: 3,
                })
                .await
                .unwrap();
        }

        let mut config = LoopConfig {
            cycle_interval: Duration::from_secs(60),
            ..Default::default()
        };

        let old_interval = config.cycle_interval;
        strategy.adjust_interval(&mut config).await.unwrap();

        // Should have decreased
        assert!(config.cycle_interval < old_interval);
    }

    #[tokio::test]
    async fn test_adjust_interval_slowdown() {
        let state = Arc::new(RwLock::new(LoopState::default()));
        let strategy = AdaptiveStrategy::new(state);

        // Record low success rate
        for _ in 0..10 {
            strategy
                .record_outcome(CycleOutcome::Failure {
                    reason: "test".to_string(),
                })
                .await
                .unwrap();
        }

        let mut config = LoopConfig {
            cycle_interval: Duration::from_secs(60),
            ..Default::default()
        };

        let old_interval = config.cycle_interval;
        strategy.adjust_interval(&mut config).await.unwrap();

        // Should have increased
        assert!(config.cycle_interval > old_interval);
    }

    #[tokio::test]
    async fn test_average_calculations() {
        let state = Arc::new(RwLock::new(LoopState::default()));
        let strategy = AdaptiveStrategy::new(state);

        strategy
            .record_outcome(CycleOutcome::Success {
                proposals: 10,
                promoted: 5,
            })
            .await
            .unwrap();
        strategy
            .record_outcome(CycleOutcome::Success {
                proposals: 20,
                promoted: 15,
            })
            .await
            .unwrap();

        let avg_proposals = strategy.average_proposals().await.unwrap();
        assert!((avg_proposals - 15.0).abs() < 0.01);

        let avg_promotions = strategy.average_promotions().await.unwrap();
        assert!((avg_promotions - 10.0).abs() < 0.01);
    }
}
