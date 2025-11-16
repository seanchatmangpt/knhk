//! Deterministic Multi-Core Scheduler
//!
//! Implements type-safe, deterministic scheduling with resource contracts.

use core::marker::PhantomData;
use alloc::vec::Vec;
use crate::timing::{TickBudget, TickCounter};
use crate::isa::{GuardContext, TaskResult, MuInstruction};
use crate::sigma::SigmaPointer;
use crate::concurrency::types::{CoreLocal, GuardSet, NoGuards};
use crate::concurrency::queues::{WorkQueue, GlobalOrdered, QueueError};
use crate::concurrency::logical_time::{LogicalClock, Timestamp};
use crate::concurrency::replay::{ReplayLog, ReplayEvent};

/// Scheduler errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulerError {
    /// Queue error
    Queue(QueueError),
    /// Tick budget exceeded
    TickBudgetExceeded,
    /// Guard validation failed
    GuardValidationFailed,
    /// No work available
    NoWork,
    /// Invalid core ID
    InvalidCore,
}

impl From<QueueError> for SchedulerError {
    fn from(e: QueueError) -> Self {
        SchedulerError::Queue(e)
    }
}

/// Priority trait (compile-time priority levels)
pub trait Priority {
    const LEVEL: u8;
}

/// High priority
#[derive(Debug, Clone, Copy)]
pub struct PriorityHigh;

impl Priority for PriorityHigh {
    const LEVEL: u8 = 0;
}

/// Normal priority
#[derive(Debug, Clone, Copy)]
pub struct PriorityNormal;

impl Priority for PriorityNormal {
    const LEVEL: u8 = 1;
}

/// Low priority
#[derive(Debug, Clone, Copy)]
pub struct PriorityLow;

impl Priority for PriorityLow {
    const LEVEL: u8 = 2;
}

/// Schedulable task with typed resource contracts
///
/// # Type Parameters
///
/// - `B`: TickBudget limit (compile-time constraint)
/// - `P`: Priority level (compile-time priority)
/// - `G`: Guard set (compile-time guard requirements)
///
/// # Usage
///
/// ```rust,no_run
/// use knhk_mu_kernel::concurrency::*;
/// use knhk_mu_kernel::timing::TickBudget;
///
/// let task = SchedulableTask::<_, PriorityHigh, NoGuards>::new(
///     task_id,
///     TickBudget::chatman(),
///     PriorityHigh,
///     NoGuards,
///     TaskWork::Observation(ctx),
/// );
/// ```
pub struct SchedulableTask<B, P, G>
where
    P: Priority,
    G: GuardSet,
{
    /// Task ID
    pub task_id: u64,
    /// Tick budget
    pub tick_budget: TickBudget,
    /// Priority (phantom, enforced by type system)
    _priority: PhantomData<P>,
    /// Guards (enforced by type system)
    pub guards: G,
    /// Task work
    pub work: TaskWork,
    /// Sigma snapshot reference
    pub sigma: &'static SigmaPointer,
}

impl<B, P, G> SchedulableTask<B, P, G>
where
    P: Priority,
    G: GuardSet,
{
    /// Create new schedulable task
    pub fn new(
        task_id: u64,
        tick_budget: TickBudget,
        _priority: P,
        guards: G,
        work: TaskWork,
        sigma: &'static SigmaPointer,
    ) -> Result<Self, SchedulerError> {
        // Validate guards at construction time
        guards
            .validate()
            .map_err(|_| SchedulerError::GuardValidationFailed)?;

        Ok(Self {
            task_id,
            tick_budget,
            _priority: PhantomData,
            guards,
            work,
            sigma,
        })
    }

    /// Get priority level
    pub const fn priority(&self) -> u8 {
        P::LEVEL
    }

    /// Get guard count
    pub const fn guard_count(&self) -> usize {
        G::COUNT
    }
}

/// Task work to execute
#[derive(Debug, Clone, Copy)]
pub enum TaskWork {
    /// Execute with observation
    Observation(GuardContext),
    /// Pure computation (no I/O)
    Pure(u64),
}

/// Execution result
#[derive(Debug, Clone, Copy)]
pub struct ExecutionResult {
    /// Task ID
    pub task_id: u64,
    /// Task result
    pub result: TaskResult,
    /// Logical timestamp
    pub timestamp: Timestamp,
    /// Core ID that executed
    pub core_id: u8,
}

/// Deterministic multi-core scheduler
///
/// # Type Parameters
///
/// - `CORES`: Number of cores (compile-time constant)
///
/// # Properties
///
/// 1. **Deterministic**: Same inputs → same outputs
/// 2. **Type-Safe**: Resource contracts enforced at compile time
/// 3. **Lock-Free**: No blocking operations on hot path
/// 4. **Replay-able**: Full execution can be replayed
///
/// # Architecture
///
/// ```text
/// Scheduler<4>:
///   core_queues[4]     - Per-core work queues (CoreLocal)
///   global_order       - Globally ordered decision queue
///   logical_clock      - Lamport clock for timestamps
///   replay_log         - Event log for determinism
/// ```
pub struct DeterministicScheduler<const CORES: usize> {
    /// Per-core work queues (lock-free, SPSC)
    core_queues: [CoreLocal<WorkQueue<ScheduledTask, 1024>>; CORES],
    /// Globally ordered queue (for decisions requiring total order)
    global_order: GlobalOrdered<ScheduledTask>,
    /// Logical clock (Lamport)
    logical_clock: LogicalClock,
    /// Replay log (for determinism)
    replay_log: ReplayLog,
    /// Sigma pointer (active ontology)
    sigma: &'static SigmaPointer,
}

/// Internal task representation (type-erased for storage)
#[derive(Clone, Copy)]
struct ScheduledTask {
    task_id: u64,
    priority: u8,
    guard_count: u8,
    tick_budget: TickBudget,
    work: TaskWork,
}

impl<const CORES: usize> DeterministicScheduler<CORES> {
    /// Create new deterministic scheduler
    ///
    /// # Constraints
    ///
    /// - CORES must be power of 2 (for efficient modulo)
    /// - CORES must be ≤ 256 (u8 core_id)
    pub fn new(sigma: &'static SigmaPointer) -> Self {
        assert!(CORES > 0 && CORES <= 256);
        assert!(CORES.is_power_of_two(), "CORES must be power of 2");

        // Create core queues
        let core_queues: [CoreLocal<WorkQueue<ScheduledTask, 1024>>; CORES] =
            core::array::from_fn(|_| WorkQueue::new());

        Self {
            core_queues,
            global_order: GlobalOrdered::new(),
            logical_clock: LogicalClock::new(),
            replay_log: ReplayLog::new(),
            sigma,
        }
    }

    /// Enqueue task (assign to core)
    ///
    /// Uses task_id to deterministically assign to core.
    pub fn enqueue<P, G>(
        &mut self,
        task: SchedulableTask<TickBudget, P, G>,
    ) -> Result<(), SchedulerError>
    where
        P: Priority,
        G: GuardSet,
    {
        // Deterministic core assignment (hash task_id)
        let core_id = (task.task_id as usize) & (CORES - 1);

        // Create internal representation
        let scheduled = ScheduledTask {
            task_id: task.task_id,
            priority: task.priority(),
            guard_count: task.guard_count() as u8,
            tick_budget: task.tick_budget,
            work: task.work,
        };

        // Enqueue to core-local queue
        self.core_queues[core_id].with_mut(|queue| {
            queue.enqueue(scheduled)
        })?;

        // Log enqueue event
        let timestamp = self.logical_clock.tick();
        self.replay_log.record(ReplayEvent::TaskEnqueued {
            task_id: task.task_id,
            core_id: core_id as u8,
            timestamp,
        });

        Ok(())
    }

    /// Enqueue to global ordered queue
    ///
    /// For tasks requiring total order across all cores.
    pub fn enqueue_ordered<P, G>(
        &mut self,
        task: SchedulableTask<TickBudget, P, G>,
        timestamp: Timestamp,
    ) -> Result<(), SchedulerError>
    where
        P: Priority,
        G: GuardSet,
    {
        let scheduled = ScheduledTask {
            task_id: task.task_id,
            priority: task.priority(),
            guard_count: task.guard_count() as u8,
            tick_budget: task.tick_budget,
            work: task.work,
        };

        // Enqueue with timestamp
        self.global_order.enqueue(timestamp, 0, scheduled)?;

        // Log event
        self.replay_log.record(ReplayEvent::TaskEnqueued {
            task_id: task.task_id,
            core_id: 255,  // Global queue marker
            timestamp,
        });

        Ok(())
    }

    /// Run scheduling cycle on specific core
    ///
    /// Executes one task from the core-local queue.
    pub fn run_cycle(&mut self, core_id: usize) -> Result<ExecutionResult, SchedulerError> {
        if core_id >= CORES {
            return Err(SchedulerError::InvalidCore);
        }

        // Try to dequeue from core-local queue
        let scheduled = self.core_queues[core_id]
            .with_mut(|queue| queue.dequeue())
            .map_err(|_| SchedulerError::NoWork)?;

        // Execute task
        self.execute_task(scheduled, core_id as u8)
    }

    /// Execute task (deterministic)
    fn execute_task(
        &mut self,
        task: ScheduledTask,
        core_id: u8,
    ) -> Result<ExecutionResult, SchedulerError> {
        // Start timing
        let mut tick_counter = TickCounter::new();
        tick_counter.start();

        // Create tick budget
        let mut budget = task.tick_budget;

        // Execute based on work type
        let result = match task.work {
            TaskWork::Observation(ctx) => {
                MuInstruction::eval_task(task.task_id, &ctx, &mut budget)
                    .map_err(|_| SchedulerError::TickBudgetExceeded)?
            }
            TaskWork::Pure(value) => {
                // Pure computation (deterministic)
                budget.consume(2);  // 2 ticks for pure computation

                TaskResult {
                    task_id: task.task_id,
                    output_hash: [value; 4],
                    ticks_used: budget.used,
                }
            }
        };

        // Get logical timestamp
        let timestamp = self.logical_clock.tick();

        // Measure actual ticks
        let actual_ticks = tick_counter.ticks();

        // Log execution
        self.replay_log.record(ReplayEvent::TaskExecuted {
            task_id: task.task_id,
            core_id,
            timestamp,
            ticks: actual_ticks,
            output_hash: result.output_hash,
        });

        Ok(ExecutionResult {
            task_id: task.task_id,
            result,
            timestamp,
            core_id,
        })
    }

    /// Get current logical time
    pub fn now(&self) -> Timestamp {
        self.logical_clock.now()
    }

    /// Get replay log (for deterministic replay)
    pub fn replay_log(&self) -> &ReplayLog {
        &self.replay_log
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sigma::SigmaPointer;

    #[test]
    fn test_scheduler_creation() {
        let sigma_ptr = Box::leak(Box::new(SigmaPointer::new()));
        let _scheduler = DeterministicScheduler::<4>::new(sigma_ptr);
        // Scheduler created successfully
    }

    #[test]
    fn test_task_creation() {
        let sigma_ptr = Box::leak(Box::new(SigmaPointer::new()));

        let task = SchedulableTask::new(
            1,
            TickBudget::chatman(),
            PriorityHigh,
            NoGuards,
            TaskWork::Pure(42),
            sigma_ptr,
        );

        assert!(task.is_ok());
        let task = task.unwrap();
        assert_eq!(task.task_id, 1);
        assert_eq!(task.priority(), 0);
        assert_eq!(task.guard_count(), 0);
    }

    #[test]
    fn test_task_enqueue_dequeue() {
        let sigma_ptr = Box::leak(Box::new(SigmaPointer::new()));
        let mut scheduler = DeterministicScheduler::<4>::new(sigma_ptr);

        let task = SchedulableTask::new(
            1,
            TickBudget::chatman(),
            PriorityNormal,
            NoGuards,
            TaskWork::Pure(42),
            sigma_ptr,
        )
        .unwrap();

        // Enqueue task
        assert!(scheduler.enqueue(task).is_ok());

        // Run cycle on core 1 (task_id=1 maps to core 1)
        let result = scheduler.run_cycle(1);
        assert!(result.is_ok());

        let exec_result = result.unwrap();
        assert_eq!(exec_result.task_id, 1);
        assert_eq!(exec_result.core_id, 1);
    }

    #[test]
    fn test_deterministic_timestamps() {
        let sigma_ptr = Box::leak(Box::new(SigmaPointer::new()));
        let mut scheduler = DeterministicScheduler::<2>::new(sigma_ptr);

        let t1 = scheduler.now();

        let task1 = SchedulableTask::new(
            1,
            TickBudget::chatman(),
            PriorityHigh,
            NoGuards,
            TaskWork::Pure(42),
            sigma_ptr,
        )
        .unwrap();

        scheduler.enqueue(task1).unwrap();

        let t2 = scheduler.now();

        // Timestamp should have increased
        assert!(t2 > t1);
    }

    #[test]
    fn test_global_ordered_queue() {
        let sigma_ptr = Box::leak(Box::new(SigmaPointer::new()));
        let mut scheduler = DeterministicScheduler::<4>::new(sigma_ptr);

        // Enqueue tasks with explicit timestamps (out of order)
        let task1 = SchedulableTask::new(
            1,
            TickBudget::chatman(),
            PriorityHigh,
            NoGuards,
            TaskWork::Pure(1),
            sigma_ptr,
        )
        .unwrap();

        let task2 = SchedulableTask::new(
            2,
            TickBudget::chatman(),
            PriorityHigh,
            NoGuards,
            TaskWork::Pure(2),
            sigma_ptr,
        )
        .unwrap();

        scheduler
            .enqueue_ordered(task1, Timestamp::from_raw(10))
            .unwrap();
        scheduler
            .enqueue_ordered(task2, Timestamp::from_raw(5))
            .unwrap();

        // Global queue should maintain timestamp order
        // (This would be tested with dequeue from global_order)
    }
}
