// kernel/degradation.rs - Fallback patterns and graceful degradation
// Phase 3: Handling cascading failures and recovery paths
// DOCTRINE: Covenant 5 (Latency Is Our Currency) - Degrade gracefully to preserve latency

use dashmap::DashMap;
use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

/// Degradation strategy for system overload
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DegradationStrategy {
    /// Drop non-essential work
    LoadShedding,
    /// Process at reduced rate
    RateLimiting,
    /// Batch work for later processing
    Buffering,
    /// Use simplified processing
    FeatureReduction,
    /// Switch to approximate algorithms
    ApproximateComputation,
    /// Pass through without processing
    PassThrough,
}

/// Degradation level with thresholds
#[derive(Debug, Clone)]
pub struct DegradationLevel {
    pub name: String,
    pub threshold: f64,
    pub strategies: Vec<DegradationStrategy>,
    pub priority_cutoff: u8,
    pub sampling_rate: f64,
}

/// Degradation state for a component
#[derive(Debug, Clone)]
pub struct DegradationState {
    pub component: String,
    pub current_level: usize,
    pub active_strategies: Vec<DegradationStrategy>,
    pub degraded_since: Option<Instant>,
    pub recovery_attempts: u32,
}

/// Main degradation manager
pub struct DegradationManager {
    levels: Vec<DegradationLevel>,
    component_states: Arc<DashMap<String, DegradationState>>,
    metrics: Arc<DegradationMetrics>,
    recovery_policy: Arc<RecoveryPolicy>,
    circuit_breakers: Arc<DashMap<String, CircuitBreaker>>,
}

#[derive(Debug)]
struct DegradationMetrics {
    degradation_events: AtomicU64,
    recovery_events: AtomicU64,
    current_degraded_components: AtomicUsize,
    work_items_shed: AtomicU64,
    work_items_deferred: AtomicU64,
    feature_reductions: AtomicU64,
}

impl DegradationManager {
    pub fn new() -> Self {
        let levels = vec![
            DegradationLevel {
                name: "Normal".to_string(),
                threshold: 0.0,
                strategies: vec![],
                priority_cutoff: 0,
                sampling_rate: 1.0,
            },
            DegradationLevel {
                name: "Light".to_string(),
                threshold: 0.7,
                strategies: vec![DegradationStrategy::RateLimiting],
                priority_cutoff: 3,
                sampling_rate: 0.9,
            },
            DegradationLevel {
                name: "Moderate".to_string(),
                threshold: 0.85,
                strategies: vec![
                    DegradationStrategy::LoadShedding,
                    DegradationStrategy::FeatureReduction,
                ],
                priority_cutoff: 5,
                sampling_rate: 0.7,
            },
            DegradationLevel {
                name: "Severe".to_string(),
                threshold: 0.95,
                strategies: vec![
                    DegradationStrategy::LoadShedding,
                    DegradationStrategy::ApproximateComputation,
                    DegradationStrategy::Buffering,
                ],
                priority_cutoff: 7,
                sampling_rate: 0.5,
            },
            DegradationLevel {
                name: "Critical".to_string(),
                threshold: 0.99,
                strategies: vec![DegradationStrategy::PassThrough],
                priority_cutoff: 9,
                sampling_rate: 0.1,
            },
        ];

        Self {
            levels,
            component_states: Arc::new(DashMap::new()),
            metrics: Arc::new(DegradationMetrics {
                degradation_events: AtomicU64::new(0),
                recovery_events: AtomicU64::new(0),
                current_degraded_components: AtomicUsize::new(0),
                work_items_shed: AtomicU64::new(0),
                work_items_deferred: AtomicU64::new(0),
                feature_reductions: AtomicU64::new(0),
            }),
            recovery_policy: Arc::new(RecoveryPolicy::default()),
            circuit_breakers: Arc::new(DashMap::new()),
        }
    }

    /// Update degradation level based on load
    pub fn update_degradation(&self, component: String, load: f64) -> DegradationDecision {
        let new_level = self.calculate_level(load);
        let mut decision = DegradationDecision::default();

        // Get or create component state
        let mut entry = self
            .component_states
            .entry(component.clone())
            .or_insert_with(|| DegradationState {
                component: component.clone(),
                current_level: 0,
                active_strategies: vec![],
                degraded_since: None,
                recovery_attempts: 0,
            });

        let old_level = entry.current_level;

        if new_level != old_level {
            entry.current_level = new_level;
            entry.active_strategies = self.levels[new_level].strategies.clone();

            if new_level > old_level {
                // Degrading
                entry.degraded_since = Some(Instant::now());
                self.metrics
                    .degradation_events
                    .fetch_add(1, Ordering::Relaxed);
                self.metrics
                    .current_degraded_components
                    .fetch_add(1, Ordering::Relaxed);

                decision.action = DegradationAction::Degrade;
                decision.strategies = entry.active_strategies.clone();

                warn!(
                    "Component {} degraded to level {}: {}",
                    component, new_level, self.levels[new_level].name
                );
            } else {
                // Recovering
                entry.recovery_attempts += 1;
                self.metrics.recovery_events.fetch_add(1, Ordering::Relaxed);

                if new_level == 0 {
                    entry.degraded_since = None;
                    self.metrics
                        .current_degraded_components
                        .fetch_sub(1, Ordering::Relaxed);
                }

                decision.action = DegradationAction::Recover;

                info!(
                    "Component {} recovered to level {}: {}",
                    component, new_level, self.levels[new_level].name
                );
            }
        } else {
            decision.action = DegradationAction::Maintain;
        }

        decision.level = new_level;
        decision.priority_cutoff = self.levels[new_level].priority_cutoff;
        decision.sampling_rate = self.levels[new_level].sampling_rate;

        decision
    }

    fn calculate_level(&self, load: f64) -> usize {
        for (i, level) in self.levels.iter().enumerate().rev() {
            if load >= level.threshold {
                return i;
            }
        }
        0
    }

    /// Apply degradation strategy to work item
    pub fn apply_degradation(&self, component: &str, work: WorkItem) -> DegradedWork {
        let state = self.component_states.get(component).map(|s| s.clone());

        let strategies = state
            .as_ref()
            .map(|s| s.active_strategies.clone())
            .unwrap_or_default();

        let mut degraded = DegradedWork {
            original: work.clone(),
            applied_strategies: vec![],
            should_process: true,
            deferred: false,
            simplified: false,
        };

        for strategy in strategies {
            match strategy {
                DegradationStrategy::LoadShedding => {
                    if work.priority
                        < state
                            .as_ref()
                            .map(|s| self.levels[s.current_level].priority_cutoff)
                            .unwrap_or(0)
                    {
                        degraded.should_process = false;
                        self.metrics.work_items_shed.fetch_add(1, Ordering::Relaxed);
                    }
                }
                DegradationStrategy::RateLimiting => {
                    let sampling_rate = state
                        .as_ref()
                        .map(|s| self.levels[s.current_level].sampling_rate)
                        .unwrap_or(1.0);

                    if rand::random::<f64>() > sampling_rate {
                        degraded.should_process = false;
                    }
                }
                DegradationStrategy::Buffering => {
                    degraded.deferred = true;
                    self.metrics
                        .work_items_deferred
                        .fetch_add(1, Ordering::Relaxed);
                }
                DegradationStrategy::FeatureReduction => {
                    degraded.simplified = true;
                    self.metrics
                        .feature_reductions
                        .fetch_add(1, Ordering::Relaxed);
                }
                DegradationStrategy::ApproximateComputation => {
                    degraded.simplified = true;
                }
                DegradationStrategy::PassThrough => {
                    degraded.should_process = false;
                    degraded.simplified = true;
                }
            }

            degraded.applied_strategies.push(strategy);
        }

        degraded
    }

    /// Check if component should accept work
    pub fn should_accept_work(&self, component: &str, priority: u8) -> bool {
        if let Some(state) = self.component_states.get(component) {
            priority >= self.levels[state.current_level].priority_cutoff
        } else {
            true
        }
    }

    /// Get circuit breaker for component
    pub fn get_circuit_breaker(&self, component: &str) -> Arc<CircuitBreaker> {
        if let Some(cb_ref) = self.circuit_breakers.get(component) {
            Arc::new(cb_ref.value().clone())
        } else {
            let cb = CircuitBreaker::new(
                component.to_string(),
                CircuitBreakerConfig::default(),
            );
            self.circuit_breakers.insert(component.to_string(), cb.clone());
            Arc::new(cb)
        }
    }

    pub fn get_metrics(&self) -> DegradationStatistics {
        DegradationStatistics {
            degradation_events: self.metrics.degradation_events.load(Ordering::Relaxed),
            recovery_events: self.metrics.recovery_events.load(Ordering::Relaxed),
            current_degraded_components: self
                .metrics
                .current_degraded_components
                .load(Ordering::Relaxed),
            work_items_shed: self.metrics.work_items_shed.load(Ordering::Relaxed),
            work_items_deferred: self.metrics.work_items_deferred.load(Ordering::Relaxed),
            feature_reductions: self.metrics.feature_reductions.load(Ordering::Relaxed),
        }
    }
}

/// Work item for processing
#[derive(Debug, Clone)]
pub struct WorkItem {
    pub id: String,
    pub priority: u8,
    pub payload: Vec<u8>,
    pub features: Vec<String>,
    pub deadline: Option<Instant>,
}

/// Degraded work item with applied strategies
#[derive(Debug, Clone)]
pub struct DegradedWork {
    pub original: WorkItem,
    pub applied_strategies: Vec<DegradationStrategy>,
    pub should_process: bool,
    pub deferred: bool,
    pub simplified: bool,
}

/// Decision from degradation manager
#[derive(Debug, Clone, Default)]
pub struct DegradationDecision {
    pub action: DegradationAction,
    pub level: usize,
    pub strategies: Vec<DegradationStrategy>,
    pub priority_cutoff: u8,
    pub sampling_rate: f64,
}

#[derive(Debug, Clone, Default)]
pub enum DegradationAction {
    #[default]
    Maintain,
    Degrade,
    Recover,
}

/// Recovery policy for degraded components
#[derive(Debug, Clone)]
pub struct RecoveryPolicy {
    pub min_stable_duration: Duration,
    pub recovery_rate: f64,
    pub max_recovery_attempts: u32,
    pub backoff_multiplier: f64,
}

impl Default for RecoveryPolicy {
    fn default() -> Self {
        Self {
            min_stable_duration: Duration::from_secs(30),
            recovery_rate: 0.1,
            max_recovery_attempts: 5,
            backoff_multiplier: 2.0,
        }
    }
}

/// Circuit breaker for cascading failure prevention
#[derive(Clone)]
pub struct CircuitBreaker {
    name: String,
    state: Arc<Mutex<CircuitBreakerState>>,
    config: CircuitBreakerConfig,
    failure_count: Arc<AtomicU64>,
    success_count: Arc<AtomicU64>,
    last_failure: Arc<Mutex<Option<Instant>>>,
    last_state_change: Arc<Mutex<Instant>>,
}

#[derive(Debug, Clone, Copy)]
enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u64,
    pub success_threshold: u64,
    pub timeout: Duration,
    pub half_open_max_calls: u64,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout: Duration::from_secs(60),
            half_open_max_calls: 3,
        }
    }
}

impl CircuitBreaker {
    pub fn new(name: String, config: CircuitBreakerConfig) -> Self {
        Self {
            name,
            state: Arc::new(Mutex::new(CircuitBreakerState::Closed)),
            config,
            failure_count: Arc::new(AtomicU64::new(0)),
            success_count: Arc::new(AtomicU64::new(0)),
            last_failure: Arc::new(Mutex::new(None)),
            last_state_change: Arc::new(Mutex::new(Instant::now())),
        }
    }

    pub fn call<F, T, E>(&self, f: F) -> Result<T, E>
    where
        F: FnOnce() -> Result<T, E>,
        E: From<String>,
    {
        let current_state = *self.state.lock();

        match current_state {
            CircuitBreakerState::Open => {
                // Check if timeout has elapsed
                if self.should_attempt_reset() {
                    self.transition_to(CircuitBreakerState::HalfOpen);
                    self.attempt_call(f)
                } else {
                    Err(E::from(format!("Circuit breaker {} is open", self.name)))
                }
            }
            CircuitBreakerState::HalfOpen => self.attempt_call(f),
            CircuitBreakerState::Closed => self.attempt_call(f),
        }
    }

    fn attempt_call<F, T, E>(&self, f: F) -> Result<T, E>
    where
        F: FnOnce() -> Result<T, E>,
        E: From<String>,
    {
        match f() {
            Ok(result) => {
                self.record_success();
                Ok(result)
            }
            Err(error) => {
                self.record_failure();
                Err(error)
            }
        }
    }

    fn record_success(&self) {
        self.success_count.fetch_add(1, Ordering::Relaxed);
        let current_state = *self.state.lock();

        match current_state {
            CircuitBreakerState::HalfOpen => {
                if self.success_count.load(Ordering::Relaxed) >= self.config.success_threshold {
                    self.transition_to(CircuitBreakerState::Closed);
                    self.reset_counts();
                }
            }
            CircuitBreakerState::Closed => {
                // Reset failure count on success in closed state
                self.failure_count.store(0, Ordering::Relaxed);
            }
            _ => {}
        }
    }

    fn record_failure(&self) {
        self.failure_count.fetch_add(1, Ordering::Relaxed);
        *self.last_failure.lock() = Some(Instant::now());

        let current_state = *self.state.lock();

        match current_state {
            CircuitBreakerState::Closed => {
                if self.failure_count.load(Ordering::Relaxed) >= self.config.failure_threshold {
                    self.transition_to(CircuitBreakerState::Open);
                    warn!(
                        "Circuit breaker {} opened after {} failures",
                        self.name, self.config.failure_threshold
                    );
                }
            }
            CircuitBreakerState::HalfOpen => {
                self.transition_to(CircuitBreakerState::Open);
                warn!(
                    "Circuit breaker {} reopened after failure in half-open state",
                    self.name
                );
            }
            _ => {}
        }
    }

    fn should_attempt_reset(&self) -> bool {
        if let Some(last_failure) = *self.last_failure.lock() {
            last_failure.elapsed() >= self.config.timeout
        } else {
            false
        }
    }

    fn transition_to(&self, new_state: CircuitBreakerState) {
        let mut state = self.state.lock();
        *state = new_state;
        *self.last_state_change.lock() = Instant::now();

        debug!(
            "Circuit breaker {} transitioned to {:?}",
            self.name, new_state
        );
    }

    fn reset_counts(&self) {
        self.failure_count.store(0, Ordering::Relaxed);
        self.success_count.store(0, Ordering::Relaxed);
    }

    pub fn is_open(&self) -> bool {
        matches!(*self.state.lock(), CircuitBreakerState::Open)
    }

    pub fn get_state(&self) -> String {
        format!("{:?}", *self.state.lock())
    }
}

/// Error recovery strategies
pub struct ErrorRecovery {
    strategies: Vec<Box<dyn RecoveryStrategy>>,
    max_retries: u32,
    backoff: ExponentialBackoff,
}

trait RecoveryStrategy: Send + Sync {
    fn attempt_recovery(&self, error: &str) -> Result<(), String>;
}

struct ExponentialBackoff {
    base: Duration,
    max: Duration,
    multiplier: f64,
    current: Arc<Mutex<Duration>>,
}

impl ErrorRecovery {
    pub fn new(max_retries: u32) -> Self {
        Self {
            strategies: vec![],
            max_retries,
            backoff: ExponentialBackoff {
                base: Duration::from_millis(100),
                max: Duration::from_secs(30),
                multiplier: 2.0,
                current: Arc::new(Mutex::new(Duration::from_millis(100))),
            },
        }
    }

    pub fn add_strategy(&mut self, strategy: Box<dyn RecoveryStrategy>) {
        self.strategies.push(strategy);
    }

    pub fn attempt_recovery(&self, error: &str, attempt: u32) -> Result<(), String> {
        if attempt >= self.max_retries {
            return Err(format!("Max retries ({}) exceeded", self.max_retries));
        }

        // Wait with backoff
        let wait_time = self.backoff.get_wait_time();
        std::thread::sleep(wait_time);

        // Try recovery strategies
        for strategy in &self.strategies {
            if let Ok(()) = strategy.attempt_recovery(error) {
                self.backoff.reset();
                return Ok(());
            }
        }

        self.backoff.increase();
        Err("All recovery strategies failed".to_string())
    }
}

impl ExponentialBackoff {
    fn get_wait_time(&self) -> Duration {
        *self.current.lock()
    }

    fn increase(&self) {
        let mut current = self.current.lock();
        let new_duration = current.mul_f64(self.multiplier).min(self.max);
        *current = new_duration;
    }

    fn reset(&self) {
        *self.current.lock() = self.base;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DegradationStatistics {
    pub degradation_events: u64,
    pub recovery_events: u64,
    pub current_degraded_components: usize,
    pub work_items_shed: u64,
    pub work_items_deferred: u64,
    pub feature_reductions: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_degradation_levels() {
        let manager = DegradationManager::new();

        let decision = manager.update_degradation("test".to_string(), 0.5);
        assert_eq!(decision.level, 0); // Normal

        let decision = manager.update_degradation("test".to_string(), 0.75);
        assert_eq!(decision.level, 1); // Light

        let decision = manager.update_degradation("test".to_string(), 0.90);
        assert_eq!(decision.level, 2); // Moderate

        let decision = manager.update_degradation("test".to_string(), 0.96);
        assert_eq!(decision.level, 3); // Severe
    }

    #[test]
    fn test_work_degradation() {
        let manager = DegradationManager::new();

        // Set component to moderate degradation
        manager.update_degradation("test".to_string(), 0.87);

        let work = WorkItem {
            id: "work-1".to_string(),
            priority: 2,
            payload: vec![1, 2, 3],
            features: vec!["feature1".to_string()],
            deadline: None,
        };

        let degraded = manager.apply_degradation("test", work.clone());
        assert!(!degraded.should_process); // Priority too low
        assert!(!degraded.applied_strategies.is_empty());

        let high_priority_work = WorkItem {
            id: "work-2".to_string(),
            priority: 8,
            payload: vec![1, 2, 3],
            features: vec!["feature1".to_string()],
            deadline: None,
        };

        let degraded = manager.apply_degradation("test", high_priority_work);
        assert!(degraded.should_process); // High priority passes
    }

    #[test]
    fn test_circuit_breaker() {
        let breaker = CircuitBreaker::new(
            "test".to_string(),
            CircuitBreakerConfig {
                failure_threshold: 3,
                success_threshold: 2,
                timeout: Duration::from_millis(100),
                half_open_max_calls: 2,
            },
        );

        // Should succeed while closed
        let result: Result<(), String> = breaker.call(|| Ok(()));
        assert!(result.is_ok());

        // Record failures
        for _ in 0..3 {
            let _: Result<(), String> = breaker.call(|| Err("error".to_string()));
        }

        // Should be open now
        assert!(breaker.is_open());

        // Wait for timeout
        std::thread::sleep(Duration::from_millis(150));

        // Should transition to half-open and allow attempt
        let result: Result<(), String> = breaker.call(|| Ok(()));
        assert!(result.is_ok());
    }

    #[test]
    fn test_exponential_backoff() {
        let backoff = ExponentialBackoff {
            base: Duration::from_millis(10),
            max: Duration::from_millis(100),
            multiplier: 2.0,
            current: Arc::new(Mutex::new(Duration::from_millis(10))),
        };

        assert_eq!(backoff.get_wait_time(), Duration::from_millis(10));

        backoff.increase();
        assert_eq!(backoff.get_wait_time(), Duration::from_millis(20));

        backoff.increase();
        assert_eq!(backoff.get_wait_time(), Duration::from_millis(40));

        backoff.increase();
        assert_eq!(backoff.get_wait_time(), Duration::from_millis(80));

        backoff.increase();
        assert_eq!(backoff.get_wait_time(), Duration::from_millis(100)); // Capped at max

        backoff.reset();
        assert_eq!(backoff.get_wait_time(), Duration::from_millis(10));
    }
}
