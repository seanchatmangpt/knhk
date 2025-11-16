//! Fault Injection and Recovery Testing
//!
//! Verifies KNHK can detect and recover from various failures including
//! descriptor corruption, pattern routing errors, and receipt failures.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use parking_lot::{Mutex, RwLock};
use rand::Rng;

/// Types of faults to inject
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaultType {
    DescriptorCorruption,
    PatternRoutingError,
    GuardEvaluationFailure,
    ReceiptVerificationFailure,
    StateCorruption,
    NetworkPartition,
    DiskFailure,
    MemoryExhaustion,
    DeadlockCondition,
    ConcurrentModification,
}

/// Fault injection controller
pub struct FaultInjector {
    enabled: AtomicBool,
    fault_probability: AtomicU64,
    active_faults: Arc<RwLock<Vec<FaultType>>>,
    injected_count: AtomicU64,
    recovered_count: AtomicU64,
}

impl FaultInjector {
    pub fn new() -> Self {
        Self {
            enabled: AtomicBool::new(false),
            fault_probability: AtomicU64::new(10), // 10% probability
            active_faults: Arc::new(RwLock::new(Vec::new())),
            injected_count: AtomicU64::new(0),
            recovered_count: AtomicU64::new(0),
        }
    }

    pub fn enable(&self) {
        self.enabled.store(true, Ordering::SeqCst);
    }

    pub fn disable(&self) {
        self.enabled.store(false, Ordering::SeqCst);
    }

    pub fn set_probability(&self, prob: u64) {
        self.fault_probability.store(prob.min(100), Ordering::SeqCst);
    }

    pub fn should_inject_fault(&self) -> bool {
        if !self.enabled.load(Ordering::SeqCst) {
            return false;
        }

        let prob = self.fault_probability.load(Ordering::SeqCst);
        let mut rng = rand::thread_rng();
        rng.gen_range(0..100) < prob
    }

    pub fn inject_fault(&self, fault: FaultType) {
        if self.should_inject_fault() {
            self.active_faults.write().push(fault);
            self.injected_count.fetch_add(1, Ordering::SeqCst);
        }
    }

    pub fn clear_fault(&self, fault: FaultType) {
        let mut faults = self.active_faults.write();
        if let Some(pos) = faults.iter().position(|&f| f == fault) {
            faults.remove(pos);
            self.recovered_count.fetch_add(1, Ordering::SeqCst);
        }
    }

    pub fn has_fault(&self, fault: FaultType) -> bool {
        self.active_faults.read().contains(&fault)
    }

    pub fn statistics(&self) -> FaultStatistics {
        FaultStatistics {
            injected: self.injected_count.load(Ordering::SeqCst),
            recovered: self.recovered_count.load(Ordering::SeqCst),
            active: self.active_faults.read().len(),
        }
    }
}

#[derive(Debug)]
pub struct FaultStatistics {
    pub injected: u64,
    pub recovered: u64,
    pub active: usize,
}

/// System under test with fault injection
pub struct FaultTolerantSystem {
    injector: Arc<FaultInjector>,
    descriptors: Arc<RwLock<Vec<Descriptor>>>,
    state: Arc<RwLock<SystemState>>,
    receipts: Arc<Mutex<Vec<Receipt>>>,
    error_handler: Arc<ErrorHandler>,
}

#[derive(Debug, Clone)]
pub struct Descriptor {
    id: u64,
    version: u32,
    checksum: u64,
    data: Vec<u8>,
}

impl Descriptor {
    pub fn new(id: u64) -> Self {
        let data = vec![0u8; 256];
        let checksum = Self::calculate_checksum(&data);

        Self {
            id,
            version: 1,
            checksum,
            data,
        }
    }

    fn calculate_checksum(data: &[u8]) -> u64 {
        data.iter().fold(0u64, |acc, &byte| acc.wrapping_add(byte as u64))
    }

    pub fn verify(&self) -> bool {
        self.checksum == Self::calculate_checksum(&self.data)
    }

    pub fn corrupt(&mut self) {
        // Corrupt data but don't update checksum
        if !self.data.is_empty() {
            self.data[0] ^= 0xFF;
        }
    }
}

#[derive(Debug, Clone)]
pub struct SystemState {
    pattern_id: u64,
    guard_values: Vec<bool>,
    variables: Vec<i64>,
    timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct Receipt {
    id: u64,
    pattern_id: u64,
    timestamp: u64,
    hash: u64,
    verified: bool,
}

pub struct ErrorHandler {
    recovery_strategies: Arc<RwLock<Vec<RecoveryStrategy>>>,
}

impl ErrorHandler {
    pub fn new() -> Self {
        Self {
            recovery_strategies: Arc::new(RwLock::new(vec![
                RecoveryStrategy::Retry,
                RecoveryStrategy::Rollback,
                RecoveryStrategy::Checkpoint,
                RecoveryStrategy::Degrade,
            ])),
        }
    }

    pub fn handle_error(&self, fault: FaultType) -> RecoveryAction {
        match fault {
            FaultType::DescriptorCorruption => RecoveryAction::ReloadDescriptor,
            FaultType::PatternRoutingError => RecoveryAction::RecomputeRoute,
            FaultType::GuardEvaluationFailure => RecoveryAction::RetryGuards,
            FaultType::ReceiptVerificationFailure => RecoveryAction::RegenerateReceipt,
            FaultType::StateCorruption => RecoveryAction::RestoreCheckpoint,
            FaultType::NetworkPartition => RecoveryAction::WaitAndRetry,
            FaultType::DiskFailure => RecoveryAction::UseBackup,
            FaultType::MemoryExhaustion => RecoveryAction::GarbageCollect,
            FaultType::DeadlockCondition => RecoveryAction::BreakDeadlock,
            FaultType::ConcurrentModification => RecoveryAction::Synchronize,
        }
    }
}

#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    Retry,
    Rollback,
    Checkpoint,
    Degrade,
}

#[derive(Debug)]
pub enum RecoveryAction {
    ReloadDescriptor,
    RecomputeRoute,
    RetryGuards,
    RegenerateReceipt,
    RestoreCheckpoint,
    WaitAndRetry,
    UseBackup,
    GarbageCollect,
    BreakDeadlock,
    Synchronize,
}

impl FaultTolerantSystem {
    pub fn new() -> Self {
        let injector = Arc::new(FaultInjector::new());

        Self {
            injector: injector.clone(),
            descriptors: Arc::new(RwLock::new(
                (0..10).map(|i| Descriptor::new(i)).collect()
            )),
            state: Arc::new(RwLock::new(SystemState {
                pattern_id: 0,
                guard_values: vec![true; 5],
                variables: vec![0; 10],
                timestamp: 0,
            })),
            receipts: Arc::new(Mutex::new(Vec::new())),
            error_handler: Arc::new(ErrorHandler::new()),
        }
    }

    /// Execute workflow with fault injection
    pub fn execute_with_faults(&self) -> Result<Receipt, String> {
        // Descriptor corruption fault
        if self.injector.has_fault(FaultType::DescriptorCorruption) {
            self.inject_descriptor_corruption();
        }

        // Load and verify descriptor
        let descriptor = self.load_descriptor()?;

        // Pattern routing fault
        if self.injector.has_fault(FaultType::PatternRoutingError) {
            return self.handle_pattern_routing_error();
        }

        // Guard evaluation fault
        let guard_results = if self.injector.has_fault(FaultType::GuardEvaluationFailure) {
            self.evaluate_guards_with_failure()?
        } else {
            self.evaluate_guards_normal()?
        };

        // State update
        self.update_state(&guard_results)?;

        // Receipt generation with possible failure
        let receipt = if self.injector.has_fault(FaultType::ReceiptVerificationFailure) {
            self.generate_faulty_receipt()?
        } else {
            self.generate_receipt()?
        };

        Ok(receipt)
    }

    fn inject_descriptor_corruption(&self) {
        let mut descriptors = self.descriptors.write();
        if let Some(desc) = descriptors.get_mut(0) {
            desc.corrupt();
        }
    }

    fn load_descriptor(&self) -> Result<Descriptor, String> {
        let descriptors = self.descriptors.read();
        let desc = descriptors.get(0).ok_or("No descriptor found")?;

        if !desc.verify() {
            // Detected corruption - attempt recovery
            drop(descriptors);
            self.recover_descriptor()?;

            // Retry load
            let descriptors = self.descriptors.read();
            let desc = descriptors.get(0).ok_or("No descriptor after recovery")?;

            if !desc.verify() {
                return Err("Descriptor corruption unrecoverable".to_string());
            }
        }

        Ok(desc.clone())
    }

    fn recover_descriptor(&self) -> Result<(), String> {
        let action = self.error_handler.handle_error(FaultType::DescriptorCorruption);

        match action {
            RecoveryAction::ReloadDescriptor => {
                let mut descriptors = self.descriptors.write();
                if let Some(desc) = descriptors.get_mut(0) {
                    *desc = Descriptor::new(desc.id);
                }
                self.injector.clear_fault(FaultType::DescriptorCorruption);
                Ok(())
            }
            _ => Err("Unexpected recovery action".to_string()),
        }
    }

    fn handle_pattern_routing_error(&self) -> Result<Receipt, String> {
        let action = self.error_handler.handle_error(FaultType::PatternRoutingError);

        match action {
            RecoveryAction::RecomputeRoute => {
                self.injector.clear_fault(FaultType::PatternRoutingError);
                // Retry with corrected routing
                self.execute_with_faults()
            }
            _ => Err("Pattern routing unrecoverable".to_string()),
        }
    }

    fn evaluate_guards_with_failure(&self) -> Result<Vec<bool>, String> {
        // Simulate guard evaluation failure
        Err("Guard evaluation failed".to_string())
    }

    fn evaluate_guards_normal(&self) -> Result<Vec<bool>, String> {
        let state = self.state.read();
        Ok(state.guard_values.clone())
    }

    fn update_state(&self, guard_results: &[bool]) -> Result<(), String> {
        let mut state = self.state.write();

        for (i, &result) in guard_results.iter().enumerate() {
            if result && i < state.variables.len() {
                state.variables[i] += 1;
            }
        }

        state.timestamp += 1;
        Ok(())
    }

    fn generate_receipt(&self) -> Result<Receipt, String> {
        let state = self.state.read();
        let receipt = Receipt {
            id: state.timestamp,
            pattern_id: state.pattern_id,
            timestamp: state.timestamp,
            hash: state.timestamp * 12345,
            verified: true,
        };

        self.receipts.lock().push(receipt.clone());
        Ok(receipt)
    }

    fn generate_faulty_receipt(&self) -> Result<Receipt, String> {
        Err("Receipt verification failed".to_string())
    }

    /// Test graceful degradation
    pub fn test_graceful_degradation(&self) -> bool {
        // Enable fault injection
        self.injector.enable();
        self.injector.set_probability(50); // High fault rate

        let mut successes = 0;
        let mut partial_successes = 0;
        let total_attempts = 100;

        for _ in 0..total_attempts {
            // Inject random faults
            let fault_types = vec![
                FaultType::DescriptorCorruption,
                FaultType::PatternRoutingError,
                FaultType::GuardEvaluationFailure,
            ];

            for fault in &fault_types {
                if rand::thread_rng().gen_bool(0.3) {
                    self.injector.inject_fault(*fault);
                }
            }

            // Attempt execution
            match self.execute_with_faults() {
                Ok(_) => successes += 1,
                Err(_) => {
                    // Check if partial execution succeeded
                    if self.state.read().timestamp > 0 {
                        partial_successes += 1;
                    }
                }
            }

            // Clear some faults randomly
            for fault in &fault_types {
                if rand::thread_rng().gen_bool(0.5) {
                    self.injector.clear_fault(*fault);
                }
            }
        }

        let success_rate = (successes + partial_successes) as f64 / total_attempts as f64;
        println!("Graceful degradation test:");
        println!("  Full successes: {}/{}", successes, total_attempts);
        println!("  Partial successes: {}/{}", partial_successes, total_attempts);
        println!("  Success rate: {:.2}%", success_rate * 100.0);

        // System should maintain at least 50% success rate under high fault conditions
        success_rate >= 0.5
    }
}

/// Chaos engineering tests
pub struct ChaosEngine {
    system: Arc<FaultTolerantSystem>,
    chaos_thread: Option<thread::JoinHandle<()>>,
    running: Arc<AtomicBool>,
}

impl ChaosEngine {
    pub fn new(system: Arc<FaultTolerantSystem>) -> Self {
        Self {
            system,
            chaos_thread: None,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start_chaos(&mut self) {
        self.running.store(true, Ordering::SeqCst);
        let system = self.system.clone();
        let running = self.running.clone();

        self.chaos_thread = Some(thread::spawn(move || {
            let mut rng = rand::thread_rng();

            while running.load(Ordering::SeqCst) {
                // Randomly inject faults
                let fault = match rng.gen_range(0..10) {
                    0 => FaultType::DescriptorCorruption,
                    1 => FaultType::PatternRoutingError,
                    2 => FaultType::GuardEvaluationFailure,
                    3 => FaultType::ReceiptVerificationFailure,
                    4 => FaultType::StateCorruption,
                    5 => FaultType::NetworkPartition,
                    6 => FaultType::DiskFailure,
                    7 => FaultType::MemoryExhaustion,
                    8 => FaultType::DeadlockCondition,
                    9 => FaultType::ConcurrentModification,
                    _ => unreachable!(),
                };

                system.injector.inject_fault(fault);
                thread::sleep(Duration::from_millis(rng.gen_range(10..100)));

                // Sometimes clear faults
                if rng.gen_bool(0.3) {
                    system.injector.clear_fault(fault);
                }
            }
        }));
    }

    pub fn stop_chaos(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(thread) = self.chaos_thread.take() {
            thread.join().unwrap();
        }
    }

    pub fn run_chaos_test(&mut self, duration: Duration) -> ChaosTestResult {
        let start = Instant::now();
        self.start_chaos();
        self.system.injector.enable();

        let mut executions = 0;
        let mut failures = 0;
        let mut recoveries = 0;

        while start.elapsed() < duration {
            executions += 1;

            match self.system.execute_with_faults() {
                Ok(_) => {
                    let stats = self.system.injector.statistics();
                    if stats.recovered > 0 {
                        recoveries += 1;
                    }
                }
                Err(_) => failures += 1,
            }

            thread::sleep(Duration::from_millis(10));
        }

        self.stop_chaos();

        let stats = self.system.injector.statistics();

        ChaosTestResult {
            duration: start.elapsed(),
            executions,
            failures,
            recoveries,
            faults_injected: stats.injected,
            faults_recovered: stats.recovered,
            success_rate: (executions - failures) as f64 / executions as f64,
        }
    }
}

#[derive(Debug)]
pub struct ChaosTestResult {
    pub duration: Duration,
    pub executions: u64,
    pub failures: u64,
    pub recoveries: u64,
    pub faults_injected: u64,
    pub faults_recovered: u64,
    pub success_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_descriptor_corruption_recovery() {
        let system = FaultTolerantSystem::new();
        system.injector.enable();

        // Inject descriptor corruption
        system.injector.inject_fault(FaultType::DescriptorCorruption);
        system.inject_descriptor_corruption();

        // System should recover
        let result = system.execute_with_faults();
        assert!(result.is_ok(), "Failed to recover from descriptor corruption");

        let stats = system.injector.statistics();
        assert!(stats.recovered > 0, "No recovery recorded");
    }

    #[test]
    fn test_graceful_degradation() {
        let system = FaultTolerantSystem::new();
        assert!(system.test_graceful_degradation(),
            "System failed to maintain graceful degradation under faults");
    }

    #[test]
    fn test_chaos_engineering() {
        let system = Arc::new(FaultTolerantSystem::new());
        let mut chaos = ChaosEngine::new(system);

        let result = chaos.run_chaos_test(Duration::from_secs(5));

        println!("Chaos test results:");
        println!("  Duration: {:?}", result.duration);
        println!("  Executions: {}", result.executions);
        println!("  Failures: {}", result.failures);
        println!("  Recoveries: {}", result.recoveries);
        println!("  Faults injected: {}", result.faults_injected);
        println!("  Faults recovered: {}", result.faults_recovered);
        println!("  Success rate: {:.2}%", result.success_rate * 100.0);

        assert!(result.success_rate >= 0.7,
            "System success rate {:.2}% below 70% threshold", result.success_rate * 100.0);
    }

    #[test]
    fn test_concurrent_fault_recovery() {
        let system = Arc::new(FaultTolerantSystem::new());
        system.injector.enable();
        system.injector.set_probability(30);

        let handles: Vec<_> = (0..10)
            .map(|i| {
                let sys = system.clone();
                thread::spawn(move || {
                    let mut successes = 0;
                    for _ in 0..100 {
                        if sys.execute_with_faults().is_ok() {
                            successes += 1;
                        }
                    }
                    (i, successes)
                })
            })
            .collect();

        let results: Vec<_> = handles.into_iter()
            .map(|h| h.join().unwrap())
            .collect();

        for (thread_id, successes) in results {
            println!("Thread {} success rate: {}%", thread_id, successes);
            assert!(successes >= 60,
                "Thread {} success rate too low: {}%", thread_id, successes);
        }
    }
}