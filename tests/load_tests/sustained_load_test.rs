//! Sustained Load Testing
//!
//! Verifies system stability under sustained load including memory stability,
//! CPU predictability, and recovery from overload conditions.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use std::collections::VecDeque;
use parking_lot::{RwLock, Mutex};
use sysinfo::{System, SystemExt, ProcessExt};

const TARGET_OPS_PER_SEC: u64 = 1000;
const BURST_MULTIPLIER: u64 = 10;
const TEST_DURATION_SECS: u64 = 60; // 1 minute for quick tests, 86400 for 24h
const MEMORY_SAMPLE_INTERVAL: Duration = Duration::from_secs(1);
const CPU_SAMPLE_INTERVAL: Duration = Duration::from_millis(100);

/// Load test coordinator
pub struct LoadTestCoordinator {
    running: Arc<AtomicBool>,
    operations_completed: Arc<AtomicU64>,
    operations_failed: Arc<AtomicU64>,
    memory_samples: Arc<RwLock<Vec<MemorySample>>>,
    cpu_samples: Arc<RwLock<Vec<CpuSample>>>,
    latency_histogram: Arc<RwLock<LatencyHistogram>>,
    overload_events: Arc<RwLock<Vec<OverloadEvent>>>,
}

#[derive(Debug, Clone)]
pub struct MemorySample {
    pub timestamp: Instant,
    pub rss_bytes: u64,
    pub heap_bytes: u64,
    pub gc_count: u64,
}

#[derive(Debug, Clone)]
pub struct CpuSample {
    pub timestamp: Instant,
    pub usage_percent: f32,
    pub user_time: f64,
    pub system_time: f64,
}

#[derive(Debug)]
pub struct LatencyHistogram {
    buckets: Vec<u64>,
    bucket_width_us: u64,
    total_samples: u64,
}

impl LatencyHistogram {
    pub fn new() -> Self {
        Self {
            buckets: vec![0; 1000], // 1000 buckets
            bucket_width_us: 10,    // 10us per bucket
            total_samples: 0,
        }
    }

    pub fn record(&mut self, latency: Duration) {
        let us = latency.as_micros() as u64;
        let bucket = (us / self.bucket_width_us).min(self.buckets.len() as u64 - 1) as usize;
        self.buckets[bucket] += 1;
        self.total_samples += 1;
    }

    pub fn percentile(&self, p: f64) -> Duration {
        if self.total_samples == 0 {
            return Duration::from_micros(0);
        }

        let target_count = (self.total_samples as f64 * p / 100.0) as u64;
        let mut count = 0u64;

        for (bucket, &frequency) in self.buckets.iter().enumerate() {
            count += frequency;
            if count >= target_count {
                let us = (bucket as u64 * self.bucket_width_us) + (self.bucket_width_us / 2);
                return Duration::from_micros(us);
            }
        }

        Duration::from_micros(self.buckets.len() as u64 * self.bucket_width_us)
    }
}

#[derive(Debug, Clone)]
pub struct OverloadEvent {
    pub timestamp: Instant,
    pub queue_depth: u64,
    pub dropped_operations: u64,
    pub recovery_time: Option<Duration>,
}

impl LoadTestCoordinator {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            operations_completed: Arc::new(AtomicU64::new(0)),
            operations_failed: Arc::new(AtomicU64::new(0)),
            memory_samples: Arc::new(RwLock::new(Vec::new())),
            cpu_samples: Arc::new(RwLock::new(Vec::new())),
            latency_histogram: Arc::new(RwLock::new(LatencyHistogram::new())),
            overload_events: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Run sustained load test
    pub fn run_sustained_load(&self, duration: Duration) -> LoadTestResult {
        self.running.store(true, Ordering::Release);
        let start = Instant::now();

        // Start monitoring threads
        let memory_monitor = self.start_memory_monitor();
        let cpu_monitor = self.start_cpu_monitor();

        // Start load generation threads
        let load_generators = self.start_load_generators(TARGET_OPS_PER_SEC);

        // Wait for test duration
        while start.elapsed() < duration && self.running.load(Ordering::Acquire) {
            thread::sleep(Duration::from_millis(100));

            // Check for system stability
            if !self.check_stability() {
                println!("Warning: System instability detected at {:?}", start.elapsed());
            }
        }

        // Stop load generation
        self.running.store(false, Ordering::Release);

        // Wait for threads to complete
        for handle in load_generators {
            handle.join().unwrap();
        }

        memory_monitor.join().unwrap();
        cpu_monitor.join().unwrap();

        // Analyze results
        self.analyze_results(start.elapsed())
    }

    /// Run burst capacity test
    pub fn run_burst_test(&self) -> BurstTestResult {
        let normal_load = TARGET_OPS_PER_SEC;
        let burst_load = normal_load * BURST_MULTIPLIER;
        let burst_duration = Duration::from_secs(10);

        // Start with normal load
        self.running.store(true, Ordering::Release);
        let generators = self.start_load_generators(normal_load);

        thread::sleep(Duration::from_secs(5)); // Warm up

        // Record baseline
        let baseline_ops = self.operations_completed.load(Ordering::Acquire);
        let burst_start = Instant::now();

        // Apply burst load
        let burst_generators = self.start_load_generators(burst_load - normal_load);

        thread::sleep(burst_duration);

        // Return to normal load
        self.running.store(false, Ordering::Release);
        for handle in burst_generators {
            handle.join().unwrap();
        }

        let burst_ops = self.operations_completed.load(Ordering::Acquire) - baseline_ops;
        let actual_duration = burst_start.elapsed();

        // Continue normal load for recovery observation
        thread::sleep(Duration::from_secs(5));

        // Stop all generators
        for handle in generators {
            handle.join().unwrap();
        }

        BurstTestResult {
            burst_throughput: burst_ops as f64 / actual_duration.as_secs_f64(),
            burst_duration: actual_duration,
            recovery_time: self.measure_recovery_time(),
            dropped_operations: self.operations_failed.load(Ordering::Acquire),
        }
    }

    /// Test memory stability
    pub fn test_memory_stability(&self, duration: Duration) -> MemoryStabilityResult {
        self.running.store(true, Ordering::Release);
        let start = Instant::now();

        let memory_monitor = self.start_detailed_memory_monitor();
        let generators = self.start_load_generators(TARGET_OPS_PER_SEC);

        // Run load test
        thread::sleep(duration);

        self.running.store(false, Ordering::Release);
        for handle in generators {
            handle.join().unwrap();
        }
        memory_monitor.join().unwrap();

        // Analyze memory patterns
        let samples = self.memory_samples.read();
        if samples.is_empty() {
            return MemoryStabilityResult::default();
        }

        let initial_memory = samples[0].rss_bytes;
        let final_memory = samples.last().unwrap().rss_bytes;
        let peak_memory = samples.iter().map(|s| s.rss_bytes).max().unwrap();

        // Calculate leak rate (bytes per second)
        let leak_rate = if final_memory > initial_memory {
            (final_memory - initial_memory) as f64 / duration.as_secs_f64()
        } else {
            0.0
        };

        // Check for memory growth pattern
        let is_stable = leak_rate < 1024.0; // Less than 1KB/s growth

        MemoryStabilityResult {
            initial_memory,
            final_memory,
            peak_memory,
            leak_rate,
            is_stable,
            gc_count: samples.last().unwrap().gc_count,
        }
    }

    /// Test CPU predictability
    pub fn test_cpu_predictability(&self) -> CpuPredictabilityResult {
        self.running.store(true, Ordering::Release);

        let cpu_monitor = self.start_detailed_cpu_monitor();
        let generators = self.start_load_generators(TARGET_OPS_PER_SEC);

        // Run for 30 seconds
        thread::sleep(Duration::from_secs(30));

        self.running.store(false, Ordering::Release);
        for handle in generators {
            handle.join().unwrap();
        }
        cpu_monitor.join().unwrap();

        // Analyze CPU usage patterns
        let samples = self.cpu_samples.read();
        if samples.is_empty() {
            return CpuPredictabilityResult::default();
        }

        let cpu_values: Vec<f32> = samples.iter().map(|s| s.usage_percent).collect();
        let mean_cpu = cpu_values.iter().sum::<f32>() / cpu_values.len() as f32;

        // Calculate standard deviation
        let variance = cpu_values.iter()
            .map(|&x| {
                let diff = x - mean_cpu;
                diff * diff
            })
            .sum::<f32>() / cpu_values.len() as f32;
        let stddev = variance.sqrt();

        // Calculate jitter (max deviation from mean)
        let max_cpu = cpu_values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let min_cpu = cpu_values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let jitter = max_cpu - min_cpu;

        CpuPredictabilityResult {
            mean_cpu_percent: mean_cpu,
            stddev_cpu: stddev,
            jitter_percent: jitter,
            is_predictable: stddev < 10.0 && jitter < 20.0, // Low variation
        }
    }

    /// Test recovery from overload
    pub fn test_overload_recovery(&self) -> OverloadRecoveryResult {
        self.running.store(true, Ordering::Release);

        // Start with normal load
        let generators = self.start_load_generators(TARGET_OPS_PER_SEC);
        thread::sleep(Duration::from_secs(5));

        // Measure baseline throughput
        let baseline_start = self.operations_completed.load(Ordering::Acquire);
        thread::sleep(Duration::from_secs(5));
        let baseline_end = self.operations_completed.load(Ordering::Acquire);
        let baseline_throughput = (baseline_end - baseline_start) as f64 / 5.0;

        // Apply overload (5x normal)
        let overload_start = Instant::now();
        let overload_generators = self.start_load_generators(TARGET_OPS_PER_SEC * 5);

        // Run overload for 30 seconds
        thread::sleep(Duration::from_secs(30));

        // Stop overload
        self.running.store(false, Ordering::Release);
        for handle in overload_generators {
            handle.join().unwrap();
        }

        // Allow recovery
        self.running.store(true, Ordering::Release);
        let recovery_start = Instant::now();
        let mut recovered = false;
        let mut recovery_time = Duration::from_secs(0);

        // Monitor recovery for up to 60 seconds
        for _ in 0..60 {
            thread::sleep(Duration::from_secs(1));

            let current_start = self.operations_completed.load(Ordering::Acquire);
            thread::sleep(Duration::from_secs(1));
            let current_end = self.operations_completed.load(Ordering::Acquire);
            let current_throughput = (current_end - current_start) as f64;

            if current_throughput >= baseline_throughput * 0.9 {
                recovered = true;
                recovery_time = recovery_start.elapsed();
                break;
            }
        }

        // Stop normal load
        self.running.store(false, Ordering::Release);
        for handle in generators {
            handle.join().unwrap();
        }

        OverloadRecoveryResult {
            overload_duration: Duration::from_secs(30),
            recovery_time,
            recovered,
            baseline_throughput,
            dropped_during_overload: self.operations_failed.load(Ordering::Acquire),
        }
    }

    fn start_load_generators(&self, ops_per_sec: u64) -> Vec<thread::JoinHandle<()>> {
        let num_threads = 4;
        let ops_per_thread = ops_per_sec / num_threads;
        let interval = Duration::from_nanos(1_000_000_000 / ops_per_thread.max(1));

        (0..num_threads)
            .map(|_| {
                let running = self.running.clone();
                let completed = self.operations_completed.clone();
                let failed = self.operations_failed.clone();
                let histogram = self.latency_histogram.clone();

                thread::spawn(move || {
                    while running.load(Ordering::Acquire) {
                        let start = Instant::now();

                        // Simulate operation
                        if Self::execute_operation() {
                            completed.fetch_add(1, Ordering::Relaxed);
                        } else {
                            failed.fetch_add(1, Ordering::Relaxed);
                        }

                        let latency = start.elapsed();
                        histogram.write().record(latency);

                        // Rate limiting
                        if let Some(sleep_time) = interval.checked_sub(latency) {
                            thread::sleep(sleep_time);
                        }
                    }
                })
            })
            .collect()
    }

    fn execute_operation() -> bool {
        // Simulate work with 99% success rate
        use rand::Rng;
        rand::thread_rng().gen_bool(0.99)
    }

    fn start_memory_monitor(&self) -> thread::JoinHandle<()> {
        let running = self.running.clone();
        let samples = self.memory_samples.clone();

        thread::spawn(move || {
            let mut sys = System::new_all();

            while running.load(Ordering::Acquire) {
                sys.refresh_all();

                if let Some(process) = sys.process(sysinfo::get_current_pid().unwrap()) {
                    let sample = MemorySample {
                        timestamp: Instant::now(),
                        rss_bytes: process.memory() * 1024, // Convert KB to bytes
                        heap_bytes: 0, // Would need allocator instrumentation
                        gc_count: 0,   // Would need GC hooks
                    };
                    samples.write().push(sample);
                }

                thread::sleep(MEMORY_SAMPLE_INTERVAL);
            }
        })
    }

    fn start_detailed_memory_monitor(&self) -> thread::JoinHandle<()> {
        self.start_memory_monitor() // Simplified for example
    }

    fn start_cpu_monitor(&self) -> thread::JoinHandle<()> {
        let running = self.running.clone();
        let samples = self.cpu_samples.clone();

        thread::spawn(move || {
            let mut sys = System::new_all();

            while running.load(Ordering::Acquire) {
                sys.refresh_all();

                if let Some(process) = sys.process(sysinfo::get_current_pid().unwrap()) {
                    let sample = CpuSample {
                        timestamp: Instant::now(),
                        usage_percent: process.cpu_usage(),
                        user_time: 0.0,   // Would need more detailed metrics
                        system_time: 0.0,
                    };
                    samples.write().push(sample);
                }

                thread::sleep(CPU_SAMPLE_INTERVAL);
            }
        })
    }

    fn start_detailed_cpu_monitor(&self) -> thread::JoinHandle<()> {
        self.start_cpu_monitor() // Simplified for example
    }

    fn check_stability(&self) -> bool {
        // Check various stability metrics
        let failed = self.operations_failed.load(Ordering::Acquire);
        let completed = self.operations_completed.load(Ordering::Acquire);

        if completed == 0 {
            return false;
        }

        let error_rate = failed as f64 / (completed + failed) as f64;
        error_rate < 0.01 // Less than 1% error rate
    }

    fn measure_recovery_time(&self) -> Duration {
        // Simplified - would track actual recovery
        Duration::from_secs(5)
    }

    fn analyze_results(&self, duration: Duration) -> LoadTestResult {
        let completed = self.operations_completed.load(Ordering::Acquire);
        let failed = self.operations_failed.load(Ordering::Acquire);
        let histogram = self.latency_histogram.read();

        LoadTestResult {
            duration,
            total_operations: completed + failed,
            successful_operations: completed,
            failed_operations: failed,
            throughput: completed as f64 / duration.as_secs_f64(),
            error_rate: failed as f64 / (completed + failed).max(1) as f64,
            p50_latency: histogram.percentile(50.0),
            p95_latency: histogram.percentile(95.0),
            p99_latency: histogram.percentile(99.0),
            memory_stable: self.check_memory_stability(),
            cpu_stable: self.check_cpu_stability(),
        }
    }

    fn check_memory_stability(&self) -> bool {
        let samples = self.memory_samples.read();
        if samples.len() < 2 {
            return true;
        }

        let first = &samples[0];
        let last = samples.last().unwrap();

        // Check if memory growth is reasonable (< 10% increase)
        (last.rss_bytes as f64) < (first.rss_bytes as f64 * 1.1)
    }

    fn check_cpu_stability(&self) -> bool {
        let samples = self.cpu_samples.read();
        if samples.is_empty() {
            return true;
        }

        let avg_cpu: f32 = samples.iter().map(|s| s.usage_percent).sum::<f32>() / samples.len() as f32;
        avg_cpu < 80.0 // CPU usage below 80%
    }
}

#[derive(Debug)]
pub struct LoadTestResult {
    pub duration: Duration,
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub throughput: f64,
    pub error_rate: f64,
    pub p50_latency: Duration,
    pub p95_latency: Duration,
    pub p99_latency: Duration,
    pub memory_stable: bool,
    pub cpu_stable: bool,
}

#[derive(Debug)]
pub struct BurstTestResult {
    pub burst_throughput: f64,
    pub burst_duration: Duration,
    pub recovery_time: Duration,
    pub dropped_operations: u64,
}

#[derive(Debug, Default)]
pub struct MemoryStabilityResult {
    pub initial_memory: u64,
    pub final_memory: u64,
    pub peak_memory: u64,
    pub leak_rate: f64,
    pub is_stable: bool,
    pub gc_count: u64,
}

#[derive(Debug, Default)]
pub struct CpuPredictabilityResult {
    pub mean_cpu_percent: f32,
    pub stddev_cpu: f32,
    pub jitter_percent: f32,
    pub is_predictable: bool,
}

#[derive(Debug)]
pub struct OverloadRecoveryResult {
    pub overload_duration: Duration,
    pub recovery_time: Duration,
    pub recovered: bool,
    pub baseline_throughput: f64,
    pub dropped_during_overload: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sustained_load() {
        let coordinator = LoadTestCoordinator::new();
        let result = coordinator.run_sustained_load(Duration::from_secs(10));

        assert!(result.throughput > 500.0, "Throughput too low");
        assert!(result.error_rate < 0.01, "Error rate too high");
        assert!(result.memory_stable, "Memory instability detected");
        assert!(result.cpu_stable, "CPU instability detected");

        println!("Sustained Load Test Results:");
        println!("  Duration: {:?}", result.duration);
        println!("  Throughput: {:.2} ops/sec", result.throughput);
        println!("  Error rate: {:.4}%", result.error_rate * 100.0);
        println!("  P50 latency: {:?}", result.p50_latency);
        println!("  P95 latency: {:?}", result.p95_latency);
        println!("  P99 latency: {:?}", result.p99_latency);
    }

    #[test]
    fn test_burst_capacity() {
        let coordinator = LoadTestCoordinator::new();
        let result = coordinator.run_burst_test();

        assert!(result.burst_throughput > TARGET_OPS_PER_SEC as f64 * 5.0,
            "Burst capacity insufficient");

        println!("Burst Test Results:");
        println!("  Burst throughput: {:.2} ops/sec", result.burst_throughput);
        println!("  Recovery time: {:?}", result.recovery_time);
        println!("  Dropped operations: {}", result.dropped_operations);
    }

    #[test]
    fn test_memory_stability() {
        let coordinator = LoadTestCoordinator::new();
        let result = coordinator.test_memory_stability(Duration::from_secs(30));

        assert!(result.is_stable, "Memory leak detected");
        assert!(result.leak_rate < 1024.0, "Memory growth rate too high");

        println!("Memory Stability Test Results:");
        println!("  Initial memory: {} MB", result.initial_memory / (1024 * 1024));
        println!("  Final memory: {} MB", result.final_memory / (1024 * 1024));
        println!("  Peak memory: {} MB", result.peak_memory / (1024 * 1024));
        println!("  Leak rate: {:.2} bytes/sec", result.leak_rate);
    }

    #[test]
    fn test_cpu_predictability() {
        let coordinator = LoadTestCoordinator::new();
        let result = coordinator.test_cpu_predictability();

        assert!(result.is_predictable, "CPU usage unpredictable");

        println!("CPU Predictability Test Results:");
        println!("  Mean CPU: {:.2}%", result.mean_cpu_percent);
        println!("  StdDev: {:.2}%", result.stddev_cpu);
        println!("  Jitter: {:.2}%", result.jitter_percent);
    }

    #[test]
    fn test_overload_recovery() {
        let coordinator = LoadTestCoordinator::new();
        let result = coordinator.test_overload_recovery();

        assert!(result.recovered, "System did not recover from overload");
        assert!(result.recovery_time < Duration::from_secs(60),
            "Recovery time too long");

        println!("Overload Recovery Test Results:");
        println!("  Recovery time: {:?}", result.recovery_time);
        println!("  Baseline throughput: {:.2} ops/sec", result.baseline_throughput);
        println!("  Dropped during overload: {}", result.dropped_during_overload);
    }
}