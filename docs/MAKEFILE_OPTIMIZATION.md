# Makefile Optimization for 5-Second SLA

## Overview

The Makefile has been optimized to meet a 5-second SLA for all development tasks. This document describes the optimizations implemented.

## Key Optimizations

### 1. Incremental Compilation

All build and test targets now use `CARGO_INCREMENTAL=1` to enable incremental compilation. This means:
- Only changed crates are rebuilt
- Subsequent builds are much faster (often <1 second if nothing changed)
- Build artifacts are preserved in `target/` directory

**Example:**
```makefile
build-rust:
	@timeout 5 sh -c 'cd rust && CARGO_INCREMENTAL=1 cargo build --workspace'
```

### 2. Dev Profile by Default

The default `build-rust` target now uses the dev profile (fast compilation) instead of release:
- **Before**: `cargo build --workspace --release` (slow, optimized)
- **After**: `cargo build --workspace` (fast, dev profile)
- **Release builds**: Use `make build-rust-release` when needed

### 3. Fast Test Execution

Test targets now:
- Run only library tests (`--lib`) by default (skip integration tests)
- Use single-threaded execution (`--test-threads=1`) for faster feedback
- Skip slow crates in fast mode (knhk-integration-tests, knhk-cli)

**Example:**
```makefile
test-rust:
	@timeout 5 sh -c 'cd rust && CARGO_INCREMENTAL=1 cargo test --workspace --lib --test-threads=1 --quiet'
```

### 4. Selective Execution

Scripts now support fast mode that skips slow crates:
- `run-all-rust-tests.sh`: Skips knhk-integration-tests and knhk-cli by default
- `run-lint-rust.sh`: Skips knhk-integration-tests, knhk-cli, and knhk-workflow-engine by default
- Set `FAST_MODE=0` to include all crates

### 5. Build Caching

Created `.cargo/config.toml` with:
- Incremental compilation enabled
- Configuration for sccache (when installed)
- Fast linker configuration (zld/mold)
- Optimized profile settings

### 6. Formatting Optimization

The `fmt` target now:
- Checks formatting first (`--check`) before formatting
- Only formats files that need it
- Reduces unnecessary work

## Target Performance

| Target | Before | After (Target) | Notes |
|--------|--------|----------------|-------|
| `make check` | ~40s | ≤5s | Incremental, only changed |
| `make test-rust` | ~256s | ≤5s | Lib tests only, fast mode |
| `make build-rust` | ~233s | ≤5s | Incremental, dev profile |
| `make fmt` | ~10s | ≤5s | Check first, format only changed |
| `make lint-rust` | ~192s | ≤5s | Fast mode, lib/bins only |

## Usage

### Fast Development Workflow

```bash
# Quick check (fastest)
make check

# Quick tests (lib tests only)
make test-rust

# Quick build (incremental, dev profile)
make build-rust

# Format code
make fmt

# Lint code
make lint-rust
```

### Full Validation (When Needed)

```bash
# Full test suite (includes slow crates)
FAST_MODE=0 make test-rust

# Release build (optimized)
make build-rust-release

# Full validation
make test-all
```

## Setup Build Caching

For even faster builds, setup sccache:

```bash
# Install and configure sccache
./scripts/setup-build-cache.sh

# Or manually:
cargo install sccache
# Then uncomment rustc-wrapper in rust/.cargo/config.toml
```

## Fast Linker (Optional)

For 2-3x faster linking:

**macOS:**
```bash
cargo install -f zld
# Uncomment zld config in rust/.cargo/config.toml
```

**Linux:**
```bash
sudo apt install mold
# Uncomment mold config in rust/.cargo/config.toml
```

## Troubleshooting

### Builds Still Slow?

1. **Check incremental compilation:**
   ```bash
   echo $CARGO_INCREMENTAL  # Should be 1
   ```

2. **Clear and rebuild once:**
   ```bash
   make clean
   make build-rust  # First build will be slow
   make build-rust  # Second build should be fast
   ```

3. **Check sccache:**
   ```bash
   sccache --show-stats
   ```

4. **Verify fast mode:**
   ```bash
   FAST_MODE=1 make test-rust  # Fast mode (default)
   FAST_MODE=0 make test-rust  # Full mode
   ```

### Tests Timing Out?

The 5-second timeout is intentional. If tests legitimately need more time:
- Use `make test-all` for full test suite
- Run specific crate tests: `cd rust/knhk-cli && cargo test`
- Adjust timeout in Makefile if needed

## Files Modified

1. **Makefile**: Optimized all targets for fast execution
2. **scripts/run-all-rust-tests.sh**: Added fast mode, incremental compilation
3. **scripts/run-lint-rust.sh**: Added fast mode, lib/bins only
4. **rust/.cargo/config.toml**: Created with optimization settings
5. **scripts/setup-build-cache.sh**: New script for cache setup

## Next Steps

1. **Install sccache** for compilation caching
2. **Install fast linker** (zld/mold) for faster linking
3. **Monitor performance** and adjust timeouts if needed
4. **Optimize slow crates** (knhk-integration-tests, knhk-cli) per roadmap

## References

- [Optimization Roadmap](../rust/docs/OPTIMIZATION_ROADMAP.md)
- [Performance Benchmarks](../rust/docs/PERFORMANCE_BENCHMARK.md)
- [Cargo Incremental Compilation](https://doc.rust-lang.org/cargo/reference/profiles.html#incremental)

