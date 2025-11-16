//! Hot Path Latency Testing with RDTSC
//!
//! Verifies all hot path operations complete within 8 ticks (Chatman Constant).
//! Uses RDTSC for cycle-accurate timing measurement.

#![feature(test)]
extern crate test;

use std::arch::x86_64::{__rdtscp, _mm_lfence, _mm_mfence};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

const MAX_TICKS: u64 = 8;
const WARMUP_ITERATIONS: usize = 10000;
const MEASUREMENT_ITERATIONS: usize = 100000;
const PERCENTILE_BUCKETS: usize = 1000;

/// RDTSC-based cycle counter for precise measurement
#[derive(Debug, Clone)]
pub struct CycleCounter {
    baseline: u64,
    measurements: Vec<u64>,
}

impl CycleCounter {
    pub fn new() -> Self {
        let baseline = unsafe {
            _mm_lfence();
            let mut aux: u32 = 0;
            let cycles = __rdtscp(&mut aux);
            _mm_lfence();
            cycles
        };

        Self {
            baseline,
            measurements: Vec::with_capacity(MEASUREMENT_ITERATIONS),
        }
    }

    /// Start timing a hot path operation
    #[inline(always)]
    pub fn start() -> u64 {
        unsafe {
            _mm_lfence();
            let mut aux: u32 = 0;
            let cycles = __rdtscp(&mut aux);
            _mm_lfence();
            cycles
        }
    }

    /// End timing and return cycle count
    #[inline(always)]
    pub fn end(start: u64) -> u64 {
        unsafe {
            _mm_lfence();
            let mut aux: u32 = 0;
            let end = __rdtscp(&mut aux);
            _mm_lfence();
            end.saturating_sub(start)
        }
    }

    /// Record a measurement
    pub fn record(&mut self, cycles: u64) {
        self.measurements.push(cycles);
    }

    /// Calculate statistics
    pub fn statistics(&self) -> LatencyStats {
        let mut sorted = self.measurements.clone();
        sorted.sort_unstable();

        let len = sorted.len();
        if len == 0 {
            return LatencyStats::default();
        }

        let mean = sorted.iter().sum::<u64>() / len as u64;
        let p50 = sorted[len * 50 / 100];
        let p95 = sorted[len * 95 / 100];
        let p99 = sorted[len * 99 / 100];
        let p999 = sorted[len * 999 / 1000];
        let min = sorted[0];
        let max = sorted[len - 1];

        // Calculate standard deviation
        let variance = sorted.iter()
            .map(|&x| {
                let diff = if x > mean { x - mean } else { mean - x };
                diff * diff
            })
            .sum::<u64>() / len as u64;
        let stddev = (variance as f64).sqrt() as u64;

        LatencyStats {
            mean,
            p50,
            p95,
            p99,
            p999,
            min,
            max,
            stddev,
            samples: len,
            violations: sorted.iter().filter(|&&x| x > MAX_TICKS).count(),
        }
    }
}

#[derive(Debug, Default)]
pub struct LatencyStats {
    pub mean: u64,
    pub p50: u64,
    pub p95: u64,
    pub p99: u64,
    pub p999: u64,
    pub min: u64,
    pub max: u64,
    pub stddev: u64,
    pub samples: usize,
    pub violations: usize,
}

/// Hot path operations to test
pub trait HotPathOperation: Send + Sync {
    fn execute(&self) -> u64;
    fn name(&self) -> &str;
}

/// Pattern dispatch operation
pub struct PatternDispatch {
    patterns: Vec<u64>,
    dispatch_table: HashMap<u64, fn() -> u64>,
}

impl PatternDispatch {
    pub fn new() -> Self {
        let mut dispatch_table = HashMap::new();
        for i in 0..256 {
            dispatch_table.insert(i, || i);
        }

        Self {
            patterns: (0..256).collect(),
            dispatch_table,
        }
    }
}

impl HotPathOperation for PatternDispatch {
    #[inline(always)]
    fn execute(&self) -> u64 {
        let pattern_id = self.patterns[42];
        let handler = self.dispatch_table.get(&pattern_id).unwrap();
        handler()
    }

    fn name(&self) -> &str {
        "pattern_dispatch"
    }
}

/// Guard evaluation operation
pub struct GuardEvaluation {
    guards: Vec<Box<dyn Fn(u64) -> bool + Send + Sync>>,
    value: AtomicU64,
}

impl GuardEvaluation {
    pub fn new() -> Self {
        let guards: Vec<Box<dyn Fn(u64) -> bool + Send + Sync>> = vec![
            Box::new(|x| x > 0),
            Box::new(|x| x < 100),
            Box::new(|x| x % 2 == 0),
        ];

        Self {
            guards,
            value: AtomicU64::new(42),
        }
    }
}

impl HotPathOperation for GuardEvaluation {
    #[inline(always)]
    fn execute(&self) -> u64 {
        let val = self.value.load(Ordering::Relaxed);
        let mut result = 0u64;
        for guard in &self.guards {
            if guard(val) {
                result += 1;
            }
        }
        result
    }

    fn name(&self) -> &str {
        "guard_evaluation"
    }
}

/// Receipt generation operation
pub struct ReceiptGeneration {
    counter: AtomicU64,
    timestamp: AtomicU64,
}

impl ReceiptGeneration {
    pub fn new() -> Self {
        Self {
            counter: AtomicU64::new(0),
            timestamp: AtomicU64::new(0),
        }
    }
}

impl HotPathOperation for ReceiptGeneration {
    #[inline(always)]
    fn execute(&self) -> u64 {
        let count = self.counter.fetch_add(1, Ordering::Relaxed);
        let ts = CycleCounter::start();
        self.timestamp.store(ts, Ordering::Relaxed);
        count ^ ts
    }

    fn name(&self) -> &str {
        "receipt_generation"
    }
}

/// Descriptor swap operation
pub struct DescriptorSwap {
    descriptors: Vec<AtomicU64>,
    current: AtomicU64,
}

impl DescriptorSwap {
    pub fn new() -> Self {
        let descriptors = (0..16).map(|i| AtomicU64::new(i)).collect();

        Self {
            descriptors,
            current: AtomicU64::new(0),
        }
    }
}

impl HotPathOperation for DescriptorSwap {
    #[inline(always)]
    fn execute(&self) -> u64 {
        let old = self.current.load(Ordering::Acquire);
        let new = (old + 1) % 16;
        self.current.store(new, Ordering::Release);
        self.descriptors[new as usize].load(Ordering::Acquire)
    }

    fn name(&self) -> &str {
        "descriptor_swap"
    }
}

/// Test harness for hot path operations
pub struct HotPathTestHarness {
    operations: Vec<Arc<dyn HotPathOperation>>,
    results: HashMap<String, LatencyStats>,
}

impl HotPathTestHarness {
    pub fn new() -> Self {
        let operations: Vec<Arc<dyn HotPathOperation>> = vec![
            Arc::new(PatternDispatch::new()),
            Arc::new(GuardEvaluation::new()),
            Arc::new(ReceiptGeneration::new()),
            Arc::new(DescriptorSwap::new()),
        ];

        Self {
            operations,
            results: HashMap::new(),
        }
    }

    /// Run warmup iterations
    fn warmup(&self, op: &dyn HotPathOperation) {
        for _ in 0..WARMUP_ITERATIONS {
            test::black_box(op.execute());
        }
    }

    /// Measure operation latency
    pub fn measure_operation(&mut self, op: Arc<dyn HotPathOperation>) -> LatencyStats {
        let mut counter = CycleCounter::new();

        // Warmup
        self.warmup(op.as_ref());

        // Measurement
        for _ in 0..MEASUREMENT_ITERATIONS {
            let start = CycleCounter::start();
            test::black_box(op.execute());
            let cycles = CycleCounter::end(start);
            counter.record(cycles);
        }

        counter.statistics()
    }

    /// Run all tests
    pub fn run_all_tests(&mut self) -> bool {
        let mut all_pass = true;

        println!("Hot Path Latency Test Results");
        println!("==============================");
        println!("Target: ≤{} ticks per operation", MAX_TICKS);
        println!();

        for op in self.operations.clone() {
            let name = op.name().to_string();
            let stats = self.measure_operation(op);

            println!("Operation: {}", name);
            println!("  Mean:      {} ticks", stats.mean);
            println!("  P50:       {} ticks", stats.p50);
            println!("  P95:       {} ticks", stats.p95);
            println!("  P99:       {} ticks", stats.p99);
            println!("  P99.9:     {} ticks", stats.p999);
            println!("  Min:       {} ticks", stats.min);
            println!("  Max:       {} ticks", stats.max);
            println!("  StdDev:    {} ticks", stats.stddev);
            println!("  Samples:   {}", stats.samples);

            let pass = stats.p99 <= MAX_TICKS;
            if pass {
                println!("  Result:    ✓ PASS (P99 ≤ {} ticks)", MAX_TICKS);
            } else {
                println!("  Result:    ✗ FAIL (P99 > {} ticks)", MAX_TICKS);
                println!("  Violations: {} samples exceeded threshold", stats.violations);
                all_pass = false;
            }
            println!();

            self.results.insert(name, stats);
        }

        all_pass
    }

    /// Generate regression baseline
    pub fn generate_baseline(&self) -> String {
        let mut baseline = String::from("# Hot Path Latency Baseline\n\n");

        for (name, stats) in &self.results {
            baseline.push_str(&format!("{}: p50={}, p99={}, max={}\n",
                name, stats.p50, stats.p99, stats.max));
        }

        baseline
    }

    /// Compare against baseline
    pub fn compare_baseline(&self, baseline: &HashMap<String, (u64, u64, u64)>) -> bool {
        let mut regression_detected = false;

        println!("Regression Detection");
        println!("===================");

        for (name, stats) in &self.results {
            if let Some(&(base_p50, base_p99, base_max)) = baseline.get(name) {
                let p50_delta = stats.p50 as i64 - base_p50 as i64;
                let p99_delta = stats.p99 as i64 - base_p99 as i64;

                println!("{}: P50 delta: {} ticks, P99 delta: {} ticks",
                    name, p50_delta, p99_delta);

                // Detect regression if P99 increased by more than 1 tick
                if p99_delta > 1 {
                    println!("  ⚠ Regression detected!");
                    regression_detected = true;
                }
            }
        }

        !regression_detected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_hot_path_operations() {
        let mut harness = HotPathTestHarness::new();
        assert!(harness.run_all_tests(), "Hot path latency requirements not met");
    }

    #[test]
    fn test_pattern_dispatch_latency() {
        let op = PatternDispatch::new();
        let mut counter = CycleCounter::new();

        // Warmup
        for _ in 0..WARMUP_ITERATIONS {
            test::black_box(op.execute());
        }

        // Measure
        for _ in 0..1000 {
            let start = CycleCounter::start();
            test::black_box(op.execute());
            let cycles = CycleCounter::end(start);
            counter.record(cycles);
        }

        let stats = counter.statistics();
        assert!(stats.p99 <= MAX_TICKS,
            "Pattern dispatch P99 latency {} exceeds {} ticks", stats.p99, MAX_TICKS);
    }

    #[test]
    fn test_guard_evaluation_latency() {
        let op = GuardEvaluation::new();
        let mut counter = CycleCounter::new();

        for _ in 0..WARMUP_ITERATIONS {
            test::black_box(op.execute());
        }

        for _ in 0..1000 {
            let start = CycleCounter::start();
            test::black_box(op.execute());
            let cycles = CycleCounter::end(start);
            counter.record(cycles);
        }

        let stats = counter.statistics();
        assert!(stats.p99 <= MAX_TICKS,
            "Guard evaluation P99 latency {} exceeds {} ticks", stats.p99, MAX_TICKS);
    }

    #[test]
    fn test_receipt_generation_latency() {
        let op = ReceiptGeneration::new();
        let mut counter = CycleCounter::new();

        for _ in 0..WARMUP_ITERATIONS {
            test::black_box(op.execute());
        }

        for _ in 0..1000 {
            let start = CycleCounter::start();
            test::black_box(op.execute());
            let cycles = CycleCounter::end(start);
            counter.record(cycles);
        }

        let stats = counter.statistics();
        assert!(stats.p99 <= MAX_TICKS,
            "Receipt generation P99 latency {} exceeds {} ticks", stats.p99, MAX_TICKS);
    }

    #[bench]
    fn bench_hot_path_pattern_dispatch(b: &mut test::Bencher) {
        let op = PatternDispatch::new();
        b.iter(|| {
            test::black_box(op.execute());
        });
    }

    #[bench]
    fn bench_hot_path_guard_evaluation(b: &mut test::Bencher) {
        let op = GuardEvaluation::new();
        b.iter(|| {
            test::black_box(op.execute());
        });
    }
}