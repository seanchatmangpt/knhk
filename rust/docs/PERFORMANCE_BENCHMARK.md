# KNHK Monorepo Performance Benchmark Report

## Workspace Summary

- **Total Crates**: 14
- **Total Lines of Code**: 36,954
- **Average LOC per Crate**: 2,640

### Build Times

| Metric | Debug | Release |
|--------|-------|---------|
| **Total Time** | 233.11s (3.89m) | 769.19s (12.82m) |
| **Average per Crate** | 16.65s | 54.94s |

### Test & Quality Times

| Metric | Time |
|--------|------|
| **Total Test Time** | 256.25s (4.27m) |
| **Total Clippy Time** | 192.18s |
| **Average Test Time** | 18.30s |
| **Average Clippy Time** | 13.73s |

---

## Top 5 Slowest Builds (Debug)

| Rank | Crate | Build Time |
|------|-------|------------|
| 1 | `knhk-integration-tests` | 87.69s |
| 2 | `knhk-cli` | 81.05s |
| 3 | `knhk-aot` | 26.56s |
| 4 | `knhk-patterns` | 8.63s |
| 5 | `knhk-warm` | 7.59s |

---

## Top 5 Slowest Builds (Release)

| Rank | Crate | Build Time |
|------|-------|------------|
| 1 | `knhk-cli` | 207.05s |
| 2 | `knhk-config` | 155.27s |
| 3 | `knhk-integration-tests` | 97.58s |
| 4 | `knhk-unrdf` | 70.67s |
| 5 | `knhk-etl` | 57.75s |

---

## Top 5 Slowest Tests

| Rank | Crate | Test Time |
|------|-------|-----------|
| 1 | `knhk-config` | 101.61s |
| 2 | `knhk-cli` | 51.58s |
| 3 | `knhk-aot` | 47.16s |
| 4 | `knhk-unrdf` | 10.26s |
| 5 | `knhk-patterns` | 9.61s |

---

## Top 5 Largest Crates

| Rank | Crate | Lines of Code |
|------|-------|---------------|
| 1 | `knhk-etl` | 7,877 |
| 2 | `knhk-unrdf` | 5,310 |
| 3 | `knhk-sidecar` | 4,183 |
| 4 | `knhk-cli` | 3,496 |
| 5 | `knhk-patterns` | 2,918 |

---

## Build Efficiency (LOC/sec)

### Most Efficient (Debug Build)

| Rank | Crate | LOC/sec |
|------|-------|---------|
| 1 | `knhk-sidecar` | 13822.98 |
| 2 | `knhk-connectors` | 4164.35 |
| 3 | `knhk-otel` | 1965.19 |
| 4 | `knhk-hot` | 1872.95 |
| 5 | `knhk-etl` | 1296.73 |

### Least Efficient (Debug Build)

| Rank | Crate | LOC/sec |
|------|-------|---------|
| 1 | `knhk-integration-tests` | 1.57 |
| 2 | `knhk-aot` | 34.68 |
| 3 | `knhk-cli` | 43.13 |
| 4 | `knhk-warm` | 270.63 |
| 5 | `knhk-patterns` | 338.18 |

---

## Optimization Opportunities

### 🔴 Critical (High Impact)

1. **Optimize slowest builds**: `knhk-integration-tests`, `knhk-cli`, `knhk-aot`
   - Consider splitting large crates
   - Review dependency graph for circular dependencies
   - Enable incremental compilation features

2. **Optimize slow tests**: `knhk-config`, `knhk-cli`, `knhk-aot`
   - Parallelize test execution
   - Mock expensive I/O operations
   - Review test setup/teardown overhead


### 🟡 Medium Impact

1. **Refactor large crates**: `knhk-etl`, `knhk-unrdf`, `knhk-sidecar`
   - Split into smaller, focused modules
   - Extract reusable components
   - Reduce coupling between modules

2. **Improve build efficiency**: `knhk-integration-tests`, `knhk-aot`, `knhk-cli`
   - Review complex procedural macros
   - Optimize dependency compilation
   - Consider feature flags for optional dependencies


### 🟢 Low Impact (Nice to Have)

1. **Dependency Optimization**
   - Audit unused dependencies
   - Use `cargo-udeps` to find unused deps
   - Consider workspace-level dependency deduplication

2. **Compilation Cache**
   - Enable `sccache` or `mold` linker
   - Use build caching in CI/CD
   - Pre-compile common dependencies

---

## Recommendations

### Short-term (1-2 weeks)

1. **Profile slowest builds** using `cargo build --timings`
2. **Optimize test parallelization** for slow test suites
3. **Review dependency graph** for optimization opportunities

### Medium-term (1-2 months)

1. **Refactor largest crates** into smaller, focused modules
2. **Implement build caching** in development and CI
3. **Benchmark critical paths** for performance regressions

### Long-term (3-6 months)

1. **Continuous performance monitoring** in CI/CD
2. **Automated performance regression detection**
3. **Developer tooling improvements** (IDE integration, build scripts)

---

*Generated from KNHK workspace benchmark run on /Users/sac/knhk/rust*
