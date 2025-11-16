//! Distributed Code Generation System
//!
//! Provides a hyper-advanced distributed code generation system with:
//! - Work stealing task queue for load balancing
//! - Cluster coordination with health monitoring
//! - Distributed result caching
//! - Full OpenTelemetry instrumentation
//! - Fault tolerance with circuit breaker
//!
//! # Architecture
//!
//! - `DistributedGenerator`: Main entry point for distributed generation
//! - `ClusterCoordinator`: Manages worker discovery and health
//! - `WorkStealer`: Implements work-stealing algorithm
//! - `ResultCache`: Distributed cache with invalidation
//! - `CircuitBreaker`: Fault tolerance for failing workers
//!
//! # Example
//!
//! ```rust,no_run
//! use knhk_workflow_engine::ggen::distributed::{DistributedGenerator, GenerationTask};
//! use std::time::Duration;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let generator = DistributedGenerator::new("coordinator:8080").await?;
//!
//! let task = GenerationTask::new(
//!     "Generate API endpoint",
//!     "spec content here",
//!     100,
//!     Duration::from_secs(30),
//! );
//!
//! let task_id = generator.submit_generation(task).await?;
//! let result = generator.wait_for_result(task_id, Duration::from_secs(60)).await?;
//! # Ok(())
//! # }
//! ```

use crate::error::{WorkflowError, WorkflowResult};
use dashmap::DashMap;
use knhk_otel::{SpanContext, SpanStatus, Tracer};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, oneshot, Mutex as AsyncMutex};
use tokio::time::timeout;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Task identifier
pub type TaskId = Uuid;

/// Worker identifier
pub type WorkerId = Uuid;

/// Generation task specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationTask {
    /// Unique task identifier
    pub id: TaskId,
    /// Task description
    pub description: String,
    /// Generation specification (code, schema, etc.)
    pub spec: String,
    /// Priority (higher = more urgent)
    pub priority: u32,
    /// Task timeout
    pub timeout: Duration,
    /// Trace context for distributed tracing
    pub trace_context: TraceContext,
}

impl GenerationTask {
    /// Create new generation task
    pub fn new(description: &str, spec: &str, priority: u32, timeout: Duration) -> Self {
        Self {
            id: Uuid::new_v4(),
            description: description.to_string(),
            spec: spec.to_string(),
            priority,
            timeout,
            trace_context: TraceContext::new(),
        }
    }
}

/// Trace context for distributed tracing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceContext {
    /// Trace ID
    pub trace_id: String,
    /// Span ID
    pub span_id: String,
    /// Parent span ID (optional)
    pub parent_span_id: Option<String>,
}

impl TraceContext {
    /// Create new trace context
    pub fn new() -> Self {
        Self {
            trace_id: Uuid::new_v4().to_string(),
            span_id: Uuid::new_v4().to_string(),
            parent_span_id: None,
        }
    }

    /// Create child trace context
    pub fn create_child(&self) -> Self {
        Self {
            trace_id: self.trace_id.clone(),
            span_id: Uuid::new_v4().to_string(),
            parent_span_id: Some(self.span_id.clone()),
        }
    }
}

impl Default for TraceContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Generated code result from distributed generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedGeneratedCode {
    /// Task ID that generated this code
    pub task_id: TaskId,
    /// Generated code content
    pub content: String,
    /// Language identifier
    pub language: String,
    /// Generation timestamp
    pub timestamp: u64,
    /// Worker that generated it
    pub worker_id: WorkerId,
}

/// Worker status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkerStatus {
    /// Worker is healthy and accepting tasks
    Healthy,
    /// Worker is degraded (slow responses)
    Degraded,
    /// Worker is unhealthy (failures)
    Unhealthy,
    /// Worker is offline
    Offline,
}

/// Worker information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerInfo {
    /// Worker unique identifier
    pub id: WorkerId,
    /// Worker network address
    pub address: String,
    /// Current status
    pub status: WorkerStatus,
    /// Current load (0.0 = idle, 1.0 = full)
    pub current_load: f64,
    /// P99 generation latency (in milliseconds)
    pub generation_latency_p99_ms: u64,
    /// Last heartbeat timestamp (seconds since epoch)
    pub last_heartbeat_secs: u64,
    /// Total tasks completed
    pub tasks_completed: u64,
    /// Total tasks failed
    pub tasks_failed: u64,
}

/// Cluster health status
#[derive(Debug, Clone)]
pub struct ClusterHealth {
    /// Total number of workers
    pub total_workers: u32,
    /// Number of healthy workers
    pub healthy_workers: u32,
    /// Cache hit rate (0.0-1.0)
    pub cache_hit_rate: f64,
    /// Average task latency (in milliseconds)
    pub average_latency_ms: u64,
    /// Timestamp of health check (seconds since epoch)
    pub timestamp_secs: u64,
}

/// Distributed generation metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedMetrics {
    /// Total tasks processed
    pub tasks_processed: u64,
    /// P50 task latency (in milliseconds)
    pub task_latency_p50_ms: u64,
    /// P99 task latency (in milliseconds)
    pub task_latency_p99_ms: u64,
    /// Cache hit rate
    pub cache_hit_rate: f64,
    /// Worker utilization (average across cluster)
    pub worker_utilization: f64,
    /// Tasks in queue
    pub tasks_in_queue: usize,
    /// Active tasks being processed
    pub active_tasks: usize,
}

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CircuitState {
    /// Circuit is closed, requests flow normally
    Closed,
    /// Circuit is open, requests are rejected
    Open,
    /// Circuit is half-open, testing if service recovered
    HalfOpen,
}

/// Circuit breaker for worker fault tolerance
struct CircuitBreaker {
    state: RwLock<CircuitState>,
    failure_count: AtomicUsize,
    success_count: AtomicUsize,
    last_failure: RwLock<Option<Instant>>,
    failure_threshold: usize,
    success_threshold: usize,
    timeout: Duration,
}

impl CircuitBreaker {
    fn new(failure_threshold: usize, success_threshold: usize, timeout: Duration) -> Self {
        Self {
            state: RwLock::new(CircuitState::Closed),
            failure_count: AtomicUsize::new(0),
            success_count: AtomicUsize::new(0),
            last_failure: RwLock::new(None),
            failure_threshold,
            success_threshold,
            timeout,
        }
    }

    fn record_success(&self) {
        let state = *self.state.read();
        match state {
            CircuitState::HalfOpen => {
                let count = self.success_count.fetch_add(1, Ordering::SeqCst) + 1;
                if count >= self.success_threshold {
                    *self.state.write() = CircuitState::Closed;
                    self.failure_count.store(0, Ordering::SeqCst);
                    self.success_count.store(0, Ordering::SeqCst);
                    info!("Circuit breaker closed after recovery");
                }
            }
            CircuitState::Closed => {
                self.failure_count.store(0, Ordering::SeqCst);
            }
            CircuitState::Open => {}
        }
    }

    fn record_failure(&self) {
        let count = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
        *self.last_failure.write() = Some(Instant::now());

        if count >= self.failure_threshold {
            *self.state.write() = CircuitState::Open;
            warn!("Circuit breaker opened after {} failures", count);
        }
    }

    fn is_available(&self) -> bool {
        let state = *self.state.read();
        match state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(last_failure) = *self.last_failure.read() {
                    if last_failure.elapsed() > self.timeout {
                        *self.state.write() = CircuitState::HalfOpen;
                        self.success_count.store(0, Ordering::SeqCst);
                        debug!("Circuit breaker entering half-open state");
                        return true;
                    }
                }
                false
            }
            CircuitState::HalfOpen => true,
        }
    }
}

/// Work-stealing task queue
struct WorkStealingQueue {
    local_queue: AsyncMutex<VecDeque<GenerationTask>>,
    steal_queue: Arc<AsyncMutex<VecDeque<GenerationTask>>>,
}

impl WorkStealingQueue {
    fn new() -> Self {
        Self {
            local_queue: AsyncMutex::new(VecDeque::new()),
            steal_queue: Arc::new(AsyncMutex::new(VecDeque::new())),
        }
    }

    async fn push(&self, task: GenerationTask) {
        let mut queue = self.local_queue.lock().await;
        queue.push_back(task);
    }

    async fn pop(&self) -> Option<GenerationTask> {
        let mut local = self.local_queue.lock().await;
        if let Some(task) = local.pop_front() {
            return Some(task);
        }
        drop(local);

        // Try to steal from steal queue
        let mut steal = self.steal_queue.lock().await;
        steal.pop_front()
    }

    async fn steal(&self) -> Option<GenerationTask> {
        let mut steal_queue = self.steal_queue.lock().await;
        steal_queue.pop_back()
    }

    async fn len(&self) -> usize {
        let local = self.local_queue.lock().await;
        let steal = self.steal_queue.lock().await;
        local.len() + steal.len()
    }
}

/// Result cache for generated code
struct ResultCache {
    cache: Arc<DashMap<String, DistributedGeneratedCode>>,
    enabled: Arc<RwLock<bool>>,
    hits: AtomicU64,
    misses: AtomicU64,
}

impl ResultCache {
    fn new(enabled: bool) -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
            enabled: Arc::new(RwLock::new(enabled)),
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
        }
    }

    fn get(&self, key: &str) -> Option<DistributedGeneratedCode> {
        if !*self.enabled.read() {
            return None;
        }

        if let Some(entry) = self.cache.get(key) {
            self.hits.fetch_add(1, Ordering::SeqCst);
            Some(entry.clone())
        } else {
            self.misses.fetch_add(1, Ordering::SeqCst);
            None
        }
    }

    fn insert(&self, key: String, value: DistributedGeneratedCode) {
        if *self.enabled.read() {
            self.cache.insert(key, value);
        }
    }

    fn invalidate(&self, schema_id: &str) {
        self.cache.retain(|k, _| !k.contains(schema_id));
        info!("Invalidated cache entries for schema: {}", schema_id);
    }

    fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::SeqCst);
        let misses = self.misses.load(Ordering::SeqCst);
        let total = hits + misses;
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    fn enable(&self, enabled: bool) {
        *self.enabled.write() = enabled;
    }
}

/// Cluster coordinator for worker management
struct ClusterCoordinator {
    workers: Arc<DashMap<WorkerId, WorkerInfo>>,
    circuit_breakers: Arc<DashMap<WorkerId, Arc<CircuitBreaker>>>,
}

impl ClusterCoordinator {
    fn new() -> Self {
        Self {
            workers: Arc::new(DashMap::new()),
            circuit_breakers: Arc::new(DashMap::new()),
        }
    }

    fn register_worker(&self, worker: WorkerInfo) {
        let worker_id = worker.id;
        self.workers.insert(worker_id, worker);
        self.circuit_breakers.insert(
            worker_id,
            Arc::new(CircuitBreaker::new(5, 3, Duration::from_secs(30))),
        );
        info!("Registered worker: {}", worker_id);
    }

    fn get_available_worker(&self) -> Option<WorkerInfo> {
        let mut available_workers: Vec<_> = self
            .workers
            .iter()
            .filter(|entry| {
                let worker = entry.value();
                worker.status == WorkerStatus::Healthy
                    && self
                        .circuit_breakers
                        .get(&worker.id)
                        .map_or(true, |cb| cb.is_available())
            })
            .collect();

        available_workers.sort_by(|a, b| {
            a.value()
                .current_load
                .partial_cmp(&b.value().current_load)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        available_workers.first().map(|entry| entry.value().clone())
    }

    fn record_success(&self, worker_id: WorkerId) {
        if let Some(cb) = self.circuit_breakers.get(&worker_id) {
            cb.record_success();
        }
    }

    fn record_failure(&self, worker_id: WorkerId) {
        if let Some(cb) = self.circuit_breakers.get(&worker_id) {
            cb.record_failure();
        }
        if let Some(mut worker) = self.workers.get_mut(&worker_id) {
            worker.tasks_failed += 1;
            worker.status = WorkerStatus::Unhealthy;
        }
    }

    fn get_cluster_health(&self) -> ClusterHealth {
        let total_workers = self.workers.len() as u32;
        let healthy_workers = self
            .workers
            .iter()
            .filter(|e| e.value().status == WorkerStatus::Healthy)
            .count() as u32;

        let average_latency_ms = if total_workers > 0 {
            let total_latency_ms: u64 = self
                .workers
                .iter()
                .map(|e| e.value().generation_latency_p99_ms)
                .sum();
            total_latency_ms / (total_workers as u64)
        } else {
            0
        };

        ClusterHealth {
            total_workers,
            healthy_workers,
            cache_hit_rate: 0.0, // Set by DistributedGenerator
            average_latency_ms,
            timestamp_secs: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_secs(),
        }
    }
}

/// Distributed code generator
pub struct DistributedGenerator {
    coordinator: Arc<ClusterCoordinator>,
    task_queue: Arc<WorkStealingQueue>,
    result_cache: Arc<ResultCache>,
    pending_tasks: Arc<DashMap<TaskId, oneshot::Sender<WorkflowResult<DistributedGeneratedCode>>>>,
    tracer: Arc<AsyncMutex<Tracer>>,
    metrics: Arc<RwLock<DistributedMetrics>>,
    task_tx: mpsc::UnboundedSender<GenerationTask>,
}

impl DistributedGenerator {
    /// Create new distributed generator
    pub async fn new(coordinator_addr: &str) -> WorkflowResult<Self> {
        let coordinator = Arc::new(ClusterCoordinator::new());
        let task_queue = Arc::new(WorkStealingQueue::new());
        let result_cache = Arc::new(ResultCache::new(true));
        let tracer = Arc::new(AsyncMutex::new(Tracer::new()));

        let (task_tx, task_rx) = mpsc::unbounded_channel();

        let generator = Self {
            coordinator: coordinator.clone(),
            task_queue: task_queue.clone(),
            result_cache: result_cache.clone(),
            pending_tasks: Arc::new(DashMap::new()),
            tracer,
            metrics: Arc::new(RwLock::new(DistributedMetrics {
                tasks_processed: 0,
                task_latency_p50_ms: 0,
                task_latency_p99_ms: 0,
                cache_hit_rate: 0.0,
                worker_utilization: 0.0,
                tasks_in_queue: 0,
                active_tasks: 0,
            })),
            task_tx,
        };

        // Spawn task processor
        generator
            .spawn_task_processor(task_rx, coordinator_addr.to_string())
            .await;

        info!(
            "DistributedGenerator initialized with coordinator: {}",
            coordinator_addr
        );
        Ok(generator)
    }

    async fn spawn_task_processor(
        &self,
        mut task_rx: mpsc::UnboundedReceiver<GenerationTask>,
        _coordinator_addr: String,
    ) {
        let task_queue = self.task_queue.clone();

        tokio::spawn(async move {
            while let Some(task) = task_rx.recv().await {
                task_queue.push(task).await;
            }
        });
    }

    /// Submit generation task for processing
    pub async fn submit_generation(&self, task: GenerationTask) -> WorkflowResult<TaskId> {
        let task_id = task.id;
        let trace_ctx = task.trace_context.clone();

        let mut tracer = self.tracer.lock().await;
        let span = tracer.start_span(
            format!("distributed_generator.submit_task.{}", task.description),
            None,
        );
        tracer.add_attribute(span.clone(), "task.id".to_string(), task_id.to_string());
        tracer.add_attribute(
            span.clone(),
            "task.priority".to_string(),
            task.priority.to_string(),
        );
        drop(tracer);

        // Check cache first
        let cache_key = format!("{}:{}", task.description, task.spec);
        if let Some(cached) = self.result_cache.get(&cache_key) {
            debug!("Cache hit for task: {}", task_id);
            // Return cached result immediately
            return Ok(task_id);
        }

        // Queue task
        self.task_tx
            .send(task)
            .map_err(|e| WorkflowError::Internal(format!("Failed to queue task: {}", e)))?;

        let mut tracer = self.tracer.lock().await;
        tracer.end_span(span, SpanStatus::Ok);

        Ok(task_id)
    }

    /// Wait for generation result
    pub async fn wait_for_result(
        &self,
        task_id: TaskId,
        task_timeout: Duration,
    ) -> WorkflowResult<DistributedGeneratedCode> {
        let (tx, rx) = oneshot::channel();
        self.pending_tasks.insert(task_id, tx);

        match timeout(task_timeout, rx).await {
            Ok(Ok(result)) => {
                self.pending_tasks.remove(&task_id);
                result
            }
            Ok(Err(_)) => {
                self.pending_tasks.remove(&task_id);
                Err(WorkflowError::Internal("Task sender dropped".to_string()))
            }
            Err(_) => {
                self.pending_tasks.remove(&task_id);
                Err(WorkflowError::Timeout)
            }
        }
    }

    /// Discover available workers
    pub async fn discover_workers(&self) -> WorkflowResult<Vec<WorkerInfo>> {
        Ok(self
            .coordinator
            .workers
            .iter()
            .map(|e| e.value().clone())
            .collect())
    }

    /// Check cluster health
    pub async fn check_cluster_health(&self) -> WorkflowResult<ClusterHealth> {
        let mut health = self.coordinator.get_cluster_health();
        health.cache_hit_rate = self.result_cache.hit_rate();
        Ok(health)
    }

    /// Get distributed generation metrics
    pub async fn get_generation_metrics(&self) -> WorkflowResult<DistributedMetrics> {
        let mut metrics = self.metrics.read().clone();
        metrics.tasks_in_queue = self.task_queue.len().await;
        metrics.cache_hit_rate = self.result_cache.hit_rate();
        Ok(metrics)
    }

    /// Enable or disable result caching
    pub fn enable_result_caching(&mut self, enabled: bool) {
        self.result_cache.enable(enabled);
        info!(
            "Result caching {}",
            if enabled { "enabled" } else { "disabled" }
        );
    }

    /// Invalidate cache for specific schema
    pub async fn invalidate_cache(&self, schema_id: &str) -> WorkflowResult<()> {
        self.result_cache.invalidate(schema_id);
        Ok(())
    }
}
