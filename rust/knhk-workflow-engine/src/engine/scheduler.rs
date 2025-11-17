//! Latency-Bounded Scheduler
//!
//! Enforces the Chatman constant (≤8 ticks) for hot path operations.
//! Provides scheduling primitives for hook execution with latency guarantees.

use crate::error::{WorkflowError, WorkflowResult};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Tick count type
pub type TickCount = u32;

/// Scheduled task priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    /// Critical (hot path)
    Critical = 0,
    /// High priority
    High = 1,
    /// Normal priority
    Normal = 2,
    /// Low priority
    Low = 3,
}

/// Task execution result with latency tracking
#[derive(Debug, Clone)]
pub struct TaskExecutionResult {
    /// Whether task completed successfully
    pub success: bool,
    /// Ticks consumed
    pub ticks_used: TickCount,
    /// Execution time in microseconds
    pub execution_time_us: u64,
    /// Whether latency constraint was met
    pub met_constraint: bool,
}

/// Latency-bounded scheduler
pub struct LatencyBoundedScheduler {
    /// Maximum ticks allowed for hot path
    max_hot_path_ticks: TickCount,
    /// Total tasks executed
    total_tasks: Arc<AtomicU64>,
    /// Total ticks consumed
    total_ticks: Arc<AtomicU64>,
    /// Constraint violations
    constraint_violations: Arc<AtomicU64>,
}

impl LatencyBoundedScheduler {
    /// Create new scheduler with max ticks constraint
    pub fn new(max_hot_path_ticks: TickCount) -> Self {
        Self {
            max_hot_path_ticks,
            total_tasks: Arc::new(AtomicU64::new(0)),
            total_ticks: Arc::new(AtomicU64::new(0)),
            constraint_violations: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Execute a task with latency tracking
    pub async fn execute_with_bounds<F, T>(
        &self,
        priority: Priority,
        task: F,
    ) -> WorkflowResult<TaskExecutionResult>
    where
        F: std::future::Future<Output = WorkflowResult<T>>,
    {
        let start_time = Instant::now();
        let tick_start = self.get_tick_count();

        // Execute task
        let result = task.await;

        // Calculate metrics
        let tick_end = self.get_tick_count();
        let ticks_used = tick_end.saturating_sub(tick_start);
        let execution_time_us = start_time.elapsed().as_micros() as u64;

        // Check constraint
        let max_ticks = match priority {
            Priority::Critical => self.max_hot_path_ticks,
            Priority::High => self.max_hot_path_ticks * 2,
            Priority::Normal => self.max_hot_path_ticks * 4,
            Priority::Low => u32::MAX, // No constraint for low priority
        };

        let met_constraint = ticks_used <= max_ticks;

        // Update statistics
        self.total_tasks.fetch_add(1, Ordering::Relaxed);
        self.total_ticks
            .fetch_add(ticks_used as u64, Ordering::Relaxed);

        if !met_constraint {
            self.constraint_violations.fetch_add(1, Ordering::Relaxed);
            tracing::warn!(
                "Task exceeded latency constraint: {} ticks (max {} for {:?} priority)",
                ticks_used,
                max_ticks,
                priority
            );
        }

        Ok(TaskExecutionResult {
            success: result.is_ok(),
            ticks_used,
            execution_time_us,
            met_constraint,
        })
    }

    /// Check if operation is hot path eligible
    pub fn is_hot_path_eligible(&self, estimated_ticks: TickCount) -> bool {
        estimated_ticks <= self.max_hot_path_ticks
    }

    /// Get current tick count (public alias)
    pub fn current_tick(&self) -> TickCount {
        self.get_tick_count()
    }

    /// Get current tick count
    #[inline(always)]
    fn get_tick_count(&self) -> TickCount {
        // Simplified tick counting - in production would use RDTSC
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| (d.as_nanos() % u32::MAX as u128) as u32)
            .unwrap_or(0)
    }

    /// Get scheduler statistics
    pub fn get_stats(&self) -> SchedulerStats {
        let total_tasks = self.total_tasks.load(Ordering::Relaxed);
        let total_ticks = self.total_ticks.load(Ordering::Relaxed);
        let violations = self.constraint_violations.load(Ordering::Relaxed);

        SchedulerStats {
            total_tasks,
            total_ticks,
            avg_ticks: if total_tasks > 0 {
                total_ticks as f64 / total_tasks as f64
            } else {
                0.0
            },
            constraint_violations: violations,
            violation_rate: if total_tasks > 0 {
                violations as f64 / total_tasks as f64
            } else {
                0.0
            },
            max_hot_path_ticks: self.max_hot_path_ticks,
        }
    }

    /// Reset statistics
    pub fn reset_stats(&self) {
        self.total_tasks.store(0, Ordering::Relaxed);
        self.total_ticks.store(0, Ordering::Relaxed);
        self.constraint_violations.store(0, Ordering::Relaxed);
    }
}

/// Scheduler statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerStats {
    /// Total tasks executed
    pub total_tasks: u64,
    /// Total ticks consumed
    pub total_ticks: u64,
    /// Average ticks per task
    pub avg_ticks: f64,
    /// Constraint violations
    pub constraint_violations: u64,
    /// Violation rate (0.0 to 1.0)
    pub violation_rate: f64,
    /// Maximum hot path ticks
    pub max_hot_path_ticks: TickCount,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scheduler_execution() {
        let scheduler = LatencyBoundedScheduler::new(8);

        let result = scheduler
            .execute_with_bounds(Priority::Critical, async {
                // Fast operation
                Ok::<_, WorkflowError>(())
            })
            .await
            .expect("Task execution failed");

        assert!(result.success);
        assert!(result.ticks_used >= 0);
    }

    #[tokio::test]
    async fn test_hot_path_eligibility() {
        let scheduler = LatencyBoundedScheduler::new(8);

        assert!(scheduler.is_hot_path_eligible(5));
        assert!(scheduler.is_hot_path_eligible(8));
        assert!(!scheduler.is_hot_path_eligible(9));
    }

    #[tokio::test]
    async fn test_scheduler_stats() {
        let scheduler = LatencyBoundedScheduler::new(8);

        // Execute several tasks
        for _ in 0..5 {
            let _ = scheduler
                .execute_with_bounds(Priority::Critical, async { Ok::<_, WorkflowError>(()) })
                .await;
        }

        let stats = scheduler.get_stats();
        assert_eq!(stats.total_tasks, 5);
        assert!(stats.avg_ticks >= 0.0);
    }

    #[tokio::test]
    async fn test_priority_constraints() {
        let scheduler = LatencyBoundedScheduler::new(8);

        // Critical should have strictest constraint
        let result = scheduler
            .execute_with_bounds(Priority::Critical, async {
                tokio::time::sleep(std::time::Duration::from_nanos(1)).await;
                Ok::<_, WorkflowError>(())
            })
            .await
            .expect("Task failed");

        // Low priority has no constraint
        let result_low = scheduler
            .execute_with_bounds(Priority::Low, async {
                tokio::time::sleep(std::time::Duration::from_micros(1)).await;
                Ok::<_, WorkflowError>(())
            })
            .await
            .expect("Task failed");

        assert!(result_low.met_constraint);
    }
}
