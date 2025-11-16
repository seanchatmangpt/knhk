//! Concurrent Execution and Isolation Testing
//!
//! Verifies multi-workflow isolation, concurrent receipt generation,
//! race condition detection, and fairness.

use std::sync::{Arc, Barrier};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use std::collections::{HashMap, HashSet, VecDeque};
use parking_lot::{RwLock, Mutex};
use proptest::prelude::*;
use crossbeam::channel::{self, Sender, Receiver};
use dashmap::DashMap;

/// Isolated workflow executor
pub struct IsolatedWorkflowExecutor {
    workflows: Arc<DashMap<u64, WorkflowInstance>>,
    descriptor_pool: Arc<DescriptorPool>,
    receipt_generator: Arc<ConcurrentReceiptGenerator>,
    isolation_monitor: Arc<IsolationMonitor>,
}

#[derive(Debug, Clone)]
pub struct WorkflowInstance {
    pub id: u64,
    pub pattern_id: AtomicU64,
    pub state: Arc<RwLock<WorkflowState>>,
    pub receipts: Arc<RwLock<Vec<Receipt>>>,
    pub descriptor_version: AtomicU64,
}

#[derive(Debug, Clone)]
pub struct WorkflowState {
    pub variables: HashMap<String, i64>,
    pub guard_values: Vec<bool>,
    pub timestamp: u64,
    pub lock_holder: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct Receipt {
    pub id: u64,
    pub workflow_id: u64,
    pub pattern_id: u64,
    pub timestamp: u64,
    pub state_hash: u64,
}

/// Descriptor pool with concurrent swap support
pub struct DescriptorPool {
    descriptors: Arc<RwLock<Vec<Descriptor>>>,
    active_swaps: Arc<DashMap<u64, SwapOperation>>,
    swap_counter: AtomicU64,
}

#[derive(Debug, Clone)]
pub struct Descriptor {
    pub id: u64,
    pub version: AtomicU64,
    pub data: Arc<RwLock<Vec<u8>>>,
    pub readers: AtomicU64,
}

#[derive(Debug)]
pub struct SwapOperation {
    pub id: u64,
    pub old_version: u64,
    pub new_version: u64,
    pub workflow_id: u64,
    pub completed: AtomicBool,
}

impl DescriptorPool {
    pub fn new(count: usize) -> Self {
        let descriptors = (0..count)
            .map(|i| Descriptor {
                id: i as u64,
                version: AtomicU64::new(1),
                data: Arc::new(RwLock::new(vec![0u8; 256])),
                readers: AtomicU64::new(0),
            })
            .collect();

        Self {
            descriptors: Arc::new(RwLock::new(descriptors)),
            active_swaps: Arc::new(DashMap::new()),
            swap_counter: AtomicU64::new(0),
        }
    }

    pub fn atomic_swap(&self, descriptor_id: u64, workflow_id: u64) -> Result<u64, String> {
        let swap_id = self.swap_counter.fetch_add(1, Ordering::SeqCst);

        // Start swap operation
        let descriptors = self.descriptors.read();
        let descriptor = descriptors.get(descriptor_id as usize)
            .ok_or("Invalid descriptor ID")?;

        // Wait for readers to finish
        while descriptor.readers.load(Ordering::Acquire) > 0 {
            thread::yield_now();
        }

        let old_version = descriptor.version.load(Ordering::Acquire);
        let new_version = old_version + 1;

        // Record swap operation
        self.active_swaps.insert(swap_id, SwapOperation {
            id: swap_id,
            old_version,
            new_version,
            workflow_id,
            completed: AtomicBool::new(false),
        });

        // Perform atomic swap
        descriptor.version.store(new_version, Ordering::Release);

        // Mark swap as completed
        if let Some(swap) = self.active_swaps.get(&swap_id) {
            swap.completed.store(true, Ordering::Release);
        }

        Ok(new_version)
    }

    pub fn read_descriptor(&self, descriptor_id: u64) -> Result<Vec<u8>, String> {
        let descriptors = self.descriptors.read();
        let descriptor = descriptors.get(descriptor_id as usize)
            .ok_or("Invalid descriptor ID")?;

        descriptor.readers.fetch_add(1, Ordering::AcqRel);
        let data = descriptor.data.read().clone();
        descriptor.readers.fetch_sub(1, Ordering::AcqRel);

        Ok(data)
    }
}

/// Concurrent receipt generator with collision detection
pub struct ConcurrentReceiptGenerator {
    counter: AtomicU64,
    collision_detector: Arc<Mutex<HashSet<u64>>>,
    generation_times: Arc<RwLock<Vec<Duration>>>,
}

impl ConcurrentReceiptGenerator {
    pub fn new() -> Self {
        Self {
            counter: AtomicU64::new(0),
            collision_detector: Arc::new(Mutex::new(HashSet::new())),
            generation_times: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn generate_receipt(&self, workflow_id: u64, pattern_id: u64) -> Result<Receipt, String> {
        let start = Instant::now();

        // Generate unique ID atomically
        let id = self.counter.fetch_add(1, Ordering::SeqCst);

        // Check for collisions (should never happen with atomic counter)
        {
            let mut detector = self.collision_detector.lock();
            if !detector.insert(id) {
                return Err(format!("Receipt ID collision detected: {}", id));
            }
        }

        let receipt = Receipt {
            id,
            workflow_id,
            pattern_id,
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            state_hash: self.calculate_state_hash(workflow_id, pattern_id),
        };

        // Record generation time
        self.generation_times.write().push(start.elapsed());

        Ok(receipt)
    }

    fn calculate_state_hash(&self, workflow_id: u64, pattern_id: u64) -> u64 {
        // Deterministic hash calculation
        workflow_id.wrapping_mul(31).wrapping_add(pattern_id)
    }

    pub fn verify_no_collisions(&self) -> bool {
        let detector = self.collision_detector.lock();
        let expected_count = self.counter.load(Ordering::SeqCst) as usize;
        detector.len() == expected_count
    }

    pub fn get_generation_stats(&self) -> GenerationStats {
        let times = self.generation_times.read();
        if times.is_empty() {
            return GenerationStats::default();
        }

        let mut sorted_times: Vec<_> = times.iter().map(|d| d.as_nanos() as u64).collect();
        sorted_times.sort_unstable();

        GenerationStats {
            count: sorted_times.len(),
            mean_ns: sorted_times.iter().sum::<u64>() / sorted_times.len() as u64,
            p50_ns: sorted_times[sorted_times.len() / 2],
            p99_ns: sorted_times[sorted_times.len() * 99 / 100],
            max_ns: *sorted_times.last().unwrap(),
        }
    }
}

#[derive(Debug, Default)]
pub struct GenerationStats {
    pub count: usize,
    pub mean_ns: u64,
    pub p50_ns: u64,
    pub p99_ns: u64,
    pub max_ns: u64,
}

/// Isolation monitor for detecting violations
pub struct IsolationMonitor {
    violations: Arc<RwLock<Vec<IsolationViolation>>>,
    monitoring: AtomicBool,
}

#[derive(Debug, Clone)]
pub struct IsolationViolation {
    pub timestamp: u64,
    pub workflow1: u64,
    pub workflow2: u64,
    pub violation_type: ViolationType,
    pub details: String,
}

#[derive(Debug, Clone)]
pub enum ViolationType {
    StateLeakage,
    ReceiptCrossContamination,
    DescriptorRaceCondition,
    DeadlockDetected,
    UnfairScheduling,
}

impl IsolationMonitor {
    pub fn new() -> Self {
        Self {
            violations: Arc::new(RwLock::new(Vec::new())),
            monitoring: AtomicBool::new(true),
        }
    }

    pub fn report_violation(&self, violation: IsolationViolation) {
        if self.monitoring.load(Ordering::Acquire) {
            self.violations.write().push(violation);
        }
    }

    pub fn check_isolation(&self, workflow1: &WorkflowInstance, workflow2: &WorkflowInstance) -> bool {
        // Check for state leakage
        let state1 = workflow1.state.read();
        let state2 = workflow2.state.read();

        // Workflows should have independent state
        for (key, _) in &state1.variables {
            if state2.variables.contains_key(key) && workflow1.id != workflow2.id {
                self.report_violation(IsolationViolation {
                    timestamp: chrono::Utc::now().timestamp_millis() as u64,
                    workflow1: workflow1.id,
                    workflow2: workflow2.id,
                    violation_type: ViolationType::StateLeakage,
                    details: format!("Shared state variable: {}", key),
                });
                return false;
            }
        }

        true
    }

    pub fn get_violations(&self) -> Vec<IsolationViolation> {
        self.violations.read().clone()
    }
}

impl IsolatedWorkflowExecutor {
    pub fn new(max_workflows: usize) -> Self {
        Self {
            workflows: Arc::new(DashMap::new()),
            descriptor_pool: Arc::new(DescriptorPool::new(100)),
            receipt_generator: Arc::new(ConcurrentReceiptGenerator::new()),
            isolation_monitor: Arc::new(IsolationMonitor::new()),
        }
    }

    pub fn create_workflow(&self, id: u64) -> WorkflowInstance {
        let workflow = WorkflowInstance {
            id,
            pattern_id: AtomicU64::new(0),
            state: Arc::new(RwLock::new(WorkflowState {
                variables: HashMap::new(),
                guard_values: vec![true; 5],
                timestamp: 0,
                lock_holder: None,
            })),
            receipts: Arc::new(RwLock::new(Vec::new())),
            descriptor_version: AtomicU64::new(1),
        };

        self.workflows.insert(id, workflow.clone());
        workflow
    }

    pub fn execute_step(&self, workflow_id: u64) -> Result<Receipt, String> {
        let workflow = self.workflows.get(&workflow_id)
            .ok_or("Workflow not found")?;

        // Pattern transition
        let pattern_id = workflow.pattern_id.fetch_add(1, Ordering::AcqRel);

        // Generate receipt
        let receipt = self.receipt_generator.generate_receipt(workflow_id, pattern_id)?;

        // Store receipt
        workflow.receipts.write().push(receipt.clone());

        // Descriptor swap
        if pattern_id % 10 == 0 {
            self.descriptor_pool.atomic_swap(pattern_id % 100, workflow_id)?;
        }

        Ok(receipt)
    }

    pub fn run_concurrent_workflows(&self, num_workflows: usize, steps_per_workflow: usize) -> ConcurrencyTestResult {
        let barrier = Arc::new(Barrier::new(num_workflows));
        let start = Instant::now();

        let handles: Vec<_> = (0..num_workflows)
            .map(|i| {
                let executor = self.clone();
                let barrier = barrier.clone();
                let workflow_id = i as u64;

                thread::spawn(move || {
                    executor.create_workflow(workflow_id);
                    barrier.wait();

                    let mut successes = 0;
                    let mut failures = 0;

                    for _ in 0..steps_per_workflow {
                        match executor.execute_step(workflow_id) {
                            Ok(_) => successes += 1,
                            Err(_) => failures += 1,
                        }
                    }

                    (workflow_id, successes, failures)
                })
            })
            .collect();

        let results: Vec<_> = handles.into_iter()
            .map(|h| h.join().unwrap())
            .collect();

        let duration = start.elapsed();
        let total_successes: usize = results.iter().map(|(_, s, _)| *s).sum();
        let total_failures: usize = results.iter().map(|(_, _, f)| *f).sum();

        // Check for isolation violations
        let workflows: Vec<_> = self.workflows.iter()
            .map(|entry| entry.value().clone())
            .collect();

        for i in 0..workflows.len() {
            for j in i + 1..workflows.len() {
                self.isolation_monitor.check_isolation(&workflows[i], &workflows[j]);
            }
        }

        ConcurrencyTestResult {
            duration,
            num_workflows,
            steps_per_workflow,
            total_successes,
            total_failures,
            violations: self.isolation_monitor.get_violations(),
            no_collisions: self.receipt_generator.verify_no_collisions(),
            generation_stats: self.receipt_generator.get_generation_stats(),
        }
    }
}

impl Clone for IsolatedWorkflowExecutor {
    fn clone(&self) -> Self {
        Self {
            workflows: self.workflows.clone(),
            descriptor_pool: self.descriptor_pool.clone(),
            receipt_generator: self.receipt_generator.clone(),
            isolation_monitor: self.isolation_monitor.clone(),
        }
    }
}

#[derive(Debug)]
pub struct ConcurrencyTestResult {
    pub duration: Duration,
    pub num_workflows: usize,
    pub steps_per_workflow: usize,
    pub total_successes: usize,
    pub total_failures: usize,
    pub violations: Vec<IsolationViolation>,
    pub no_collisions: bool,
    pub generation_stats: GenerationStats,
}

/// Race condition detector using property-based testing
pub struct RaceConditionDetector {
    execution_traces: Arc<RwLock<Vec<ExecutionTrace>>>,
}

#[derive(Debug, Clone)]
pub struct ExecutionTrace {
    pub thread_id: u64,
    pub operation: Operation,
    pub timestamp: u64,
    pub before_state: u64,
    pub after_state: u64,
}

#[derive(Debug, Clone)]
pub enum Operation {
    Read(String),
    Write(String),
    Swap(u64),
    Lock(String),
    Unlock(String),
}

impl RaceConditionDetector {
    pub fn new() -> Self {
        Self {
            execution_traces: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn record(&self, trace: ExecutionTrace) {
        self.execution_traces.write().push(trace);
    }

    pub fn analyze_traces(&self) -> Vec<RaceCondition> {
        let traces = self.execution_traces.read();
        let mut races = Vec::new();

        for i in 0..traces.len() {
            for j in i + 1..traces.len() {
                if self.is_race_condition(&traces[i], &traces[j]) {
                    races.push(RaceCondition {
                        trace1: traces[i].clone(),
                        trace2: traces[j].clone(),
                        conflict_type: self.classify_conflict(&traces[i], &traces[j]),
                    });
                }
            }
        }

        races
    }

    fn is_race_condition(&self, t1: &ExecutionTrace, t2: &ExecutionTrace) -> bool {
        // Different threads accessing same resource
        if t1.thread_id == t2.thread_id {
            return false;
        }

        // Check for conflicting operations
        match (&t1.operation, &t2.operation) {
            (Operation::Write(r1), Operation::Write(r2)) => r1 == r2,
            (Operation::Write(r1), Operation::Read(r2)) => r1 == r2,
            (Operation::Read(r1), Operation::Write(r2)) => r1 == r2,
            _ => false,
        }
    }

    fn classify_conflict(&self, t1: &ExecutionTrace, t2: &ExecutionTrace) -> ConflictType {
        match (&t1.operation, &t2.operation) {
            (Operation::Write(_), Operation::Write(_)) => ConflictType::WriteWrite,
            (Operation::Write(_), Operation::Read(_)) => ConflictType::WriteRead,
            (Operation::Read(_), Operation::Write(_)) => ConflictType::ReadWrite,
            _ => ConflictType::Unknown,
        }
    }
}

#[derive(Debug)]
pub struct RaceCondition {
    pub trace1: ExecutionTrace,
    pub trace2: ExecutionTrace,
    pub conflict_type: ConflictType,
}

#[derive(Debug)]
pub enum ConflictType {
    WriteWrite,
    WriteRead,
    ReadWrite,
    Unknown,
}

/// Fairness verifier
pub struct FairnessVerifier {
    workflow_executions: Arc<DashMap<u64, WorkflowExecutionStats>>,
}

#[derive(Debug, Clone)]
pub struct WorkflowExecutionStats {
    pub executions: AtomicU64,
    pub wait_times: Arc<RwLock<Vec<Duration>>>,
    pub starvation_count: AtomicU64,
}

impl FairnessVerifier {
    pub fn new() -> Self {
        Self {
            workflow_executions: Arc::new(DashMap::new()),
        }
    }

    pub fn record_execution(&self, workflow_id: u64, wait_time: Duration) {
        let entry = self.workflow_executions.entry(workflow_id)
            .or_insert_with(|| WorkflowExecutionStats {
                executions: AtomicU64::new(0),
                wait_times: Arc::new(RwLock::new(Vec::new())),
                starvation_count: AtomicU64::new(0),
            });

        entry.executions.fetch_add(1, Ordering::SeqCst);
        entry.wait_times.write().push(wait_time);

        // Check for starvation (wait time > 1 second)
        if wait_time > Duration::from_secs(1) {
            entry.starvation_count.fetch_add(1, Ordering::SeqCst);
        }
    }

    pub fn verify_fairness(&self) -> FairnessReport {
        let mut min_executions = u64::MAX;
        let mut max_executions = 0u64;
        let mut total_starvation = 0u64;

        for entry in self.workflow_executions.iter() {
            let executions = entry.executions.load(Ordering::SeqCst);
            min_executions = min_executions.min(executions);
            max_executions = max_executions.max(executions);
            total_starvation += entry.starvation_count.load(Ordering::SeqCst);
        }

        let fairness_ratio = if max_executions > 0 {
            min_executions as f64 / max_executions as f64
        } else {
            1.0
        };

        FairnessReport {
            min_executions,
            max_executions,
            fairness_ratio,
            total_starvation,
            is_fair: fairness_ratio >= 0.8 && total_starvation == 0,
        }
    }
}

#[derive(Debug)]
pub struct FairnessReport {
    pub min_executions: u64,
    pub max_executions: u64,
    pub fairness_ratio: f64,
    pub total_starvation: u64,
    pub is_fair: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_isolation() {
        let executor = IsolatedWorkflowExecutor::new(100);
        let result = executor.run_concurrent_workflows(50, 100);

        assert_eq!(result.violations.len(), 0, "Isolation violations detected");
        assert!(result.no_collisions, "Receipt ID collisions detected");
        assert_eq!(result.total_failures, 0, "Workflow execution failures");

        println!("Concurrent execution test:");
        println!("  Workflows: {}", result.num_workflows);
        println!("  Steps per workflow: {}", result.steps_per_workflow);
        println!("  Total successes: {}", result.total_successes);
        println!("  Duration: {:?}", result.duration);
        println!("  Mean receipt generation: {} ns", result.generation_stats.mean_ns);
    }

    #[test]
    fn test_race_condition_detection() {
        let detector = RaceConditionDetector::new();
        let detector_ref = Arc::new(detector);

        // Simulate concurrent traces
        let handles: Vec<_> = (0..10)
            .map(|i| {
                let detector = detector_ref.clone();
                thread::spawn(move || {
                    for j in 0..10 {
                        detector.record(ExecutionTrace {
                            thread_id: i,
                            operation: if j % 2 == 0 {
                                Operation::Write("shared_resource".to_string())
                            } else {
                                Operation::Read("shared_resource".to_string())
                            },
                            timestamp: i * 10 + j,
                            before_state: j as u64,
                            after_state: (j + 1) as u64,
                        });
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        let races = detector_ref.analyze_traces();
        println!("Detected {} potential race conditions", races.len());
    }

    #[test]
    fn test_fairness() {
        let verifier = FairnessVerifier::new();

        // Simulate fair scheduling
        for workflow_id in 0..10 {
            for _ in 0..100 {
                verifier.record_execution(workflow_id, Duration::from_millis(10));
            }
        }

        let report = verifier.verify_fairness();
        assert!(report.is_fair, "Unfair workflow scheduling detected");
        assert_eq!(report.total_starvation, 0, "Workflow starvation detected");

        println!("Fairness report:");
        println!("  Min executions: {}", report.min_executions);
        println!("  Max executions: {}", report.max_executions);
        println!("  Fairness ratio: {:.2}", report.fairness_ratio);
    }

    proptest! {
        #[test]
        fn prop_no_receipt_collisions(
            num_workflows in 1usize..100,
            steps in 1usize..1000
        ) {
            let executor = IsolatedWorkflowExecutor::new(100);
            let result = executor.run_concurrent_workflows(num_workflows, steps);
            prop_assert!(result.no_collisions);
        }

        #[test]
        fn prop_isolation_maintained(
            workflow_pairs in prop::collection::vec((0u64..100, 0u64..100), 1..50)
        ) {
            let executor = IsolatedWorkflowExecutor::new(100);

            for (w1, w2) in workflow_pairs {
                if w1 != w2 {
                    let workflow1 = executor.create_workflow(w1);
                    let workflow2 = executor.create_workflow(w2);

                    prop_assert!(executor.isolation_monitor.check_isolation(&workflow1, &workflow2));
                }
            }
        }
    }
}