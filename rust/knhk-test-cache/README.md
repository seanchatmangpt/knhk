# knhk-test-cache: Pure Rust Autonomic Test Cache Daemon

**DFLSS-Aligned Implementation**

## Overview

Pure Rust autonomic test cache daemon that monitors code changes and keeps test binaries pre-compiled, achieving ≤5 second test execution SLA.

## DFLSS Alignment

### Architecture Principles
- **80/20 Focus**: Critical path optimization (test binary pre-compilation)
- **Production-Ready**: No placeholders, real implementations, proper error handling
- **Autonomic**: Self-governing system that maintains invariants continuously
- **Performance**: ≤5 second SLA for test execution

### Mathematical Foundations
- **μ (Measurement)**: Code hashing (SHA-256) measures code state
- **O (Observations)**: File system events (file changes)
- **A (Actions)**: Test binary pre-compilation
- **Idempotence**: μ∘μ = μ (hashing is idempotent)
- **Determinism**: Same code → same hash → same cache result

### ACHI Identity Principles
- **Source**: O only (file system events, not cached state)
- **Determinism**: Same code → same hash → deterministic cache lookup
- **No partials**: All-or-none cache operations
- **Guards**: Cache expiration (1 hour TTL) enforces freshness

## Package Structure

```
knhk-test-cache/
├── Cargo.toml          # Package manifest
├── README.md           # This file
└── src/
    ├── lib.rs          # Library entry point
    ├── bin/
    │   └── main.rs     # CLI binary
    ├── cache.rs        # Test result caching
    ├── compiler.rs     # Test binary pre-compilation
    ├── daemon.rs       # Daemon lifecycle management
    ├── hasher.rs       # Code hashing for cache invalidation
    └── watcher.rs      # File system watching
```

## Components

### 1. CodeHasher (`hasher.rs`)
- Generates SHA-256 hash of all Rust source files
- Deterministic: same code → same hash
- Excludes `target/`, `.git/`, `.test-cache/`
- Used for cache invalidation

### 2. Cache (`cache.rs`)
- Stores test results by code hash
- Automatic expiration (1 hour TTL)
- Keeps last 10 cached results
- JSON serialization for persistence

### 3. FileWatcher (`watcher.rs`)
- Monitors `.rs` files for changes using `notify` crate
- Debouncing (1 second) to batch rapid changes
- Excludes build artifacts and cache directories
- Sends change events to async channel

### 4. TestCompiler (`compiler.rs`)
- Pre-compiles test binaries using `cargo build --tests --workspace`
- Incremental compilation enabled
- Error handling with detailed messages
- Checks cargo availability

### 5. Daemon (`daemon.rs`)
- Manages daemon lifecycle (start/stop/status/rebuild)
- PID file management
- Coordinates file watcher and compiler
- Graceful shutdown on SIGINT

## Usage

### CLI Commands

```bash
# Start daemon
knhk-test-cache start

# Stop daemon
knhk-test-cache stop

# Check status
knhk-test-cache status

# Force rebuild
knhk-test-cache rebuild
```

### Library Usage

```rust
use knhk_test_cache::{Daemon, CodeHasher, Cache, TestCompiler};

// Create daemon
let daemon = Daemon::new(project_root);

// Start daemon
daemon.start().await?;

// Check status
let status = daemon.status()?;
println!("Running: {}", status.running);
```

## Performance

| Scenario | Time | Notes |
|----------|------|-------|
| Code unchanged (cached) | 0.1-0.5s | Instant cache lookup |
| Code changed, binaries ready | 2-5s | Parallel test execution |
| Code changed, no binaries | 7-15s | Incremental compilation |

## Dependencies

- `notify` - File system watching
- `tokio` - Async runtime
- `sha2` - Code hashing
- `serde` - Cache serialization
- `clap` - CLI interface
- `tracing` - Structured logging

## Testing

```bash
# Run tests
cargo test --package knhk-test-cache

# Run with features
cargo test --package knhk-test-cache --features std
```

## Integration

### With Makefile

```makefile
test-cache-start:
	@knhk-test-cache start

test-cache-stop:
	@knhk-test-cache stop

test-cache-status:
	@knhk-test-cache status
```

### With CI/CD

```yaml
- name: Start test cache daemon
  run: knhk-test-cache start

- name: Run tests
  run: cargo test --workspace
```

## Future Enhancements

1. **Distributed caching** - Share cache across team
2. **Predictive compilation** - Pre-compile likely-to-change files
3. **Smart test selection** - Only run tests affected by changes
4. **Cloud cache** - Remote cache for CI/CD
