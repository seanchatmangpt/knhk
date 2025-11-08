# KNHK Compilation Performance Benchmark (In Progress)

**Status:** 🔄 Benchmark running (5/13 packages completed)
**Started:** 2025-11-07 17:41:51
**Current Package:** knhk-unrdf (5,310 LOC, 700 transitive deps)

## Completed Results (4 packages)

| Package | LOC | Files | Direct Deps | Trans Deps | Clean Build | Incremental | Test Build |
|---------|----:|------:|------------:|-----------:|------------:|------------:|-----------:|
| **knhk-lockchain** | 1,097 | 4 | 0 | 140 | **68.37s** | 1.37s | 34.37s |
| knhk-otel | 1,577 | 2 | 0 | 264 | 22.12s | 2.04s | **100.70s** |
| knhk-hot | 1,867 | 8 | 0 | 6 | 11.29s | 2.54s | 1.05s |
| knhk-connectors | 2,378 | 3 | 0 | 1 | **0.90s** | 1.31s | 89.64s |

## Early Observations

### 🐌 Slowest Clean Builds
1. **knhk-lockchain**: 68.37s (1,097 LOC, 140 deps)
2. **knhk-otel**: 22.12s (1,577 LOC, 264 deps)
3. **knhk-hot**: 11.29s (1,867 LOC, 6 deps)

**Key Insight:** Build time strongly correlates with dependency count, not LOC!
- knhk-lockchain: 62ms per LOC (but 140 deps!)
- knhk-hot: 6ms per LOC (only 6 deps)
- knhk-connectors: 0.4ms per LOC (only 1 dep!)

### 🐢 Slowest Test Compilations
1. **knhk-otel**: 100.70s (264 transitive deps!)
2. **knhk-connectors**: 89.64s
3. **knhk-lockchain**: 34.37s

### ⚡ Best Incremental Performance
All packages show excellent incremental rebuild times (1-2.5s), indicating:
- ✅ Good module structure
- ✅ Effective incremental compilation
- ✅ Limited cross-module dependencies

### 🔍 Dependency Analysis

**Correlation: Transitive Deps → Build Time**

| Package | Trans Deps | Clean Build | ms/dep |
|---------|----------:|------------:|-------:|
| knhk-connectors | 1 | 0.90s | 896ms |
| knhk-hot | 6 | 11.29s | 1,882ms |
| knhk-lockchain | 140 | 68.37s | 488ms |
| knhk-otel | 264 | 22.12s | 84ms |

**Anomaly:** knhk-lockchain has unusually high ms/dep (488ms), suggesting:
- Complex dependency tree structure
- Possible optimization opportunity
- May benefit from dependency reduction

## Pending Packages

Still benchmarking (9 packages remaining):
- **knhk-unrdf** (currently running - 5,310 LOC, **700 deps!**)
- knhk-etl
- knhk-warm
- knhk-aot
- knhk-validation
- knhk-config
- knhk-patterns
- knhk-cli
- knhk-integration-tests

### Expected Concerns

**knhk-unrdf** (currently running):
- Largest codebase seen so far: 5,310 LOC
- **Massive dependency count: 700 transitive deps**
- Expected build time: **80-120+ seconds**
- Likely to be the slowest package overall

## Preliminary Recommendations

Based on partial data:

### 1. High-Priority Optimizations

**knhk-lockchain** (68.37s build):
- ⚠️ Review 140 transitive dependencies
- Consider splitting into smaller crates
- Identify unnecessary dependencies with `cargo tree`

**knhk-otel** (100.70s test compilation):
- ⚠️ 264 transitive dependencies is excessive
- Test compilation 4.5x slower than clean build
- Consider reducing OpenTelemetry feature flags

### 2. Dependency Management

All packages show `0 direct dependencies` in metadata, suggesting:
- ✅ Good use of workspace dependencies
- ✅ Centralized dependency management
- ℹ️ All deps inherited from workspace Cargo.toml

### 3. Workspace Optimization Opportunities

**Parallel Build Strategy:**
```bash
# Build fast packages first (unlock dependents)
cargo build -p knhk-connectors  # 0.9s
cargo build -p knhk-hot          # 11.3s

# Then slow packages
cargo build -p knhk-lockchain    # 68.4s
cargo build -p knhk-otel         # 22.1s

# Use parallel jobs
CARGO_BUILD_JOBS=8 cargo build --workspace
```

**Profiling Command:**
```bash
cargo build --workspace --timings
# Generates HTML report in target/cargo-timings/
```

## Next Steps

1. ⏳ Wait for remaining 9 packages to complete
2. 📊 Generate comprehensive performance profiles
3. 🔍 Detailed dependency tree analysis for slow packages
4. 📈 Create optimization priority matrix
5. 🎯 Generate actionable remediation plan

---

**Note:** Full report will be generated automatically when benchmark completes.

**Refresh Status:**
```bash
/Users/sac/knhk/rust/scripts/partial_benchmark_analysis.sh
```
