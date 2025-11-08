// perf.rs: Linux perf_event integration for cycle-accurate validation
// Uses perf_event_open syscall to measure cycles/byte, IPC, branch-miss, L1D miss

#[cfg(target_os = "linux")]
use perf_event::{Builder, Counter};

/// Perf measurement result
#[derive(Debug, Clone)]
pub struct PerfMeasurement {
    pub cycles: u64,
    pub instructions: u64,
    pub branch_misses: u64,
    pub l1d_misses: u64,
    pub cycles_per_byte: f64,
    pub ipc: f64, // Instructions per cycle
    pub branch_miss_rate: f64,
    pub l1d_miss_rate: f64,
}

/// Perf event manager for Linux
#[cfg(target_os = "linux")]
pub struct PerfEventManager {
    cycle_counter: Option<Counter>,
    instruction_counter: Option<Counter>,
    branch_miss_counter: Option<Counter>,
    l1d_miss_counter: Option<Counter>,
}

#[cfg(target_os = "linux")]
impl PerfEventManager {
    /// Create a new perf event manager
    pub fn new() -> Result<Self, String> {
        // Create counters for different events
        let cycle_counter = Builder::new()
            .kind(perf_event::events::Hardware::CPU_CYCLES)
            .build()
            .map_err(|e| format!("Failed to create cycle counter: {}", e))?;

        let instruction_counter = Builder::new()
            .kind(perf_event::events::Hardware::INSTRUCTIONS)
            .build()
            .map_err(|e| format!("Failed to create instruction counter: {}", e))?;

        let branch_miss_counter = Builder::new()
            .kind(perf_event::events::Hardware::BRANCH_MISSES)
            .build()
            .map_err(|e| format!("Failed to create branch miss counter: {}", e))?;

        let l1d_miss_counter = Builder::new()
            .kind(perf_event::events::Hardware::CACHE_REFERENCES)
            .build()
            .map_err(|e| format!("Failed to create L1D miss counter: {}", e))?;

        Ok(Self {
            cycle_counter: Some(cycle_counter),
            instruction_counter: Some(instruction_counter),
            branch_miss_counter: Some(branch_miss_counter),
            l1d_miss_counter: Some(l1d_miss_counter),
        })
    }

    /// Start measurement
    pub fn start(&mut self) -> Result<(), String> {
        // Enable counters
        if let Some(ref mut counter) = self.cycle_counter {
            counter
                .enable()
                .map_err(|e| format!("Failed to enable cycle counter: {}", e))?;
        }
        if let Some(ref mut counter) = self.instruction_counter {
            counter
                .enable()
                .map_err(|e| format!("Failed to enable instruction counter: {}", e))?;
        }
        if let Some(ref mut counter) = self.branch_miss_counter {
            counter
                .enable()
                .map_err(|e| format!("Failed to enable branch miss counter: {}", e))?;
        }
        if let Some(ref mut counter) = self.l1d_miss_counter {
            counter
                .enable()
                .map_err(|e| format!("Failed to enable L1D miss counter: {}", e))?;
        }
        Ok(())
    }

    /// Stop measurement and read counters
    pub fn stop(&mut self, bytes_processed: usize) -> Result<PerfMeasurement, String> {
        // Disable counters
        if let Some(ref mut counter) = self.cycle_counter {
            counter
                .disable()
                .map_err(|e| format!("Failed to disable cycle counter: {}", e))?;
        }
        if let Some(ref mut counter) = self.instruction_counter {
            counter
                .disable()
                .map_err(|e| format!("Failed to disable instruction counter: {}", e))?;
        }
        if let Some(ref mut counter) = self.branch_miss_counter {
            counter
                .disable()
                .map_err(|e| format!("Failed to disable branch miss counter: {}", e))?;
        }
        if let Some(ref mut counter) = self.l1d_miss_counter {
            counter
                .disable()
                .map_err(|e| format!("Failed to disable L1D miss counter: {}", e))?;
        }

        // Read counters
        let cycles = self
            .cycle_counter
            .as_ref()
            .and_then(|c| c.read().ok())
            .unwrap_or(0);
        let instructions = self
            .instruction_counter
            .as_ref()
            .and_then(|c| c.read().ok())
            .unwrap_or(0);
        let branch_misses = self
            .branch_miss_counter
            .as_ref()
            .and_then(|c| c.read().ok())
            .unwrap_or(0);
        let l1d_misses = self
            .l1d_miss_counter
            .as_ref()
            .and_then(|c| c.read().ok())
            .unwrap_or(0);

        let cycles_per_byte = if bytes_processed > 0 {
            cycles as f64 / bytes_processed as f64
        } else {
            0.0
        };

        let ipc = if cycles > 0 {
            instructions as f64 / cycles as f64
        } else {
            0.0
        };

        let branch_miss_rate = if instructions > 0 {
            branch_misses as f64 / instructions as f64
        } else {
            0.0
        };

        let l1d_miss_rate = if instructions > 0 {
            l1d_misses as f64 / instructions as f64
        } else {
            0.0
        };

        Ok(PerfMeasurement {
            cycles,
            instructions,
            branch_misses,
            l1d_misses,
            cycles_per_byte,
            ipc,
            branch_miss_rate,
            l1d_miss_rate,
        })
    }
}

/// Benchmark result with both macOS time and Linux perf measurements
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub operation: String,
    pub bytes_processed: usize,
    pub macos_time_ns: u64,
    pub linux_perf: Option<PerfMeasurement>,
    pub cycles_per_byte_macos: f64,
    pub cycles_per_byte_linux: Option<f64>,
}

/// Run benchmark with both macOS time and Linux perf
pub fn benchmark_with_perf<F>(operation: &str, bytes: usize, f: F) -> BenchmarkResult
where
    F: FnOnce(),
{
    let start_time = std::time::Instant::now();
    f();
    let macos_time_ns = start_time.elapsed().as_nanos() as u64;

    // Estimate cycles from time (assuming 4GHz CPU)
    let cycles_per_byte_macos = if bytes > 0 {
        (macos_time_ns as f64 * 4.0) / bytes as f64
    } else {
        0.0
    };

    #[cfg(target_os = "linux")]
    {
        // On Linux, use perf_event_open for accurate measurement
        if let Ok(mut manager) = PerfEventManager::new() {
            if manager.start().is_ok() {
                // Re-run function with perf measurement
                let _ = f();
                if let Ok(perf) = manager.stop(bytes) {
                    return BenchmarkResult {
                        operation: operation.to_string(),
                        bytes_processed: bytes,
                        macos_time_ns,
                        linux_perf: Some(perf.clone()),
                        cycles_per_byte_macos,
                        cycles_per_byte_linux: Some(perf.cycles_per_byte),
                    };
                }
            }
        }
    }

    BenchmarkResult {
        operation: operation.to_string(),
        bytes_processed: bytes,
        macos_time_ns,
        linux_perf: None,
        cycles_per_byte_macos,
        cycles_per_byte_linux: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_with_perf() {
        let result = benchmark_with_perf("test_op", 100, || {
            // Simulate work
            let _ = (0..100).sum::<usize>();
        });

        assert_eq!(result.operation, "test_op");
        assert_eq!(result.bytes_processed, 100);
        assert!(result.macos_time_ns > 0);
    }
}
