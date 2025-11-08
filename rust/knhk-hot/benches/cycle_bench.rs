// knhk-hot/benches/cycle_bench.rs
// Cycle-accurate benchmarking framework inspired by simdjson
// Measures: cycles/op, instructions/op, throughput (ops/sec)
//
// Based on simdjson's benchmark.h and KNHK's target of ≤8 ticks for hot path

use std::hint::black_box;
use std::time::{Duration, Instant};

#[cfg(target_os = "linux")]
use perf_event::{Builder, Counter, Group};

/// Performance counters for detailed analysis
#[derive(Debug, Clone, Default)]
pub struct PerfCounters {
    pub cycles: u64,
    pub instructions: u64,
    pub cache_refs: u64,
    pub cache_misses: u64,
    pub branches: u64,
    pub branch_misses: u64,
}

impl PerfCounters {
    pub fn ipc(&self) -> f64 {
        if self.cycles == 0 {
            0.0
        } else {
            self.instructions as f64 / self.cycles as f64
        }
    }

    pub fn cache_miss_rate(&self) -> f64 {
        if self.cache_refs == 0 {
            0.0
        } else {
            self.cache_misses as f64 / self.cache_refs as f64
        }
    }

    pub fn branch_miss_rate(&self) -> f64 {
        if self.branches == 0 {
            0.0
        } else {
            self.branch_misses as f64 / self.branches as f64
        }
    }
}

/// Benchmark result aggregation (similar to simdjson's event_aggregate)
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub name: String,
    pub iterations: usize,
    pub total_time: Duration,
    pub best_time: Duration,
    pub worst_time: Duration,
    pub median_time: Duration,
    pub best_counters: PerfCounters,
    pub median_counters: PerfCounters,
}

impl BenchmarkResult {
    pub fn new(name: String) -> Self {
        Self {
            name,
            iterations: 0,
            total_time: Duration::ZERO,
            best_time: Duration::MAX,
            worst_time: Duration::ZERO,
            median_time: Duration::ZERO,
            best_counters: PerfCounters::default(),
            median_counters: PerfCounters::default(),
        }
    }

    /// Cycles per operation (target: ≤8 for hot path)
    pub fn cycles_per_op(&self) -> f64 {
        if self.iterations == 0 {
            0.0
        } else {
            self.best_counters.cycles as f64 / self.iterations as f64
        }
    }

    /// Instructions per operation
    pub fn instructions_per_op(&self) -> f64 {
        if self.iterations == 0 {
            0.0
        } else {
            self.best_counters.instructions as f64 / self.iterations as f64
        }
    }

    /// Operations per second
    pub fn ops_per_sec(&self) -> f64 {
        if self.best_time.as_secs_f64() == 0.0 {
            0.0
        } else {
            self.iterations as f64 / self.best_time.as_secs_f64()
        }
    }

    /// Nanoseconds per operation
    pub fn ns_per_op(&self) -> f64 {
        if self.iterations == 0 {
            0.0
        } else {
            self.best_time.as_nanos() as f64 / self.iterations as f64
        }
    }

    /// Print results in simdjson-style format
    pub fn print_report(&self) {
        println!("\n{}", "=".repeat(80));
        println!("Benchmark: {}", self.name);
        println!("{}", "=".repeat(80));
        println!(
            "{:<40} : {:>12.2} cycles/op",
            "Best",
            self.cycles_per_op()
        );
        println!(
            "{:<40} : {:>12.2} instrs/op",
            "",
            self.instructions_per_op()
        );
        println!(
            "{:<40} : {:>12.2} IPC",
            "",
            self.best_counters.ipc()
        );
        println!(
            "{:<40} : {:>12.2} ns/op",
            "",
            self.ns_per_op()
        );
        println!(
            "{:<40} : {:>12.0} ops/sec",
            "",
            self.ops_per_sec()
        );
        println!(
            "{:<40} : {:>12.2}%",
            "Cache miss rate",
            self.best_counters.cache_miss_rate() * 100.0
        );
        println!(
            "{:<40} : {:>12.2}%",
            "Branch miss rate",
            self.best_counters.branch_miss_rate() * 100.0
        );
        println!(
            "{:<40} : {:>12} iterations",
            "Sample size",
            self.iterations
        );

        // KNHK-specific validation
        let ticks = self.cycles_per_op();
        if ticks <= 8.0 {
            println!("\n✅ HOT PATH COMPLIANT: {:.2} ticks ≤ 8 ticks", ticks);
        } else {
            println!("\n❌ EXCEEDS HOT PATH BUDGET: {:.2} ticks > 8 ticks", ticks);
        }
        println!("{}", "=".repeat(80));
    }
}

/// Benchmark harness inspired by simdjson's BEST_TIME macro
pub struct BenchmarkHarness {
    warmup_iterations: usize,
    measure_iterations: usize,
}

impl Default for BenchmarkHarness {
    fn default() -> Self {
        Self {
            warmup_iterations: 1000,
            measure_iterations: 10000,
        }
    }
}

impl BenchmarkHarness {
    pub fn new(warmup: usize, measure: usize) -> Self {
        Self {
            warmup_iterations: warmup,
            measure_iterations: measure,
        }
    }

    /// Benchmark a function with hardware performance counters (Linux only)
    #[cfg(target_os = "linux")]
    pub fn measure<F, R>(&self, name: &str, mut f: F) -> BenchmarkResult
    where
        F: FnMut() -> R,
    {
        use perf_event::events::Hardware;

        // Warmup phase
        for _ in 0..self.warmup_iterations {
            black_box(f());
        }

        // Setup performance counters
        let mut group = Group::new().expect("Failed to create perf counter group");
        let cycles_counter = Builder::new()
            .group(&mut group)
            .kind(Hardware::CPU_CYCLES)
            .build()
            .expect("Failed to create cycles counter");

        let instrs_counter = Builder::new()
            .group(&mut group)
            .kind(Hardware::INSTRUCTIONS)
            .build()
            .expect("Failed to create instructions counter");

        let cache_refs_counter = Builder::new()
            .group(&mut group)
            .kind(Hardware::CACHE_REFERENCES)
            .build()
            .expect("Failed to create cache refs counter");

        let cache_misses_counter = Builder::new()
            .group(&mut group)
            .kind(Hardware::CACHE_MISSES)
            .build()
            .expect("Failed to create cache misses counter");

        let branches_counter = Builder::new()
            .group(&mut group)
            .kind(Hardware::BRANCH_INSTRUCTIONS)
            .build()
            .expect("Failed to create branches counter");

        let branch_misses_counter = Builder::new()
            .group(&mut group)
            .kind(Hardware::BRANCH_MISSES)
            .build()
            .expect("Failed to create branch misses counter");

        // Measurement phase
        group.enable().expect("Failed to enable counters");
        let start = Instant::now();

        for _ in 0..self.measure_iterations {
            std::sync::atomic::fence(std::sync::atomic::Ordering::Acquire);
            black_box(f());
            std::sync::atomic::fence(std::sync::atomic::Ordering::Release);
        }

        let elapsed = start.elapsed();
        group.disable().expect("Failed to disable counters");

        // Read counters
        let counts = group.read().expect("Failed to read counters");

        let counters = PerfCounters {
            cycles: counts[&cycles_counter],
            instructions: counts[&instrs_counter],
            cache_refs: counts[&cache_refs_counter],
            cache_misses: counts[&cache_misses_counter],
            branches: counts[&branches_counter],
            branch_misses: counts[&branch_misses_counter],
        };

        let mut result = BenchmarkResult::new(name.to_string());
        result.iterations = self.measure_iterations;
        result.total_time = elapsed;
        result.best_time = elapsed;
        result.worst_time = elapsed;
        result.median_time = elapsed;
        result.best_counters = counters.clone();
        result.median_counters = counters;

        result
    }

    /// Fallback benchmark without hardware counters (macOS, other platforms)
    #[cfg(not(target_os = "linux"))]
    pub fn measure<F, R>(&self, name: &str, mut f: F) -> BenchmarkResult
    where
        F: FnMut() -> R,
    {
        // Warmup phase
        for _ in 0..self.warmup_iterations {
            black_box(f());
        }

        // Measurement phase
        let start = Instant::now();

        for _ in 0..self.measure_iterations {
            std::sync::atomic::fence(std::sync::atomic::Ordering::Acquire);
            black_box(f());
            std::sync::atomic::fence(std::sync::atomic::Ordering::Release);
        }

        let elapsed = start.elapsed();

        let mut result = BenchmarkResult::new(name.to_string());
        result.iterations = self.measure_iterations;
        result.total_time = elapsed;
        result.best_time = elapsed;
        result.worst_time = elapsed;
        result.median_time = elapsed;

        println!("\n⚠️  Running without hardware performance counters (not Linux)");
        println!("   Only timing measurements available.");

        result
    }

    /// Run multiple trials and keep best result
    pub fn measure_multi<F, R>(&self, name: &str, mut f: F, trials: usize) -> BenchmarkResult
    where
        F: FnMut() -> R,
    {
        let mut best = self.measure(name, &mut f);

        for _ in 1..trials {
            let current = self.measure(name, &mut f);
            if current.best_time < best.best_time {
                best = current;
            }
        }

        best
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_harness() {
        let harness = BenchmarkHarness::new(100, 1000);

        let result = harness.measure("simple_addition", || {
            let a = 42u64;
            let b = 17u64;
            a.wrapping_add(b)
        });

        assert_eq!(result.iterations, 1000);
        assert!(result.best_time > Duration::ZERO);

        #[cfg(target_os = "linux")]
        {
            assert!(result.best_counters.cycles > 0);
            assert!(result.best_counters.instructions > 0);
        }
    }

    #[test]
    fn test_perf_counters_calculations() {
        let counters = PerfCounters {
            cycles: 1000,
            instructions: 1500,
            cache_refs: 100,
            cache_misses: 10,
            branches: 200,
            branch_misses: 5,
        };

        assert_eq!(counters.ipc(), 1.5);
        assert_eq!(counters.cache_miss_rate(), 0.1);
        assert_eq!(counters.branch_miss_rate(), 0.025);
    }
}
