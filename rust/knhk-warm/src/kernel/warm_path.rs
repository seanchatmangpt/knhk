// kernel/warm_path.rs - Warm stratum executor (sub-millisecond budget)
// Phase 3: Warm Path & Descriptor Management
// DOCTRINE: All 6 covenants apply, Rule 4 (All changes are descriptor changes)

use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use crossbeam::channel::{bounded, Receiver, Sender, TryRecvError};
use dashmap::DashMap;
use parking_lot::{RwLock, Mutex};
use tracing::{debug, error, info, warn, trace};
use serde::{Serialize, Deserialize};
use std::collections::VecDeque;
use std::mem::ManuallyDrop;
use std::ptr;
use std::alloc::{alloc, dealloc, Layout};

// Constants for warm path execution
const WARM_PATH_BUDGET_US: u64 = 1000; // 1ms budget for warm path
const STATS_COLLECTION_INTERVAL_MS: u64 = 100;
const MAX_BUFFER_SIZE: usize = 16384;
const DEGRADATION_THRESHOLD: f64 = 0.8; // 80% of budget triggers degradation

/// Warm path execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WarmPathResult {
    Success {
        execution_time_us: u64,
        items_processed: usize,
        telemetry_emitted: usize,
    },
    Degraded {
        reason: String,
        fallback_mode: FallbackMode,
        items_deferred: usize,
    },
    Failed {
        error: String,
        recovery_action: RecoveryAction,
    },
}

/// Fallback modes for graceful degradation
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FallbackMode {
    ReducedThroughput,
    BufferedExecution,
    DeferredProcessing,
    PassThrough,
}

/// Recovery actions for failures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryAction {
    Retry,
    Rollback,
    Emergency,
    Shutdown,
}

/// Statistics for warm path execution
#[derive(Debug, Clone)]
pub struct WarmPathStats {
    pub total_executions: AtomicU64,
    pub successful_executions: AtomicU64,
    pub degraded_executions: AtomicU64,
    pub failed_executions: AtomicU64,
    pub total_execution_time_us: AtomicU64,
    pub items_processed: AtomicU64,
    pub telemetry_emitted: AtomicU64,
    pub budget_violations: AtomicU64,
    pub current_load: AtomicU64,
    pub peak_load: AtomicU64,
}

impl WarmPathStats {
    fn new() -> Self {
        Self {
            total_executions: AtomicU64::new(0),
            successful_executions: AtomicU64::new(0),
            degraded_executions: AtomicU64::new(0),
            failed_executions: AtomicU64::new(0),
            total_execution_time_us: AtomicU64::new(0),
            items_processed: AtomicU64::new(0),
            telemetry_emitted: AtomicU64::new(0),
            budget_violations: AtomicU64::new(0),
            current_load: AtomicU64::new(0),
            peak_load: AtomicU64::new(0),
        }
    }

    pub fn record_execution(&self, result: &WarmPathResult, duration: Duration) {
        self.total_executions.fetch_add(1, Ordering::Relaxed);
        let duration_us = duration.as_micros() as u64;
        self.total_execution_time_us.fetch_add(duration_us, Ordering::Relaxed);

        match result {
            WarmPathResult::Success { items_processed, telemetry_emitted, .. } => {
                self.successful_executions.fetch_add(1, Ordering::Relaxed);
                self.items_processed.fetch_add(*items_processed as u64, Ordering::Relaxed);
                self.telemetry_emitted.fetch_add(*telemetry_emitted as u64, Ordering::Relaxed);
            }
            WarmPathResult::Degraded { .. } => {
                self.degraded_executions.fetch_add(1, Ordering::Relaxed);
            }
            WarmPathResult::Failed { .. } => {
                self.failed_executions.fetch_add(1, Ordering::Relaxed);
            }
        }

        if duration_us > WARM_PATH_BUDGET_US {
            self.budget_violations.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn update_load(&self, current: u64) {
        self.current_load.store(current, Ordering::Relaxed);

        // Update peak load if necessary
        let mut peak = self.peak_load.load(Ordering::Relaxed);
        while current > peak {
            match self.peak_load.compare_exchange_weak(
                peak,
                current,
                Ordering::Release,
                Ordering::Relaxed
            ) {
                Ok(_) => break,
                Err(x) => peak = x,
            }
        }
    }
}

/// Work item for warm path processing
#[derive(Debug, Clone)]
pub struct WorkItem {
    pub id: u64,
    pub payload: Vec<u8>,
    pub priority: u8,
    pub created_at: Instant,
    pub deadline: Option<Instant>,
}

/// Buffer for telemetry and receipts
pub struct TelemetryBuffer {
    receipts: Mutex<VecDeque<Receipt>>,
    metrics: DashMap<String, f64>,
    events: Mutex<VecDeque<Event>>,
    max_size: usize,
}

impl TelemetryBuffer {
    fn new(max_size: usize) -> Self {
        Self {
            receipts: Mutex::new(VecDeque::with_capacity(max_size)),
            metrics: DashMap::new(),
            events: Mutex::new(VecDeque::with_capacity(max_size)),
            max_size,
        }
    }

    fn add_receipt(&self, receipt: Receipt) -> bool {
        let mut receipts = self.receipts.lock();
        if receipts.len() >= self.max_size {
            receipts.pop_front(); // Drop oldest
        }
        receipts.push_back(receipt);
        true
    }

    fn add_metric(&self, name: String, value: f64) {
        self.metrics.insert(name, value);
    }

    fn add_event(&self, event: Event) -> bool {
        let mut events = self.events.lock();
        if events.len() >= self.max_size {
            events.pop_front(); // Drop oldest
        }
        events.push_back(event);
        true
    }

    fn flush(&self) -> (Vec<Receipt>, Vec<(String, f64)>, Vec<Event>) {
        let mut receipts = self.receipts.lock();
        let mut events = self.events.lock();

        let flushed_receipts: Vec<Receipt> = receipts.drain(..).collect();
        let flushed_events: Vec<Event> = events.drain(..).collect();
        let flushed_metrics: Vec<(String, f64)> =
            self.metrics.iter().map(|e| (e.key().clone(), *e.value())).collect();

        self.metrics.clear();

        (flushed_receipts, flushed_metrics, flushed_events)
    }
}

/// Receipt for completed work
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    pub work_id: u64,
    pub completed_at: Instant,
    pub execution_time_us: u64,
    pub result: String,
}

/// Event for telemetry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub timestamp: Instant,
    pub event_type: String,
    pub details: String,
}

/// Custom slab allocator for warm path
pub struct WarmPathAllocator {
    slab: *mut u8,
    layout: Layout,
    free_list: Mutex<Vec<usize>>,
    allocated: AtomicUsize,
    total_size: usize,
}

unsafe impl Send for WarmPathAllocator {}
unsafe impl Sync for WarmPathAllocator {}

impl WarmPathAllocator {
    pub fn new(size: usize) -> Self {
        let layout = Layout::from_size_align(size, 64).expect("Invalid layout");
        let slab = unsafe { alloc(layout) };

        if slab.is_null() {
            panic!("Failed to allocate warm path slab");
        }

        Self {
            slab,
            layout,
            free_list: Mutex::new(Vec::new()),
            allocated: AtomicUsize::new(0),
            total_size: size,
        }
    }

    pub fn allocate(&self, size: usize) -> Option<*mut u8> {
        let aligned_size = (size + 63) & !63; // 64-byte alignment

        let current = self.allocated.fetch_add(aligned_size, Ordering::SeqCst);
        if current + aligned_size > self.total_size {
            self.allocated.fetch_sub(aligned_size, Ordering::SeqCst);
            return None;
        }

        unsafe {
            Some(self.slab.add(current))
        }
    }

    pub fn deallocate(&self, _ptr: *mut u8, _size: usize) {
        // Simple slab allocator - no individual deallocation
        // Reset happens on executor reset
    }

    pub fn reset(&self) {
        self.allocated.store(0, Ordering::SeqCst);
        self.free_list.lock().clear();
    }
}

impl Drop for WarmPathAllocator {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.slab, self.layout);
        }
    }
}

/// Main warm path executor
pub struct WarmPathExecutor {
    stats: Arc<WarmPathStats>,
    work_queue: Arc<crossbeam::queue::ArrayQueue<WorkItem>>,
    telemetry_buffer: Arc<TelemetryBuffer>,
    allocator: Arc<WarmPathAllocator>,
    coordination_tx: Sender<CoordinationMessage>,
    coordination_rx: Receiver<CoordinationMessage>,
    descriptor_version: Arc<AtomicU64>,
    running: Arc<AtomicBool>,
    degraded: Arc<AtomicBool>,
    load_monitor: Arc<LoadMonitor>,
}

impl WarmPathExecutor {
    pub fn new(queue_size: usize, buffer_size: usize, slab_size: usize) -> Self {
        let (tx, rx) = bounded(1024);

        Self {
            stats: Arc::new(WarmPathStats::new()),
            work_queue: Arc::new(crossbeam::queue::ArrayQueue::new(queue_size)),
            telemetry_buffer: Arc::new(TelemetryBuffer::new(buffer_size)),
            allocator: Arc::new(WarmPathAllocator::new(slab_size)),
            coordination_tx: tx,
            coordination_rx: rx,
            descriptor_version: Arc::new(AtomicU64::new(0)),
            running: Arc::new(AtomicBool::new(true)),
            degraded: Arc::new(AtomicBool::new(false)),
            load_monitor: Arc::new(LoadMonitor::new()),
        }
    }

    /// Execute warm path work
    pub fn execute(&self) -> WarmPathResult {
        let start = Instant::now();
        let mut items_processed = 0;
        let mut telemetry_emitted = 0;

        // Check if we're in degraded mode
        if self.degraded.load(Ordering::Acquire) {
            return self.execute_degraded();
        }

        // Process work items within budget
        while start.elapsed().as_micros() < WARM_PATH_BUDGET_US as u128 * DEGRADATION_THRESHOLD as u128 {
            match self.work_queue.pop() {
                Some(item) => {
                    match self.process_item(&item) {
                        Ok(receipt) => {
                            self.telemetry_buffer.add_receipt(receipt);
                            telemetry_emitted += 1;
                            items_processed += 1;
                        }
                        Err(e) => {
                            warn!("Failed to process item {}: {}", item.id, e);
                            // Attempt recovery
                            if let Err(re) = self.recover_item(&item) {
                                error!("Recovery failed for item {}: {}", item.id, re);
                            }
                        }
                    }
                }
                None => break, // Queue empty
            }
        }

        let execution_time = start.elapsed();
        let execution_time_us = execution_time.as_micros() as u64;

        // Update statistics
        self.stats.update_load(self.work_queue.len() as u64);

        // Check for budget violation
        if execution_time_us > WARM_PATH_BUDGET_US {
            warn!("Warm path budget exceeded: {}us > {}us", execution_time_us, WARM_PATH_BUDGET_US);
            self.degraded.store(true, Ordering::Release);

            return WarmPathResult::Degraded {
                reason: format!("Budget exceeded: {}us", execution_time_us),
                fallback_mode: FallbackMode::BufferedExecution,
                items_deferred: self.work_queue.len(),
            };
        }

        // Emit aggregated statistics
        self.emit_statistics();

        WarmPathResult::Success {
            execution_time_us,
            items_processed,
            telemetry_emitted,
        }
    }

    fn process_item(&self, item: &WorkItem) -> Result<Receipt, String> {
        let start = Instant::now();

        // Simulate processing based on payload
        // In real implementation, this would dispatch to appropriate handler
        std::thread::sleep(Duration::from_micros(10)); // Simulated work

        let execution_time_us = start.elapsed().as_micros() as u64;

        Ok(Receipt {
            work_id: item.id,
            completed_at: Instant::now(),
            execution_time_us,
            result: format!("Processed {} bytes", item.payload.len()),
        })
    }

    fn recover_item(&self, item: &WorkItem) -> Result<(), String> {
        // Attempt to re-queue with lower priority
        let mut recovered = item.clone();
        recovered.priority = recovered.priority.saturating_sub(1);

        self.work_queue.push(recovered)
            .map_err(|_| "Recovery queue full".to_string())
    }

    fn execute_degraded(&self) -> WarmPathResult {
        // In degraded mode, we buffer work for later processing
        let items_deferred = self.work_queue.len();

        info!("Executing in degraded mode, {} items deferred", items_deferred);

        // Notify coordination layer
        let _ = self.coordination_tx.try_send(
            CoordinationMessage::DegradationNotice {
                mode: FallbackMode::BufferedExecution,
                items_deferred,
            }
        );

        WarmPathResult::Degraded {
            reason: "System in degraded mode".to_string(),
            fallback_mode: FallbackMode::BufferedExecution,
            items_deferred,
        }
    }

    fn emit_statistics(&self) {
        let total = self.stats.total_executions.load(Ordering::Relaxed);
        let successful = self.stats.successful_executions.load(Ordering::Relaxed);
        let degraded = self.stats.degraded_executions.load(Ordering::Relaxed);
        let failed = self.stats.failed_executions.load(Ordering::Relaxed);

        self.telemetry_buffer.add_metric("warm_path.total".to_string(), total as f64);
        self.telemetry_buffer.add_metric("warm_path.successful".to_string(), successful as f64);
        self.telemetry_buffer.add_metric("warm_path.degraded".to_string(), degraded as f64);
        self.telemetry_buffer.add_metric("warm_path.failed".to_string(), failed as f64);

        if total > 0 {
            let avg_time = self.stats.total_execution_time_us.load(Ordering::Relaxed) as f64 / total as f64;
            self.telemetry_buffer.add_metric("warm_path.avg_execution_us".to_string(), avg_time);
        }
    }

    pub fn submit_work(&self, item: WorkItem) -> Result<(), WorkItem> {
        self.work_queue.push(item)
    }

    pub fn flush_telemetry(&self) -> (Vec<Receipt>, Vec<(String, f64)>, Vec<Event>) {
        self.telemetry_buffer.flush()
    }

    pub fn reset(&self) {
        self.allocator.reset();
        self.degraded.store(false, Ordering::Release);
        // Note: We don't clear the work queue to preserve pending work
    }

    pub fn shutdown(&self) {
        self.running.store(false, Ordering::Release);

        // Notify coordination layer
        let _ = self.coordination_tx.send(CoordinationMessage::Shutdown);
    }

    pub fn get_stats(&self) -> Arc<WarmPathStats> {
        Arc::clone(&self.stats)
    }

    pub fn is_degraded(&self) -> bool {
        self.degraded.load(Ordering::Acquire)
    }

    pub fn clear_degradation(&self) {
        self.degraded.store(false, Ordering::Release);
    }
}

/// Load monitor for tracking system load
struct LoadMonitor {
    samples: RwLock<VecDeque<LoadSample>>,
    max_samples: usize,
}

#[derive(Debug, Clone)]
struct LoadSample {
    timestamp: Instant,
    queue_depth: usize,
    execution_time_us: u64,
}

impl LoadMonitor {
    fn new() -> Self {
        Self {
            samples: RwLock::new(VecDeque::with_capacity(100)),
            max_samples: 100,
        }
    }

    fn record_sample(&self, queue_depth: usize, execution_time_us: u64) {
        let mut samples = self.samples.write();

        if samples.len() >= self.max_samples {
            samples.pop_front();
        }

        samples.push_back(LoadSample {
            timestamp: Instant::now(),
            queue_depth,
            execution_time_us,
        });
    }

    fn get_average_load(&self) -> f64 {
        let samples = self.samples.read();
        if samples.is_empty() {
            return 0.0;
        }

        let total: usize = samples.iter().map(|s| s.queue_depth).sum();
        total as f64 / samples.len() as f64
    }

    fn predict_overload(&self) -> bool {
        let samples = self.samples.read();
        if samples.len() < 10 {
            return false;
        }

        // Simple trend analysis: check if load is increasing
        let recent: Vec<_> = samples.iter().rev().take(5).collect();
        let older: Vec<_> = samples.iter().rev().skip(5).take(5).collect();

        let recent_avg: f64 = recent.iter().map(|s| s.queue_depth as f64).sum::<f64>() / 5.0;
        let older_avg: f64 = older.iter().map(|s| s.queue_depth as f64).sum::<f64>() / 5.0;

        recent_avg > older_avg * 1.5 // 50% increase indicates potential overload
    }
}

/// Coordination messages between warm path and outer layers
#[derive(Debug, Clone)]
pub enum CoordinationMessage {
    DegradationNotice {
        mode: FallbackMode,
        items_deferred: usize,
    },
    LoadReport {
        current_load: f64,
        predicted_overload: bool,
    },
    DescriptorUpdate {
        version: u64,
    },
    Shutdown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_warm_path_executor_creation() {
        let executor = WarmPathExecutor::new(1000, 100, 1024 * 1024);
        assert!(!executor.is_degraded());
    }

    #[test]
    fn test_work_submission() {
        let executor = WarmPathExecutor::new(10, 100, 1024 * 1024);

        let item = WorkItem {
            id: 1,
            payload: vec![1, 2, 3],
            priority: 10,
            created_at: Instant::now(),
            deadline: None,
        };

        assert!(executor.submit_work(item).is_ok());
    }

    #[test]
    fn test_statistics_tracking() {
        let stats = WarmPathStats::new();

        let result = WarmPathResult::Success {
            execution_time_us: 500,
            items_processed: 10,
            telemetry_emitted: 5,
        };

        stats.record_execution(&result, Duration::from_micros(500));

        assert_eq!(stats.total_executions.load(Ordering::Relaxed), 1);
        assert_eq!(stats.successful_executions.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_telemetry_buffer() {
        let buffer = TelemetryBuffer::new(10);

        for i in 0..15 {
            let receipt = Receipt {
                work_id: i,
                completed_at: Instant::now(),
                execution_time_us: 100,
                result: format!("Result {}", i),
            };
            buffer.add_receipt(receipt);
        }

        let (receipts, _, _) = buffer.flush();
        assert_eq!(receipts.len(), 10); // Max size enforced
    }

    #[test]
    fn test_load_monitor() {
        let monitor = LoadMonitor::new();

        for i in 0..10 {
            monitor.record_sample(i * 10, 100);
        }

        let avg = monitor.get_average_load();
        assert!(avg > 0.0);
    }

    #[test]
    fn test_degradation_detection() {
        let executor = WarmPathExecutor::new(100, 100, 1024 * 1024);

        // Simulate overload
        for i in 0..100 {
            let item = WorkItem {
                id: i,
                payload: vec![0; 1000],
                priority: 5,
                created_at: Instant::now(),
                deadline: Some(Instant::now() + Duration::from_millis(100)),
            };
            let _ = executor.submit_work(item);
        }

        // Execute and check for degradation
        let result = executor.execute();
        match result {
            WarmPathResult::Degraded { .. } | WarmPathResult::Success { .. } => {
                // Expected outcomes
            }
            _ => panic!("Unexpected result"),
        }
    }
}