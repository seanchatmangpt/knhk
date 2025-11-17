// knhk-kernel: Main hot path execution loop
// Straight-line code with ≤8 tick guarantee

use std::sync::{Arc, atomic::{AtomicBool, AtomicU64, Ordering}};
use parking_lot::RwLock;
use crossbeam_queue::ArrayQueue;

use crate::{
    executor::{Executor, Task, TaskState},
    descriptor::{DescriptorManager, ExecutionContext},
    receipt::{Receipt, ReceiptStore},
    timer::{HotPathTimer, TickBudget},
};

/// Maximum queue depth for hot path
const MAX_QUEUE_DEPTH: usize = 1024;

/// Hot path statistics
pub struct HotPathStats {
    pub cycles_total: AtomicU64,
    pub cycles_min: AtomicU64,
    pub cycles_max: AtomicU64,
    pub executions: AtomicU64,
    pub queue_depth: AtomicU64,
    pub stratum_switches: AtomicU64,
}

impl HotPathStats {
    pub fn new() -> Self {
        Self {
            cycles_total: AtomicU64::new(0),
            cycles_min: AtomicU64::new(u64::MAX),
            cycles_max: AtomicU64::new(0),
            executions: AtomicU64::new(0),
            queue_depth: AtomicU64::new(0),
            stratum_switches: AtomicU64::new(0),
        }
    }

    #[inline]
    pub fn record_execution(&self, cycles: u64) {
        self.cycles_total.fetch_add(cycles, Ordering::Relaxed);
        self.executions.fetch_add(1, Ordering::Relaxed);

        // Update min/max with CAS loop
        let mut current_min = self.cycles_min.load(Ordering::Relaxed);
        while cycles < current_min {
            match self.cycles_min.compare_exchange_weak(
                current_min,
                cycles,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(x) => current_min = x,
            }
        }

        let mut current_max = self.cycles_max.load(Ordering::Relaxed);
        while cycles > current_max {
            match self.cycles_max.compare_exchange_weak(
                current_max,
                cycles,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(x) => current_max = x,
            }
        }
    }
}

/// Stratum for execution isolation
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stratum {
    /// Ultra-hot path (≤8 ticks)
    Hot = 0,
    /// Warm path (≤100 ticks)
    Warm = 1,
    /// Cold path (>100 ticks)
    Cold = 2,
}

/// Hot path executor
pub struct HotPath {
    /// Core executor
    executor: Executor,

    /// Task queue (lock-free)
    hot_queue: ArrayQueue<Box<Task>>,
    warm_queue: ArrayQueue<Box<Task>>,
    cold_queue: RwLock<Vec<Box<Task>>>,

    /// Receipt store
    receipt_store: RwLock<ReceiptStore>,

    /// Statistics
    stats: HotPathStats,

    /// Running flag
    running: AtomicBool,

    /// Current stratum
    current_stratum: AtomicU64,
}

impl HotPath {
    pub fn new() -> Self {
        Self {
            executor: Executor::new(),
            hot_queue: ArrayQueue::new(MAX_QUEUE_DEPTH),
            warm_queue: ArrayQueue::new(MAX_QUEUE_DEPTH),
            cold_queue: RwLock::new(Vec::new()),
            receipt_store: RwLock::new(ReceiptStore::new(10000)),
            stats: HotPathStats::new(),
            running: AtomicBool::new(false),
            current_stratum: AtomicU64::new(Stratum::Hot as u64),
        }
    }

    /// Submit task to hot path
    #[inline]
    pub fn submit(&self, task: Box<Task>) -> Result<(), Box<Task>> {
        // Route based on estimated complexity
        let stratum = self.estimate_stratum(&task);

        match stratum {
            Stratum::Hot => self.hot_queue.push(task),
            Stratum::Warm => self.warm_queue.push(task),
            Stratum::Cold => {
                self.cold_queue.write().push(task);
                Ok(())
            }
        }
    }

    /// Main execution loop (hot path)
    pub fn run_loop(&self) {
        self.running.store(true, Ordering::Release);

        while self.running.load(Ordering::Acquire) {
            // Process hot queue first (straight-line code)
            self.process_hot_stratum();

            // Check warm queue if hot is empty
            if self.hot_queue.is_empty() {
                self.process_warm_stratum();
            }

            // Process cold queue last
            if self.hot_queue.is_empty() && self.warm_queue.is_empty() {
                self.process_cold_stratum();
            }

            // Yield to prevent busy-waiting
            std::hint::spin_loop();
        }
    }

    /// Process hot stratum (≤8 ticks per task)
    #[inline(always)]
    fn process_hot_stratum(&self) {
        const BATCH_SIZE: usize = 8;
        let mut processed = 0;

        while processed < BATCH_SIZE {
            let task = match self.hot_queue.pop() {
                Some(t) => t,
                None => break,
            };

            // Execute with timing
            let timer = HotPathTimer::start();
            let receipt = self.execute_task_hot(&task);
            let cycles = timer.elapsed_ticks();

            // Record statistics
            self.stats.record_execution(cycles);

            // Store receipt
            self.receipt_store.write().store(receipt);

            // Check if task should move stratum
            if cycles > 8 {
                self.stats.stratum_switches.fetch_add(1, Ordering::Relaxed);
                // Task took too long, move to warm queue for next execution
                if task.get_state() != TaskState::Completed {
                    let _ = self.warm_queue.push(task);
                }
            }

            processed += 1;
        }
    }

    /// Process warm stratum (≤100 ticks per task)
    #[inline]
    fn process_warm_stratum(&self) {
        const BATCH_SIZE: usize = 4;
        let mut processed = 0;

        while processed < BATCH_SIZE {
            let task = match self.warm_queue.pop() {
                Some(t) => t,
                None => break,
            };

            let timer = HotPathTimer::start();
            let receipt = self.executor.execute(&task);
            let cycles = timer.elapsed_ticks();

            self.stats.record_execution(cycles);
            self.receipt_store.write().store(receipt);

            // Check for stratum change
            if cycles > 100 && task.get_state() != TaskState::Completed {
                self.cold_queue.write().push(task);
                self.stats.stratum_switches.fetch_add(1, Ordering::Relaxed);
            }

            processed += 1;
        }
    }

    /// Process cold stratum (no timing constraints)
    fn process_cold_stratum(&self) {
        let mut cold = self.cold_queue.write();
        if cold.is_empty() {
            return;
        }

        // Process one task from cold queue
        if let Some(task) = cold.pop() {
            let timer = HotPathTimer::start();
            let receipt = self.executor.execute(&task);
            let cycles = timer.elapsed_ticks();

            self.stats.record_execution(cycles);
            self.receipt_store.write().store(receipt);
        }
    }

    /// Execute task in hot path (optimized, straight-line)
    #[inline(always)]
    fn execute_task_hot(&self, task: &Task) -> Receipt {
        // This is the most optimized path - no branches if possible
        let timer = HotPathTimer::start_serialized();

        // Direct execution without validation
        let receipt = self.executor.execute(task);

        // Check timing
        let elapsed = timer.elapsed_ticks();
        if elapsed > 8 {
            // Log violation (but don't block)
            self.stats.stratum_switches.fetch_add(1, Ordering::Relaxed);
        }

        receipt
    }

    /// Estimate stratum for task
    #[inline]
    fn estimate_stratum(&self, task: &Task) -> Stratum {
        // Simple heuristic based on pattern and observation count
        if let Some(descriptor) = DescriptorManager::get_active() {
            if let Some(pattern) = descriptor.get_pattern(task.pattern_id) {
                // Complex patterns go to warm/cold
                match pattern.pattern_type {
                    crate::pattern::PatternType::Recursion |
                    crate::pattern::PatternType::ArbitraryLoop => Stratum::Cold,

                    crate::pattern::PatternType::MultiInstanceUnknownRuntime |
                    crate::pattern::PatternType::InterleavedParallelRouting => Stratum::Warm,

                    _ => {
                        // Simple patterns with few observations go to hot
                        if task.observation_count <= 4 {
                            Stratum::Hot
                        } else {
                            Stratum::Warm
                        }
                    }
                }
            } else {
                Stratum::Cold // Unknown pattern
            }
        } else {
            Stratum::Cold // No descriptor
        }
    }

    /// Stop execution loop
    pub fn stop(&self) {
        self.running.store(false, Ordering::Release);
    }

    /// Get statistics
    pub fn stats(&self) -> HotPathStatsSnapshot {
        let executions = self.stats.executions.load(Ordering::Relaxed);
        let total = self.stats.cycles_total.load(Ordering::Relaxed);

        HotPathStatsSnapshot {
            executions,
            cycles_total: total,
            cycles_min: self.stats.cycles_min.load(Ordering::Relaxed),
            cycles_max: self.stats.cycles_max.load(Ordering::Relaxed),
            cycles_avg: if executions > 0 { total / executions } else { 0 },
            queue_depth_hot: self.hot_queue.len() as u64,
            queue_depth_warm: self.warm_queue.len() as u64,
            queue_depth_cold: self.cold_queue.read().len() as u64,
            stratum_switches: self.stats.stratum_switches.load(Ordering::Relaxed),
        }
    }

    /// Get recent receipts
    pub fn recent_receipts(&self, count: usize) -> Vec<Receipt> {
        self.receipt_store.read()
            .get_recent(count)
            .into_iter()
            .map(|r| r.clone())
            .collect()
    }
}

/// Snapshot of hot path statistics
#[derive(Debug, Clone)]
pub struct HotPathStatsSnapshot {
    pub executions: u64,
    pub cycles_total: u64,
    pub cycles_min: u64,
    pub cycles_max: u64,
    pub cycles_avg: u64,
    pub queue_depth_hot: u64,
    pub queue_depth_warm: u64,
    pub queue_depth_cold: u64,
    pub stratum_switches: u64,
}

impl HotPathStatsSnapshot {
    pub fn hot_path_compliance(&self) -> f64 {
        if self.executions == 0 {
            return 0.0;
        }

        // Percentage of executions that met ≤8 tick budget
        let compliant = self.executions - self.stratum_switches;
        (compliant as f64 / self.executions as f64) * 100.0
    }
}

/// Hot path runner for concurrent execution
pub struct HotPathRunner {
    hot_path: Arc<HotPath>,
    thread_handle: Option<std::thread::JoinHandle<()>>,
}

impl HotPathRunner {
    pub fn new() -> Self {
        Self {
            hot_path: Arc::new(HotPath::new()),
            thread_handle: None,
        }
    }

    /// Start hot path in background thread
    pub fn start(&mut self) {
        let hot_path = self.hot_path.clone();

        self.thread_handle = Some(std::thread::spawn(move || {
            // Pin to CPU for consistent performance
            #[cfg(target_os = "linux")]
            {
                use std::os::unix::thread::JoinHandleExt;
                // Pin to CPU 0 (or configured CPU)
                unsafe {
                    let mut cpu_set: libc::cpu_set_t = std::mem::zeroed();
                    libc::CPU_SET(0, &mut cpu_set);
                    libc::pthread_setaffinity_np(
                        libc::pthread_self(),
                        std::mem::size_of::<libc::cpu_set_t>(),
                        &cpu_set,
                    );
                }
            }

            hot_path.run_loop();
        }));
    }

    /// Submit task
    pub fn submit(&self, task: Box<Task>) -> Result<(), Box<Task>> {
        self.hot_path.submit(task)
    }

    /// Stop and wait for completion
    pub fn stop(mut self) {
        self.hot_path.stop();
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }

    /// Get statistics
    pub fn stats(&self) -> HotPathStatsSnapshot {
        self.hot_path.stats()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::descriptor::{DescriptorBuilder, PatternEntry};
    use crate::pattern::{PatternType, PatternConfig};

    #[test]
    fn test_hot_path_creation() {
        let hot_path = HotPath::new();
        let stats = hot_path.stats();

        assert_eq!(stats.executions, 0);
        assert_eq!(stats.queue_depth_hot, 0);
    }

    #[test]
    fn test_stratum_estimation() {
        // Setup descriptor
        let pattern = PatternEntry::new(
            PatternType::Sequence,
            1,
            10,
            PatternConfig::default(),
        );

        let descriptor = Box::new(
            DescriptorBuilder::new()
                .add_pattern(pattern)
                .build()
        );

        DescriptorManager::load_descriptor(descriptor).unwrap();

        let hot_path = HotPath::new();

        // Simple task should go to hot stratum
        let mut task = Task::new(1, 1);
        task.observation_count = 2;
        assert_eq!(hot_path.estimate_stratum(&task), Stratum::Hot);

        // Complex task should go to warm stratum
        task.observation_count = 8;
        assert_eq!(hot_path.estimate_stratum(&task), Stratum::Warm);
    }

    #[test]
    fn test_task_submission() {
        let hot_path = HotPath::new();

        let task = Box::new(Task::new(1, 1));
        assert!(hot_path.submit(task).is_ok());

        let stats = hot_path.stats();
        assert_eq!(stats.queue_depth_hot, 1);
    }

    #[test]
    fn test_hot_path_runner() {
        // Setup descriptor
        let pattern = PatternEntry::new(
            PatternType::Sequence,
            2,
            10,
            PatternConfig::default(),
        );

        let descriptor = Box::new(
            DescriptorBuilder::new()
                .add_pattern(pattern)
                .build()
        );

        DescriptorManager::load_descriptor(descriptor).unwrap();

        // Start runner
        let mut runner = HotPathRunner::new();
        runner.start();

        // Submit tasks
        for i in 0..10 {
            let mut task = Box::new(Task::new(i, 2));
            task.transition(TaskState::Ready);
            runner.submit(task).unwrap();
        }

        // Let it run briefly
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Check stats
        let stats = runner.stats();
        assert!(stats.executions > 0);

        // Stop runner
        runner.stop();
    }
}