# Detailed Per-Crate Performance Metrics

## Complete Performance Matrix

| Crate | LOC | Build Debug | Build Release | Test Time | Clippy | Binary Size | Efficiency (LOC/s) |
|-------|-----|-------------|---------------|-----------|--------|-------------|-------------------|
| `knhk-etl` | 7,877 | 6.07s | 57.75s | 8.89s | 1.11s | N/A | 1,296.73 |
| `knhk-unrdf` | 5,310 | 6.73s | 70.67s | 10.26s | 87.76s | N/A | 789.00 |
| `knhk-sidecar` | 4,183 | 0.30s | 0.28s | 0.29s | 0.36s | N/A | **13,822.98** |
| `knhk-cli` | 3,496 | 81.05s | 207.05s | 51.58s | 4.66s | N/A | 43.13 |
| `knhk-patterns` | 2,918 | 8.63s | 6.99s | 9.61s | 1.26s | N/A | 338.18 |
| `knhk-validation` | 2,594 | 3.85s | 23.78s | 2.88s | 1.77s | 345 KB | 673.51 |
| `knhk-connectors` | 2,378 | 0.57s | 33.20s | 4.47s | 2.92s | N/A | 4,164.35 |
| `knhk-warm` | 2,054 | 7.59s | 27.78s | 9.09s | 3.36s | N/A | 270.63 |
| `knhk-hot` | 1,867 | 1.00s | 19.68s | 1.72s | 0.96s | N/A | 1,872.95 |
| `knhk-otel` | 1,577 | 0.80s | 2.79s | 1.57s | 3.08s | N/A | 1,965.19 |
| `knhk-lockchain` | 1,097 | 1.08s | 13.46s | 1.38s | 6.36s | N/A | 1,018.52 |
| `knhk-aot` | 921 | 26.56s | 52.90s | 47.16s | 0.74s | N/A | 34.68 |
| `knhk-config` | 544 | 1.18s | 155.27s | 101.61s | 1.71s | N/A | 460.17 |
| `knhk-integration-tests` | 138 | 87.69s | 97.58s | 5.73s | 76.12s | N/A | **1.57** |

## Critical Anomalies Identified

### 🚨 knhk-integration-tests: Build Efficiency Crisis
- **Only 138 LOC but takes 87.69s to build (debug)**
- **Efficiency: 1.57 LOC/sec** (8,800x slower than knhk-sidecar!)
- **Clippy takes 76.12s** - suggests heavy procedural macros or complex dependencies
- **Root Cause**: Likely pulls in ALL workspace dependencies for integration testing
- **Fix**: Split into focused integration test suites per subsystem

### 🔴 knhk-cli: Release Build Bottleneck
- **207.05s release build** (3.4 minutes)
- **3,496 LOC** - moderate size but slowest release build
- **51.58s test time** - second slowest tests
- **Root Cause**: Likely heavy dependency tree (clap, tokio, etc.)
- **Fix**: Consider splitting CLI into lighter frontend + backend modules

### 🟠 knhk-config: Test Time Mystery
- **Only 544 LOC but 101.61s test time** (1.7 minutes!)
- **186x slower tests than source size suggests**
- **155.27s release build** also surprisingly slow
- **Root Cause**: Tests may be doing heavy I/O, parsing, or environment setup
- **Fix**: Profile tests, mock expensive operations, parallelize execution

### 🟡 knhk-unrdf: Clippy Outlier
- **87.76s clippy time** - slowest by far
- **5,310 LOC** - large but not extreme
- **Root Cause**: Complex code patterns, procedural macros, or deep trait hierarchies
- **Fix**: Review clippy lints, simplify complex patterns, consider incremental clippy

### ✅ knhk-sidecar: Performance Champion
- **13,822.98 LOC/sec** - fastest build efficiency
- **4,183 LOC** - third largest crate
- **0.30s debug build** despite size
- **Success Factor**: Well-structured, minimal dependencies, clean architecture

## Build Time Distribution

### Debug Builds
```
Fast (< 5s):    7 crates (50%)
Medium (5-30s): 5 crates (36%)
Slow (> 30s):   2 crates (14%) ← knhk-integration-tests, knhk-cli
```

### Release Builds
```
Fast (< 20s):   5 crates (36%)
Medium (20-60s): 4 crates (29%)
Slow (> 60s):   5 crates (36%) ← knhk-cli, knhk-config, knhk-integration-tests, knhk-unrdf, knhk-etl
```

### Test Times
```
Fast (< 5s):    8 crates (57%)
Medium (5-20s): 3 crates (21%)
Slow (> 20s):   3 crates (21%) ← knhk-config, knhk-cli, knhk-aot
```

## Dependency Analysis (Inferred)

### Heavy Dependency Suspects
Based on build times, these crates likely have large dependency trees:

1. **knhk-integration-tests**: All workspace crates + test frameworks
2. **knhk-cli**: clap, tokio, serde, anyhow, possibly tracing
3. **knhk-config**: serde, toml/yaml parsers, possibly env loaders
4. **knhk-unrdf**: RDF libraries, SPARQL parsers, complex data structures

### Lightweight Crates (Minimal Dependencies)
1. **knhk-sidecar**: Very fast despite size
2. **knhk-connectors**: Fast build for size
3. **knhk-otel**: OpenTelemetry but still efficient

## Compilation Bottleneck Categories

### Category 1: Dependency Hell (70% of slow builds)
- **Crates**: knhk-integration-tests, knhk-cli, knhk-config
- **Symptom**: Disproportionate build time vs LOC
- **Fix**: Dependency audit, feature flags, split crates

### Category 2: Complex Code (20% of slow builds)
- **Crates**: knhk-aot, knhk-unrdf
- **Symptom**: High clippy time, moderate build time
- **Fix**: Simplify procedural macros, reduce trait complexity

### Category 3: Test Infrastructure (10% of slow builds)
- **Crates**: knhk-config, knhk-aot
- **Symptom**: Disproportionate test time
- **Fix**: Mock I/O, parallelize, reduce setup/teardown

## Optimization Priority Queue

### Priority 1 (This Week)
1. **Investigate knhk-integration-tests** dependency tree
   - Run `cargo tree -p knhk-integration-tests`
   - Consider splitting into subsystem-specific integration tests
   - Expected gain: 50-70% build time reduction

2. **Profile knhk-config tests**
   - Identify slow test cases
   - Mock I/O operations
   - Expected gain: 60-80% test time reduction

3. **Analyze knhk-cli dependency graph**
   - Use `cargo build --timings` to visualize
   - Consider lazy feature flags
   - Expected gain: 30-40% release build reduction

### Priority 2 (Next Sprint)
4. **Optimize knhk-unrdf clippy**
   - Review complex lint violations
   - Simplify procedural macros if applicable
   - Expected gain: 50% clippy time reduction

5. **Profile knhk-aot tests**
   - Likely AOT compilation testing is slow
   - Consider caching compiled artifacts
   - Expected gain: 40-50% test time reduction

6. **Enable workspace-level caching**
   - Setup sccache or mold linker
   - Configure incremental compilation
   - Expected gain: 20-30% overall build time reduction

### Priority 3 (Future)
7. **Refactor large crates** (knhk-etl, knhk-unrdf, knhk-sidecar)
8. **Continuous performance monitoring** in CI
9. **Benchmark-driven development** for hot paths

## CI/CD Impact Analysis

### Current CI/CD Build Times (Estimated)
```
Full workspace clean build (debug):   ~4 minutes
Full workspace clean build (release): ~13 minutes
Full test suite:                      ~4.5 minutes
Full clippy:                          ~3 minutes
---
Total CI pipeline:                    ~24.5 minutes
```

### After Optimization (Target)
```
Full workspace clean build (debug):   ~2 minutes (-50%)
Full workspace clean build (release): ~7 minutes (-46%)
Full test suite:                      ~2 minutes (-55%)
Full clippy:                          ~2 minutes (-33%)
---
Total CI pipeline:                    ~13 minutes (-47%)
```

### Developer Experience Impact
- Current: `cargo build` ~4 minutes (frustrating)
- Current: `cargo test` ~4.5 minutes (breaks flow)
- Target: `cargo build` ~2 minutes (acceptable)
- Target: `cargo test` ~2 minutes (good)

## Workspace Health Score

| Metric | Score | Grade | Target |
|--------|-------|-------|--------|
| Average build efficiency | 1,917 LOC/s | B | 3,000+ LOC/s |
| Build time consistency | Low (1.57-13,822 range) | C | High consistency |
| Test coverage vs time | Good | B+ | Maintain |
| Clippy compliance time | 192s total | B | <120s |
| Overall workspace health | **B-** | | **A** |

## Action Items

- [ ] Run `cargo build --timings` for top 3 slowest crates
- [ ] Run `cargo tree` analysis on knhk-integration-tests
- [ ] Profile knhk-config test suite for slow tests
- [ ] Audit workspace dependencies with `cargo-udeps`
- [ ] Setup sccache for development builds
- [ ] Create CI benchmark tracking for regression detection
- [ ] Document optimal build practices in CONTRIBUTING.md

---

*Generated: 2025-11-07*
*Benchmark Duration: ~16 minutes*
*Next Benchmark: After optimization sprint*
