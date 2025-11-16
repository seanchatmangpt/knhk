//! Throughput Benchmarks
//!
//! Measures sustained throughput, burst capacity, and resource efficiency
//! for production workloads.

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use parking_lot::{RwLock, Mutex};
use crossbeam::channel::{bounded, Sender, Receiver};

/// Throughput test workload
pub struct WorkloadGenerator {
    counter: Arc<AtomicU64>,
    stop_flag: Arc<AtomicBool>,
    workers: Vec<thread::JoinHandle<WorkerStats>>,
}

#[derive(Debug, Default)]
pub struct WorkerStats {
    pub operations: u64,
    pub duration: Duration,
    pub errors: u64,
    pub max_latency: Duration,
    pub total_latency: Duration,
}

impl WorkloadGenerator {
    pub fn new(num_workers: usize) -> Self {
        Self {
            counter: Arc::new(AtomicU64::new(0)),
            stop_flag: Arc::new(AtomicBool::new(false)),
            workers: Vec::with_capacity(num_workers),
        }
    }

    pub fn start_sustained_load(&mut self, ops_per_sec: u64) -> Arc<AtomicU64> {
        let counter = self.counter.clone();
        let stop_flag = self.stop_flag.clone();
        let ops_per_worker = ops_per_sec / self.workers.capacity().max(1) as u64;

        for worker_id in 0..self.workers.capacity() {
            let counter_clone = counter.clone();
            let stop_clone = stop_flag.clone();

            let handle = thread::spawn(move || {
                let mut stats = WorkerStats::default();
                let interval = Duration::from_nanos(1_000_000_000 / ops_per_worker.max(1));
                let start = Instant::now();

                while !stop_clone.load(Ordering::Acquire) {
                    let op_start = Instant::now();

                    // Simulate work
                    counter_clone.fetch_add(1, Ordering::Relaxed);
                    stats.operations += 1;

                    let op_latency = op_start.elapsed();
                    stats.total_latency += op_latency;
                    if op_latency > stats.max_latency {
                        stats.max_latency = op_latency;
                    }

                    // Rate limiting
                    if let Some(sleep_time) = interval.checked_sub(op_latency) {
                        thread::sleep(sleep_time);
                    }
                }

                stats.duration = start.elapsed();
                stats
            });

            self.workers.push(handle);
        }

        counter.clone()
    }

    pub fn stop(&mut self) -> ThroughputResult {
        self.stop_flag.store(true, Ordering::Release);

        let mut total_ops = 0u64;
        let mut total_errors = 0u64;
        let mut max_latency = Duration::from_secs(0);
        let mut total_latency = Duration::from_secs(0);
        let mut total_duration = Duration::from_secs(0);

        for handle in self.workers.drain(..) {
            if let Ok(stats) = handle.join() {
                total_ops += stats.operations;
                total_errors += stats.errors;
                if stats.max_latency > max_latency {
                    max_latency = stats.max_latency;
                }
                total_latency += stats.total_latency;
                if stats.duration > total_duration {
                    total_duration = stats.duration;
                }
            }
        }

        ThroughputResult {
            total_operations: total_ops,
            duration: total_duration,
            throughput: total_ops as f64 / total_duration.as_secs_f64(),
            errors: total_errors,
            max_latency,
            avg_latency: if total_ops > 0 {
                total_latency / total_ops as u32
            } else {
                Duration::from_secs(0)
            },
        }
    }
}

#[derive(Debug)]
pub struct ThroughputResult {
    pub total_operations: u64,
    pub duration: Duration,
    pub throughput: f64,
    pub errors: u64,
    pub max_latency: Duration,
    pub avg_latency: Duration,
}

/// Pipeline throughput test
pub struct PipelineProcessor {
    stages: Vec<Arc<dyn Fn(u64) -> u64 + Send + Sync>>,
    channels: Vec<(Sender<u64>, Receiver<u64>)>,
    workers: Vec<Option<thread::JoinHandle<()>>>,
}

impl PipelineProcessor {
    pub fn new() -> Self {
        let stages: Vec<Arc<dyn Fn(u64) -> u64 + Send + Sync>> = vec![
            Arc::new(|x| x * 2),          // Stage 1: Multiply
            Arc::new(|x| x + 100),         // Stage 2: Add
            Arc::new(|x| x % 1000),        // Stage 3: Modulo
            Arc::new(|x| x.wrapping_mul(31)), // Stage 4: Hash
        ];

        let channels = (0..stages.len() + 1)
            .map(|_| bounded::<u64>(1000))
            .collect();

        Self {
            stages,
            channels,
            workers: Vec::new(),
        }
    }

    pub fn start(&mut self) {
        for i in 0..self.stages.len() {
            let stage = self.stages[i].clone();
            let receiver = self.channels[i].1.clone();
            let sender = self.channels[i + 1].0.clone();

            let handle = thread::spawn(move || {
                while let Ok(value) = receiver.recv() {
                    let processed = stage(value);
                    if sender.send(processed).is_err() {
                        break;
                    }
                }
            });

            self.workers.push(Some(handle));
        }
    }

    pub fn process(&self, value: u64) -> Result<(), &'static str> {
        self.channels[0].0.send(value)
            .map_err(|_| "Pipeline input failed")
    }

    pub fn collect(&self) -> Result<u64, &'static str> {
        self.channels.last()
            .ok_or("No output channel")?
            .1.recv()
            .map_err(|_| "Pipeline output failed")
    }
}

/// Burst capacity test
pub struct BurstTester {
    max_burst: AtomicU64,
    current_burst: AtomicU64,
    burst_history: Arc<RwLock<Vec<BurstEvent>>>,
}

#[derive(Debug, Clone)]
pub struct BurstEvent {
    pub timestamp: Instant,
    pub size: u64,
    pub duration: Duration,
    pub success: bool,
}

impl BurstTester {
    pub fn new() -> Self {
        Self {
            max_burst: AtomicU64::new(0),
            current_burst: AtomicU64::new(0),
            burst_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn test_burst(&self, size: u64) -> BurstResult {
        let start = Instant::now();
        self.current_burst.store(size, Ordering::Release);

        let mut successful = 0u64;
        let mut failed = 0u64;

        // Simulate burst processing
        for _ in 0..size {
            if thread_rng().gen_bool(0.95) {
                successful += 1;
            } else {
                failed += 1;
            }
        }

        let duration = start.elapsed();

        // Update max burst if this is larger
        self.max_burst.fetch_max(successful, Ordering::AcqRel);

        // Record event
        self.burst_history.write().push(BurstEvent {
            timestamp: start,
            size,
            duration,
            success: failed == 0,
        });

        self.current_burst.store(0, Ordering::Release);

        BurstResult {
            requested: size,
            processed: successful,
            failed,
            duration,
            throughput: successful as f64 / duration.as_secs_f64(),
        }
    }

    pub fn find_max_sustainable_burst(&self) -> u64 {
        let mut low = 100u64;
        let mut high = 100000u64;
        let mut best = 0u64;

        while low <= high {
            let mid = (low + high) / 2;
            let result = self.test_burst(mid);

            if result.failed == 0 && result.throughput > 1000.0 {
                best = mid;
                low = mid + 1;
            } else {
                high = mid - 1;
            }
        }

        best
    }
}

#[derive(Debug)]
pub struct BurstResult {
    pub requested: u64,
    pub processed: u64,
    pub failed: u64,
    pub duration: Duration,
    pub throughput: f64,
}

/// Memory efficiency test
pub struct MemoryEfficiencyTester {
    allocations: Arc<AtomicU64>,
    deallocations: Arc<AtomicU64>,
    peak_memory: Arc<AtomicU64>,
    current_memory: Arc<AtomicU64>,
}

impl MemoryEfficiencyTester {
    pub fn new() -> Self {
        Self {
            allocations: Arc::new(AtomicU64::new(0)),
            deallocations: Arc::new(AtomicU64::new(0)),
            peak_memory: Arc::new(AtomicU64::new(0)),
            current_memory: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn allocate(&self, size: u64) {
        self.allocations.fetch_add(1, Ordering::Relaxed);
        let new_size = self.current_memory.fetch_add(size, Ordering::AcqRel) + size;
        self.peak_memory.fetch_max(new_size, Ordering::AcqRel);
    }

    pub fn deallocate(&self, size: u64) {
        self.deallocations.fetch_add(1, Ordering::Relaxed);
        self.current_memory.fetch_sub(size, Ordering::AcqRel);
    }

    pub fn get_efficiency_metrics(&self) -> MemoryEfficiencyMetrics {
        let allocs = self.allocations.load(Ordering::Acquire);
        let deallocs = self.deallocations.load(Ordering::Acquire);
        let current = self.current_memory.load(Ordering::Acquire);
        let peak = self.peak_memory.load(Ordering::Acquire);

        MemoryEfficiencyMetrics {
            total_allocations: allocs,
            total_deallocations: deallocs,
            allocation_rate: allocs as f64 / deallocs.max(1) as f64,
            current_usage: current,
            peak_usage: peak,
            efficiency: if peak > 0 {
                current as f64 / peak as f64
            } else {
                1.0
            },
        }
    }
}

#[derive(Debug)]
pub struct MemoryEfficiencyMetrics {
    pub total_allocations: u64,
    pub total_deallocations: u64,
    pub allocation_rate: f64,
    pub current_usage: u64,
    pub peak_usage: u64,
    pub efficiency: f64,
}

use rand::{thread_rng, Rng};

/// Benchmark sustained throughput
fn bench_sustained_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("sustained_throughput");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10);

    for &target_ops in &[1000u64, 5000, 10000] {
        group.throughput(Throughput::Elements(target_ops));
        group.bench_function(format!("{}_ops_per_sec", target_ops), |b| {
            b.iter_custom(|_iters| {
                let mut generator = WorkloadGenerator::new(4);
                generator.start_sustained_load(target_ops);
                thread::sleep(Duration::from_secs(5));
                let result = generator.stop();
                Duration::from_secs_f64(result.total_operations as f64)
            });
        });
    }

    group.finish();
}

/// Benchmark burst throughput
fn bench_burst_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("burst_throughput");

    let tester = BurstTester::new();

    for &burst_size in &[100u64, 1000, 10000] {
        group.throughput(Throughput::Elements(burst_size));
        group.bench_function(format!("burst_{}", burst_size), |b| {
            b.iter(|| {
                let result = tester.test_burst(burst_size);
                black_box(result.processed)
            });
        });
    }

    // Find maximum sustainable burst
    let max_burst = tester.find_max_sustainable_burst();
    println!("Maximum sustainable burst: {} operations", max_burst);

    group.finish();
}

/// Benchmark pipeline throughput
fn bench_pipeline_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("pipeline_throughput");

    let mut pipeline = PipelineProcessor::new();
    pipeline.start();

    group.bench_function("4_stage_pipeline", |b| {
        let mut counter = 0u64;
        b.iter(|| {
            pipeline.process(counter).unwrap();
            counter += 1;
            if counter % 100 == 0 {
                // Collect some results to prevent backpressure
                for _ in 0..50 {
                    let _ = pipeline.collect();
                }
            }
            black_box(counter)
        });
    });

    group.finish();
}

/// Benchmark memory efficiency
fn bench_memory_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_efficiency");

    let tester = MemoryEfficiencyTester::new();

    group.bench_function("allocation_pattern", |b| {
        b.iter(|| {
            // Simulate typical allocation pattern
            for i in 0..100 {
                let size = (i % 10 + 1) * 1024; // 1KB to 10KB
                tester.allocate(size);
                if i % 3 == 0 && i > 0 {
                    tester.deallocate(size / 2);
                }
            }
        });
    });

    let metrics = tester.get_efficiency_metrics();
    println!("Memory Efficiency Metrics:");
    println!("  Total allocations: {}", metrics.total_allocations);
    println!("  Peak usage: {} bytes", metrics.peak_usage);
    println!("  Efficiency: {:.2}%", metrics.efficiency * 100.0);

    group.finish();
}

/// Benchmark concurrent throughput
fn bench_concurrent_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_throughput");

    for &num_threads in &[1, 2, 4, 8, 16] {
        group.bench_function(format!("{}_threads", num_threads), |b| {
            b.iter_custom(|iters| {
                let counter = Arc::new(AtomicU64::new(0));
                let start = Instant::now();

                let handles: Vec<_> = (0..num_threads)
                    .map(|_| {
                        let counter_clone = counter.clone();
                        thread::spawn(move || {
                            for _ in 0..iters / num_threads as u64 {
                                counter_clone.fetch_add(1, Ordering::Relaxed);
                            }
                        })
                    })
                    .collect();

                for handle in handles {
                    handle.join().unwrap();
                }

                start.elapsed()
            });
        });
    }

    group.finish();
}

/// Benchmark I/O throughput
fn bench_io_throughput(c: &mut Criterion) {
    use std::io::{Write, BufWriter};
    use tempfile::tempfile;

    let mut group = c.benchmark_group("io_throughput");

    group.bench_function("buffered_write", |b| {
        b.iter(|| {
            let file = tempfile().unwrap();
            let mut writer = BufWriter::new(file);
            let data = vec![0u8; 4096];

            for _ in 0..100 {
                writer.write_all(&data).unwrap();
            }
            writer.flush().unwrap();
        });
    });

    group.finish();
}

/// Network simulation throughput
fn bench_network_throughput(c: &mut Criterion) {
    use crossbeam::channel::bounded;

    let mut group = c.benchmark_group("network_throughput");

    let (tx, rx) = bounded(1000);

    group.bench_function("channel_throughput", |b| {
        b.iter(|| {
            // Simulate network packet processing
            for i in 0..100 {
                tx.send(i).unwrap();
            }
            for _ in 0..100 {
                let _ = rx.recv().unwrap();
            }
        });
    });

    group.finish();
}

criterion_group!(
    throughput_benches,
    bench_sustained_throughput,
    bench_burst_throughput,
    bench_pipeline_throughput,
    bench_memory_efficiency,
    bench_concurrent_throughput,
    bench_io_throughput,
    bench_network_throughput
);

criterion_main!(throughput_benches);