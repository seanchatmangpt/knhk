# KNHK Autonomous Evolution Loop

The continuous evolution engine that autonomously evolves KNHK's ontology without human intervention.

## Overview

The Autonomous Evolution Loop implements a six-step cycle that runs continuously in the background:

```
observe (O, A)
  → detect patterns
  → propose ΔΣ
  → validate against Q
  → compile Π
  → promote Σ*
  → (loop)
```

Every cycle produces a new ontology snapshot when patterns are detected and validated.

## Features

- **Autonomous Operation**: Runs continuously without human intervention
- **Safety Mechanisms**: Multiple limits prevent runaway evolution
- **Health Monitoring**: Comprehensive health status and statistics
- **Graceful Degradation**: Continues on partial failures
- **Observable**: Full OpenTelemetry integration
- **Production-Ready**: No unsafe code, comprehensive error handling
- **Testable**: Dependency injection enables thorough testing

## Quick Start

```rust
use knhk_autonomous_loop::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // Configure the loop
    let config = AutonomousLoopConfig::new()
        .with_cycle_interval(Duration::from_secs(60))
        .with_min_patterns(10)
        .with_auto_promote(true);

    // Create dependencies (from KNHK components)
    let dependencies = create_knhk_dependencies().await?;

    // Start the loop
    let handle = start_autonomous_loop(config, dependencies)?;

    // Monitor health
    let health = handle.engine().get_health().await;
    let stats = handle.engine().get_stats().await;

    println!("Status: {}", health.status());
    println!("Success rate: {:.1}%", stats.success_rate());

    // Stop gracefully
    handle.stop().await?;

    Ok(())
}
```

## Architecture

### Core Components

- **LoopEngine**: Main orchestrator that runs continuous cycles
- **EvolutionCycle**: Single cycle implementation
- **LoopDependencies**: Dependency injection container
- **LoopHealth**: Health monitoring and circuit breaker
- **LoopTelemetry**: OpenTelemetry integration

### Six-Step Cycle

1. **Observe**: Fetch recent telemetry receipts from ReceiptLog
2. **Detect**: Mine patterns using PatternMiner
3. **Propose**: Generate change proposals from patterns
4. **Validate**: Check invariants Q preservation
5. **Compile**: Create new snapshot overlay and commit
6. **Promote**: Deploy to production if ready

### Safety Mechanisms

```rust
pub struct AutonomousLoopConfig {
    /// Prevent premature evolution
    pub min_patterns_for_proposal: usize,

    /// Prevent runaway changes
    pub max_changes_per_cycle: usize,

    /// Circuit breaker threshold
    pub pause_on_error_rate: Option<f64>,

    /// Automatic rollback
    pub auto_rollback_on_slo_violation: bool,
}
```

## Configuration

```rust
let config = AutonomousLoopConfig::new()
    // Run every 60 seconds
    .with_cycle_interval(Duration::from_secs(60))

    // Require 10+ patterns before proposing
    .with_min_patterns(10)

    // Auto-promote production-ready snapshots
    .with_auto_promote(true)

    // Pause if error rate > 5%
    .with_error_threshold(5.0)

    // Alert on major events
    .with_alert_email("team@example.com".to_string());
```

## Monitoring

### Health Status

```rust
let health = handle.engine().get_health().await;

match health {
    LoopHealth::Running => println!("✅ Healthy"),
    LoopHealth::Paused { reason } => println!("⏸️  Paused: {}", reason),
    LoopHealth::Error { error, retry_count, .. } => {
        println!("❌ Error (retry {}): {}", retry_count, error)
    }
    LoopHealth::Stopped => println!("🛑 Stopped"),
}
```

### Statistics

```rust
let stats = handle.engine().get_stats().await;

println!("Total cycles: {}", stats.total_cycles);
println!("Success rate: {:.1}%", stats.success_rate());
println!("Error rate: {:.1}%", stats.error_rate);
println!("Avg duration: {}ms", stats.avg_cycle_duration_ms);
```

### Cycle History

```rust
let history = handle.engine().get_history().await;

for cycle in history {
    match cycle.result {
        CycleResult::Success { new_snapshot_id, duration_ms } => {
            println!("✅ Cycle {}: {} ({}ms)",
                cycle.cycle_id,
                hex::encode(&new_snapshot_id),
                duration_ms
            );
        }
        _ => { /* ... */ }
    }
}
```

## Testing

```bash
# Run all tests
cargo test --workspace

# Run specific test suite
cargo test --test cycle_tests
cargo test --test loop_tests
cargo test --test health_tests

# Run with logging
RUST_LOG=knhk_autonomous_loop=debug cargo test
```

## Examples

```bash
# Basic usage example
cargo run --example basic_usage

# KNHK integration example
cargo run --example knhk_integration
```

## Integration with KNHK

Add to your KNHK `main.rs`:

```rust
use knhk_autonomous_loop::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize KNHK components
    let snapshot_store = initialize_snapshot_store().await?;
    let pattern_miner = initialize_pattern_miner()?;
    // ... other components

    // Create dependencies
    let deps = LoopDependencies::new(
        Arc::new(snapshot_store),
        Arc::new(pattern_miner),
        // ... other dependencies
    );

    // Start autonomous evolution
    let _evolution_handle = start_autonomous_loop(
        AutonomousLoopConfig::default(),
        deps
    )?;

    // Run KNHK normally
    knhk_main_loop().await
}
```

## Performance

- **Cycle Duration**: Typically 100-1000ms depending on pattern count
- **Memory Usage**: Bounded (last 100 cycles retained)
- **Throughput**: Configurable (default: 60s interval)
- **Latency**: Non-blocking async operations

## Safety

- **No `unsafe` code**: 100% safe Rust
- **Comprehensive error handling**: All errors are `Result<T, EvolutionError>`
- **Graceful shutdown**: Cooperative cancellation
- **Weaver validation**: Source of truth for correctness

## Documentation

- [Architecture](docs/ARCHITECTURE.md) - Detailed system design
- [ADR-001](docs/ADR-001-AUTONOMOUS-LOOP-ARCHITECTURE.md) - Architecture decision record
- [API Documentation](https://docs.rs/knhk-autonomous-loop) - API reference

## License

Same as KNHK project.

## Contributing

See KNHK project contributing guidelines.
