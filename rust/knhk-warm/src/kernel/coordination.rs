// kernel/coordination.rs - Channel management and lock-free message passing
// Phase 3: Coordination between kernel and outer layers
// DOCTRINE: Covenant 5 (Latency Is Our Currency) - Coordination must not block

use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use crossbeam::channel::{bounded, unbounded, Receiver, Sender, TryRecvError, RecvTimeoutError};
use crossbeam::queue::{ArrayQueue, SegQueue};
use parking_lot::{Mutex, RwLock};
use dashmap::DashMap;
use tracing::{debug, error, info, warn};
use serde::{Serialize, Deserialize};

/// Message types for coordination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoordinationMessage {
    // Control messages
    Startup { config: StartupConfig },
    Shutdown { graceful: bool, timeout_ms: u64 },
    Reconfigure { config: RuntimeConfig },

    // Health messages
    HealthCheck { requester_id: String },
    HealthResponse { status: HealthStatus },

    // Load management
    LoadReport { current: f64, predicted: f64, capacity: f64 },
    BackpressureSignal { level: BackpressureLevel },

    // Work distribution
    WorkRequest { priority: u8, estimated_cost: u64 },
    WorkAssignment { work_id: String, deadline: Option<Instant> },
    WorkCompletion { work_id: String, result: WorkResult },

    // Synchronization
    Barrier { id: String, participants: usize },
    BarrierReached { id: String, participant: String },

    // Telemetry
    TelemetryBatch { size: usize, compressed: bool },
    MetricsSnapshot { timestamp: u64, metrics: Vec<(String, f64)> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupConfig {
    pub worker_threads: usize,
    pub buffer_size: usize,
    pub telemetry_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    pub max_throughput: Option<u64>,
    pub priority_threshold: u8,
    pub telemetry_sampling_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded(String),
    Unhealthy(String),
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BackpressureLevel {
    None,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkResult {
    Success(Vec<u8>),
    Failure(String),
    Timeout,
    Cancelled,
}

/// Lock-free message queue using crossbeam
pub struct LockFreeQueue<T> {
    queue: SegQueue<T>,
    size: AtomicUsize,
    capacity: usize,
}

impl<T> LockFreeQueue<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            queue: SegQueue::new(),
            size: AtomicUsize::new(0),
            capacity,
        }
    }

    pub fn push(&self, item: T) -> Result<(), T> {
        let current_size = self.size.load(Ordering::Acquire);
        if current_size >= self.capacity {
            return Err(item);
        }

        self.queue.push(item);
        self.size.fetch_add(1, Ordering::Release);
        Ok(())
    }

    pub fn pop(&self) -> Option<T> {
        let item = self.queue.pop();
        if item.is_some() {
            self.size.fetch_sub(1, Ordering::Release);
        }
        item
    }

    pub fn len(&self) -> usize {
        self.size.load(Ordering::Acquire)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Channel manager for bidirectional communication
pub struct ChannelManager {
    // Inbound channels (outer layers -> kernel)
    control_rx: Receiver<CoordinationMessage>,
    control_tx: Sender<CoordinationMessage>,
    work_rx: Receiver<CoordinationMessage>,
    work_tx: Sender<CoordinationMessage>,

    // Outbound channels (kernel -> outer layers)
    telemetry_rx: Receiver<CoordinationMessage>,
    telemetry_tx: Sender<CoordinationMessage>,
    health_rx: Receiver<CoordinationMessage>,
    health_tx: Sender<CoordinationMessage>,

    // Multiplexed channel for high-throughput
    multiplex_queue: Arc<LockFreeQueue<CoordinationMessage>>,

    // Channel statistics
    stats: Arc<ChannelStats>,
}

#[derive(Debug)]
struct ChannelStats {
    messages_sent: AtomicU64,
    messages_received: AtomicU64,
    messages_dropped: AtomicU64,
    channel_full_events: AtomicU64,
    average_latency_us: AtomicU64,
}

impl ChannelManager {
    pub fn new(buffer_size: usize) -> Self {
        let (control_tx, control_rx) = bounded(buffer_size);
        let (work_tx, work_rx) = bounded(buffer_size * 4); // Larger buffer for work
        let (telemetry_tx, telemetry_rx) = unbounded(); // Unbounded for telemetry
        let (health_tx, health_rx) = bounded(16); // Small buffer for health

        Self {
            control_rx,
            control_tx,
            work_rx,
            work_tx,
            telemetry_rx,
            telemetry_tx,
            health_rx,
            health_tx,
            multiplex_queue: Arc::new(LockFreeQueue::new(buffer_size * 8)),
            stats: Arc::new(ChannelStats {
                messages_sent: AtomicU64::new(0),
                messages_received: AtomicU64::new(0),
                messages_dropped: AtomicU64::new(0),
                channel_full_events: AtomicU64::new(0),
                average_latency_us: AtomicU64::new(0),
            }),
        }
    }

    /// Send control message
    pub fn send_control(&self, msg: CoordinationMessage) -> Result<(), String> {
        let start = Instant::now();

        self.control_tx
            .try_send(msg)
            .map_err(|e| {
                self.stats.channel_full_events.fetch_add(1, Ordering::Relaxed);
                format!("Control channel full: {}", e)
            })?;

        self.record_send_latency(start);
        Ok(())
    }

    /// Receive control message
    pub fn recv_control(&self) -> Result<CoordinationMessage, TryRecvError> {
        let msg = self.control_rx.try_recv()?;
        self.stats.messages_received.fetch_add(1, Ordering::Relaxed);
        Ok(msg)
    }

    /// Send work message
    pub fn send_work(&self, msg: CoordinationMessage) -> Result<(), String> {
        let start = Instant::now();

        self.work_tx
            .try_send(msg)
            .map_err(|e| {
                self.stats.channel_full_events.fetch_add(1, Ordering::Relaxed);
                format!("Work channel full: {}", e)
            })?;

        self.record_send_latency(start);
        Ok(())
    }

    /// Receive work message with timeout
    pub fn recv_work_timeout(&self, timeout: Duration) -> Result<CoordinationMessage, RecvTimeoutError> {
        let msg = self.work_rx.recv_timeout(timeout)?;
        self.stats.messages_received.fetch_add(1, Ordering::Relaxed);
        Ok(msg)
    }

    /// Send telemetry (non-blocking)
    pub fn send_telemetry(&self, msg: CoordinationMessage) {
        let _ = self.telemetry_tx.send(msg);
        self.stats.messages_sent.fetch_add(1, Ordering::Relaxed);
    }

    /// Send to multiplexed queue
    pub fn send_multiplex(&self, msg: CoordinationMessage) -> Result<(), CoordinationMessage> {
        self.multiplex_queue.push(msg)?;
        self.stats.messages_sent.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Receive from multiplexed queue
    pub fn recv_multiplex(&self) -> Option<CoordinationMessage> {
        let msg = self.multiplex_queue.pop();
        if msg.is_some() {
            self.stats.messages_received.fetch_add(1, Ordering::Relaxed);
        }
        msg
    }

    fn record_send_latency(&self, start: Instant) {
        let latency_us = start.elapsed().as_micros() as u64;
        self.stats.messages_sent.fetch_add(1, Ordering::Relaxed);

        // Update average (simplified - in production use proper averaging)
        let current = self.stats.average_latency_us.load(Ordering::Relaxed);
        let new_avg = (current + latency_us) / 2;
        self.stats.average_latency_us.store(new_avg, Ordering::Relaxed);
    }

    pub fn get_stats(&self) -> ChannelStatistics {
        ChannelStatistics {
            messages_sent: self.stats.messages_sent.load(Ordering::Relaxed),
            messages_received: self.stats.messages_received.load(Ordering::Relaxed),
            messages_dropped: self.stats.messages_dropped.load(Ordering::Relaxed),
            channel_full_events: self.stats.channel_full_events.load(Ordering::Relaxed),
            average_latency_us: self.stats.average_latency_us.load(Ordering::Relaxed),
            multiplex_queue_size: self.multiplex_queue.len(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelStatistics {
    pub messages_sent: u64,
    pub messages_received: u64,
    pub messages_dropped: u64,
    pub channel_full_events: u64,
    pub average_latency_us: u64,
    pub multiplex_queue_size: usize,
}

/// Backpressure controller
pub struct BackpressureController {
    thresholds: BackpressureThresholds,
    current_level: Arc<Mutex<BackpressureLevel>>,
    queue_depths: Arc<DashMap<String, usize>>,
    memory_usage: Arc<AtomicUsize>,
}

#[derive(Debug, Clone)]
struct BackpressureThresholds {
    low: f64,
    medium: f64,
    high: f64,
    critical: f64,
}

impl Default for BackpressureThresholds {
    fn default() -> Self {
        Self {
            low: 0.5,
            medium: 0.7,
            high: 0.85,
            critical: 0.95,
        }
    }
}

impl BackpressureController {
    pub fn new() -> Self {
        Self {
            thresholds: BackpressureThresholds::default(),
            current_level: Arc::new(Mutex::new(BackpressureLevel::None)),
            queue_depths: Arc::new(DashMap::new()),
            memory_usage: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn update_queue_depth(&self, queue_name: String, depth: usize, capacity: usize) {
        self.queue_depths.insert(queue_name.clone(), depth);

        let utilization = depth as f64 / capacity as f64;
        let new_level = self.calculate_level(utilization);

        let mut current = self.current_level.lock();
        if self.level_to_value(new_level) > self.level_to_value(*current) {
            *current = new_level;
            debug!("Backpressure increased to {:?} for queue {}", new_level, queue_name);
        }
    }

    pub fn update_memory_usage(&self, bytes: usize, max_bytes: usize) {
        self.memory_usage.store(bytes, Ordering::Release);

        let utilization = bytes as f64 / max_bytes as f64;
        let new_level = self.calculate_level(utilization);

        let mut current = self.current_level.lock();
        if self.level_to_value(new_level) > self.level_to_value(*current) {
            *current = new_level;
            debug!("Backpressure increased to {:?} due to memory", new_level);
        }
    }

    pub fn get_level(&self) -> BackpressureLevel {
        *self.current_level.lock()
    }

    pub fn should_accept_work(&self, priority: u8) -> bool {
        let level = self.get_level();
        match level {
            BackpressureLevel::None => true,
            BackpressureLevel::Low => priority >= 3,
            BackpressureLevel::Medium => priority >= 5,
            BackpressureLevel::High => priority >= 7,
            BackpressureLevel::Critical => priority >= 9,
        }
    }

    fn calculate_level(&self, utilization: f64) -> BackpressureLevel {
        if utilization >= self.thresholds.critical {
            BackpressureLevel::Critical
        } else if utilization >= self.thresholds.high {
            BackpressureLevel::High
        } else if utilization >= self.thresholds.medium {
            BackpressureLevel::Medium
        } else if utilization >= self.thresholds.low {
            BackpressureLevel::Low
        } else {
            BackpressureLevel::None
        }
    }

    fn level_to_value(&self, level: BackpressureLevel) -> u8 {
        match level {
            BackpressureLevel::None => 0,
            BackpressureLevel::Low => 1,
            BackpressureLevel::Medium => 2,
            BackpressureLevel::High => 3,
            BackpressureLevel::Critical => 4,
        }
    }
}

/// Shutdown coordinator for graceful termination
pub struct ShutdownCoordinator {
    shutdown_signal: Arc<AtomicBool>,
    shutdown_deadline: Arc<Mutex<Option<Instant>>>,
    components: Arc<DashMap<String, ComponentState>>,
    shutdown_sequence: Arc<RwLock<Vec<String>>>,
}

#[derive(Debug, Clone, Copy)]
enum ComponentState {
    Running,
    Stopping,
    Stopped,
}

impl ShutdownCoordinator {
    pub fn new() -> Self {
        Self {
            shutdown_signal: Arc::new(AtomicBool::new(false)),
            shutdown_deadline: Arc::new(Mutex::new(None)),
            components: Arc::new(DashMap::new()),
            shutdown_sequence: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn register_component(&self, name: String, dependencies: Vec<String>) {
        self.components.insert(name.clone(), ComponentState::Running);

        // Update shutdown sequence based on dependencies
        let mut sequence = self.shutdown_sequence.write();

        // Components with no dependencies shut down last
        if dependencies.is_empty() {
            sequence.insert(0, name);
        } else {
            // Find position after all dependencies
            let mut insert_pos = sequence.len();
            for (i, comp) in sequence.iter().enumerate() {
                if dependencies.contains(comp) {
                    insert_pos = insert_pos.min(i);
                }
            }
            sequence.insert(insert_pos, name);
        }
    }

    pub fn initiate_shutdown(&self, graceful: bool, timeout: Duration) {
        self.shutdown_signal.store(true, Ordering::Release);

        if graceful {
            *self.shutdown_deadline.lock() = Some(Instant::now() + timeout);
            info!("Graceful shutdown initiated with {:?} timeout", timeout);
        } else {
            warn!("Immediate shutdown initiated");
        }
    }

    pub fn notify_component_stopping(&self, name: &str) {
        if let Some(mut entry) = self.components.get_mut(name) {
            *entry = ComponentState::Stopping;
        }
    }

    pub fn notify_component_stopped(&self, name: &str) {
        if let Some(mut entry) = self.components.get_mut(name) {
            *entry = ComponentState::Stopped;
            info!("Component {} stopped", name);
        }
    }

    pub fn is_shutdown_requested(&self) -> bool {
        self.shutdown_signal.load(Ordering::Acquire)
    }

    pub fn is_shutdown_complete(&self) -> bool {
        self.components
            .iter()
            .all(|entry| matches!(*entry.value(), ComponentState::Stopped))
    }

    pub fn get_shutdown_progress(&self) -> (usize, usize) {
        let total = self.components.len();
        let stopped = self.components
            .iter()
            .filter(|e| matches!(*e.value(), ComponentState::Stopped))
            .count();
        (stopped, total)
    }

    pub fn force_shutdown_if_deadline_exceeded(&self) -> bool {
        if let Some(deadline) = *self.shutdown_deadline.lock() {
            if Instant::now() > deadline {
                warn!("Shutdown deadline exceeded, forcing termination");
                return true;
            }
        }
        false
    }
}

/// Health monitor for system health signaling
pub struct HealthMonitor {
    health_checks: Arc<DashMap<String, HealthCheck>>,
    overall_health: Arc<Mutex<HealthStatus>>,
    check_interval: Duration,
    last_check: Arc<Mutex<Instant>>,
}

#[derive(Debug, Clone)]
struct HealthCheck {
    name: String,
    check_fn: Arc<dyn Fn() -> HealthStatus + Send + Sync>,
    last_result: HealthStatus,
    last_checked: Instant,
}

impl HealthMonitor {
    pub fn new(check_interval: Duration) -> Self {
        Self {
            health_checks: Arc::new(DashMap::new()),
            overall_health: Arc::new(Mutex::new(HealthStatus::Unknown)),
            check_interval,
            last_check: Arc::new(Mutex::new(Instant::now())),
        }
    }

    pub fn register_health_check<F>(&self, name: String, check_fn: F)
    where
        F: Fn() -> HealthStatus + Send + Sync + 'static,
    {
        let check = HealthCheck {
            name: name.clone(),
            check_fn: Arc::new(check_fn),
            last_result: HealthStatus::Unknown,
            last_checked: Instant::now(),
        };

        self.health_checks.insert(name, check);
    }

    pub fn run_health_checks(&self) {
        let now = Instant::now();
        let mut last_check = self.last_check.lock();

        if now.duration_since(*last_check) < self.check_interval {
            return;
        }

        *last_check = now;
        drop(last_check);

        let mut any_unhealthy = false;
        let mut any_degraded = false;

        for mut entry in self.health_checks.iter_mut() {
            let result = (entry.check_fn)();
            entry.last_result = result.clone();
            entry.last_checked = now;

            match result {
                HealthStatus::Unhealthy(_) => any_unhealthy = true,
                HealthStatus::Degraded(_) => any_degraded = true,
                _ => {}
            }
        }

        let mut overall = self.overall_health.lock();
        *overall = if any_unhealthy {
            HealthStatus::Unhealthy("One or more components unhealthy".to_string())
        } else if any_degraded {
            HealthStatus::Degraded("One or more components degraded".to_string())
        } else {
            HealthStatus::Healthy
        };
    }

    pub fn get_health(&self) -> HealthStatus {
        self.overall_health.lock().clone()
    }

    pub fn get_component_health(&self, component: &str) -> Option<HealthStatus> {
        self.health_checks
            .get(component)
            .map(|entry| entry.last_result.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_free_queue() {
        let queue = LockFreeQueue::new(10);

        for i in 0..10 {
            assert!(queue.push(i).is_ok());
        }

        assert!(queue.push(11).is_err()); // Queue full

        for i in 0..10 {
            assert_eq!(queue.pop(), Some(i));
        }

        assert_eq!(queue.pop(), None);
    }

    #[test]
    fn test_channel_manager() {
        let manager = ChannelManager::new(100);

        let msg = CoordinationMessage::HealthCheck {
            requester_id: "test".to_string(),
        };

        assert!(manager.send_control(msg.clone()).is_ok());
        assert!(matches!(manager.recv_control(), Ok(_)));

        let stats = manager.get_stats();
        assert_eq!(stats.messages_sent, 1);
        assert_eq!(stats.messages_received, 1);
    }

    #[test]
    fn test_backpressure_controller() {
        let controller = BackpressureController::new();

        controller.update_queue_depth("test_queue".to_string(), 10, 100);
        assert_eq!(controller.get_level(), BackpressureLevel::None);

        controller.update_queue_depth("test_queue".to_string(), 60, 100);
        assert_eq!(controller.get_level(), BackpressureLevel::Medium);

        assert!(controller.should_accept_work(8));
        assert!(!controller.should_accept_work(2));
    }

    #[test]
    fn test_shutdown_coordinator() {
        let coordinator = ShutdownCoordinator::new();

        coordinator.register_component("component_a".to_string(), vec![]);
        coordinator.register_component("component_b".to_string(), vec!["component_a".to_string()]);

        assert!(!coordinator.is_shutdown_requested());

        coordinator.initiate_shutdown(true, Duration::from_secs(5));
        assert!(coordinator.is_shutdown_requested());

        coordinator.notify_component_stopped("component_a");
        coordinator.notify_component_stopped("component_b");

        assert!(coordinator.is_shutdown_complete());
    }

    #[test]
    fn test_health_monitor() {
        let monitor = HealthMonitor::new(Duration::from_secs(1));

        monitor.register_health_check("test_check".to_string(), || {
            HealthStatus::Healthy
        });

        monitor.run_health_checks();

        assert_eq!(monitor.get_health(), HealthStatus::Healthy);
        assert_eq!(
            monitor.get_component_health("test_check"),
            Some(HealthStatus::Healthy)
        );
    }
}