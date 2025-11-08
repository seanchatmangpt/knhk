# KNHK Performance Optimization Roadmap

## Executive Summary

**Current State:** 14 crates, 36,954 LOC, ~25 minute full CI pipeline
**Target State:** <15 minute CI pipeline, <2 minute incremental builds
**Estimated Effort:** 2-3 sprints
**Expected ROI:** 47% CI time reduction, 2x developer velocity

---

## Phase 1: Quick Wins (Week 1)

### 1.1 Fix knhk-integration-tests Build Time
**Problem:** 138 LOC taking 87.69s to build (1.57 LOC/s efficiency)
**Root Cause:** Pulls in testcontainers + all workspace dependencies
**Solution:**
```bash
# Split into focused test suites
knhk-integration-tests/
  ├── docker-tests/      # Only docker + testcontainers
  ├── etl-tests/         # Only ETL subsystem
  ├── otel-tests/        # Only OpenTelemetry
  └── full-stack-tests/  # Comprehensive (optional in CI)
```
**Expected Impact:** 87.69s → 15-20s (75% reduction)
**Effort:** 4 hours

### 1.2 Optimize knhk-config Test Suite
**Problem:** 544 LOC with 101.61s test time
**Root Cause:** Heavy I/O operations, environment setup
**Solution:**
```rust
// Before: Real filesystem I/O
#[test]
fn test_config_load() {
    let config = Config::load("test.toml").unwrap();
    assert_eq!(config.key, "value");
}

// After: In-memory mocking
#[test]
fn test_config_load() {
    let mock_data = r#"key = "value""#;
    let config = Config::from_str(mock_data).unwrap();
    assert_eq!(config.key, "value");
}
```
**Expected Impact:** 101.61s → 20-30s (70% reduction)
**Effort:** 3 hours

### 1.3 Enable sccache for Development
**Problem:** No build caching between runs
**Solution:**
```bash
# Install sccache
cargo install sccache

# Add to ~/.cargo/config.toml
[build]
rustc-wrapper = "sccache"

# Or use mold linker for faster linking
sudo apt install mold  # Linux
cargo install -f --git https://github.com/rui314/mold.git  # macOS
```
**Expected Impact:** 2nd+ builds: 4min → 30-60s (85% reduction)
**Effort:** 30 minutes

**Phase 1 Total Impact:** ~12 hours effort, ~60% CI time reduction

---

## Phase 2: Structural Improvements (Week 2-3)

### 2.1 Refactor knhk-cli
**Problem:** 207.05s release build, 3,496 LOC
**Root Cause:** Large dependency tree (clap, tokio, many workspace crates)
**Solution:**
```
Before:
knhk-cli/
  └── src/
      ├── main.rs (all commands)
      └── commands/*.rs (heavy deps)

After:
knhk-cli-core/     # Core CLI logic (~500 LOC)
knhk-cli-commands/ # Command implementations
knhk-cli/          # Thin binary wrapper (~200 LOC)
```
**Expected Impact:** 207.05s → 80-100s (50% reduction)
**Effort:** 16 hours

### 2.2 Reduce knhk-unrdf Clippy Complexity
**Problem:** 87.76s clippy time (slowest by far)
**Investigation Steps:**
```bash
# 1. Identify expensive lints
cargo clippy -p knhk-unrdf --timings

# 2. Selectively disable expensive lints (if justified)
#![allow(clippy::type_complexity)]  # If RDF structures require it
#![allow(clippy::large_enum_variant)]  # If justified by use case

# 3. Simplify procedural macros
# Review and optimize derive macros if present
```
**Expected Impact:** 87.76s → 30-40s (60% reduction)
**Effort:** 8 hours

### 2.3 Optimize knhk-aot Tests
**Problem:** 47.16s test time for 921 LOC (mostly AOT compilation tests)
**Solution:**
```rust
// Cache compiled artifacts between tests
lazy_static! {
    static ref COMPILED_CACHE: Mutex<HashMap<String, CompiledArtifact>> =
        Mutex::new(HashMap::new());
}

#[test]
fn test_aot_performance() {
    let artifact = get_or_compile("test_pattern", || {
        // Expensive compilation only happens once
        compile_pattern("test_pattern")
    });
    // Fast test using cached artifact
}
```
**Expected Impact:** 47.16s → 15-20s (60% reduction)
**Effort:** 6 hours

**Phase 2 Total Impact:** ~30 hours effort, ~30% additional CI reduction

---

## Phase 3: Infrastructure & Monitoring (Week 4)

### 3.1 CI/CD Build Caching
**Current:** Every CI run does clean build
**Solution:**
```yaml
# GitHub Actions example
- name: Cache cargo registry
  uses: actions/cache@v3
  with:
    path: |
      ~/.cargo/registry
      ~/.cargo/git
      target
    key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

# Use sccache in CI
- name: Setup sccache
  uses: mozilla-actions/sccache-action@v0.0.3

# Incremental compilation for CI
- name: Build
  env:
    CARGO_INCREMENTAL: 1
    SCCACHE_GHA_ENABLED: "true"
  run: cargo build --release
```
**Expected Impact:** CI builds: 13min → 5-7min (50% reduction on cached runs)
**Effort:** 4 hours

### 3.2 Performance Regression Detection
**Solution:**
```bash
# Add to CI pipeline
name: Performance Benchmark
on: [push, pull_request]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - name: Run benchmarks
        run: ./scripts/benchmark_crates.sh

      - name: Compare with baseline
        run: |
          python3 scripts/compare_benchmarks.py \
            --current benchmark_results.json \
            --baseline .github/baseline_benchmark.json \
            --max-regression 10%

      - name: Post results to PR
        uses: actions/github-script@v6
        # Post benchmark comparison as PR comment
```
**Expected Impact:** Prevent performance regressions
**Effort:** 8 hours

### 3.3 Developer Tooling
**Solution:**
```bash
# Add to Makefile
.PHONY: bench
bench:
	@./scripts/benchmark_crates.sh

.PHONY: bench-quick
bench-quick:
	@cargo build --workspace --timings
	@open target/cargo-timings/cargo-timing.html

.PHONY: profile-tests
profile-tests:
	@cargo test --package $(CRATE) -- --nocapture --test-threads=1 \
		| ts '[%Y-%m-%d %H:%M:%.S]'

# Developer dashboard
.PHONY: dev-stats
dev-stats:
	@echo "=== Build Health Dashboard ==="
	@cargo tree --workspace --duplicates
	@cargo udeps --all-targets
	@cargo bloat --release --crates
```
**Expected Impact:** Improved developer experience
**Effort:** 4 hours

**Phase 3 Total Impact:** ~16 hours effort, CI reliability + monitoring

---

## Implementation Timeline

### Week 1: Quick Wins (12 hours)
- [x] Benchmark complete (4h) ✅
- [ ] Split integration tests (4h)
- [ ] Mock knhk-config tests (3h)
- [ ] Setup sccache (0.5h)
- [ ] Verify improvements (0.5h)

### Week 2-3: Structural (30 hours)
- [ ] Refactor knhk-cli (16h)
- [ ] Optimize knhk-unrdf clippy (8h)
- [ ] Cache knhk-aot tests (6h)

### Week 4: Infrastructure (16 hours)
- [ ] CI caching setup (4h)
- [ ] Performance regression tests (8h)
- [ ] Developer tooling (4h)

**Total Effort:** ~58 hours (1.5 sprints)

---

## Success Metrics

### Build Times
| Metric | Current | Target | % Improvement |
|--------|---------|--------|---------------|
| Workspace build (debug) | 233s | 90s | 61% |
| Workspace build (release) | 769s | 350s | 54% |
| Workspace tests | 256s | 100s | 61% |
| Workspace clippy | 192s | 80s | 58% |
| **Total CI pipeline** | **~25 min** | **~13 min** | **48%** |

### Developer Experience
| Metric | Current | Target |
|--------|---------|--------|
| Incremental build | ~4 min | <1 min |
| Test feedback loop | ~4.5 min | <2 min |
| Code-to-test cycle | ~8.5 min | ~3 min |
| Daily build count | ~10-15 | ~30-40 |

### Code Quality
- Maintain 100% clippy compliance
- Maintain current test coverage
- Zero performance regressions
- Documented build practices

---

## Risk Mitigation

### Risk 1: Breaking Changes During Refactor
**Mitigation:**
- Refactor one crate at a time
- Maintain full test coverage during changes
- Use feature flags for incremental rollout

### Risk 2: Cache Invalidation Issues
**Mitigation:**
- Conservative cache keys (hash Cargo.lock)
- Manual cache clear workflow in CI
- Monitor cache hit rates

### Risk 3: Optimization Complexity
**Mitigation:**
- Measure before/after for every change
- Document all optimizations
- Rollback plan for each phase

---

## Maintenance Plan

### Weekly
- Run benchmark suite on main branch
- Review CI build times dashboard
- Check cache hit rates

### Monthly
- Compare benchmarks vs baseline
- Audit new dependencies for build impact
- Review and update optimization docs

### Quarterly
- Full performance audit
- Update benchmark baselines
- Re-evaluate optimization priorities

---

## Appendix: Command Reference

### Benchmark Commands
```bash
# Full benchmark (15-20 minutes)
./scripts/benchmark_crates.sh

# Quick workspace benchmark (5 minutes)
./scripts/benchmark_parallel.sh

# Analyze existing results
python3 scripts/analyze_benchmark.py

# Per-crate timing visualization
cargo build --package knhk-cli --timings
open target/cargo-timings/cargo-timing.html
```

### Dependency Analysis
```bash
# Find duplicate dependencies
cargo tree --workspace --duplicates

# Find unused dependencies
cargo install cargo-udeps
cargo +nightly udeps --all-targets

# Analyze binary bloat
cargo install cargo-bloat
cargo bloat --release --crates -n 20
```

### Build Profiling
```bash
# Time individual crate builds
time cargo build --package knhk-cli

# Profile with detailed timings
CARGO_PROFILE_RELEASE_LTO=off cargo build --release --timings

# Incremental compilation analysis
CARGO_LOG=cargo::core::compiler::fingerprint=trace cargo build
```

---

*Last Updated: 2025-11-07*
*Next Review: After Phase 1 completion*
