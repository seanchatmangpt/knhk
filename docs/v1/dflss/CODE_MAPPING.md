# KNHK v1.0 DFLSS Code Mapping

**Direct Mapping of DFLSS Documentation to Code Files**

This document maps the DFLSS (Design For Lean Six Sigma) documentation directly to implementation files and code references in the KNHK codebase.

---

## Overview

**Purpose**: Provide direct code references for all DFLSS documentation  
**Scope**: Map all DFLSS phases, CTQ requirements, and VOC statements to code  
**Last Updated**: 2025-11-09

---

## DFLSS Phase → Code Mapping

### DEFINE Phase

#### Project Charter → Code Structure

| Charter Section | Code Reference | File Path |
|----------------|----------------|-----------|
| **Core Engine** | `WorkflowEngine` | `rust/knhk-workflow-engine/src/executor/engine.rs` |
| **Workflow Parser** | `WorkflowParser` | `rust/knhk-workflow-engine/src/parser/mod.rs` |
| **State Management** | `StateManager`, `StateStore` | `rust/knhk-workflow-engine/src/state/manager.rs`, `rust/knhk-workflow-engine/src/state/store.rs` |
| **Pattern Registry** | `PatternRegistry` | `rust/knhk-workflow-engine/src/patterns/mod.rs` |
| **API Layer** | REST, gRPC, CLI | `rust/knhk-workflow-engine/src/api/rest/`, `rust/knhk-workflow-engine/src/api/grpc.rs`, `rust/knhk-cli/src/` |

#### CTQ Requirements → Implementation Files

| CTQ Metric | Implementation | Code Reference |
|------------|----------------|---------------|
| **Weaver Validation** | Weaver integration | `rust/knhk-workflow-engine/src/integration/weaver.rs` |
| **Performance ≤8 ticks** | Hot path operations | `rust/knhk-workflow-engine/src/performance/aot.rs` |
| **DoD Compliance** | Validation framework | `rust/knhk-workflow-engine/src/validation/mod.rs` |
| **Zero Warnings** | Clippy configuration | `rust/knhk-workflow-engine/src/lib.rs:54-55` |
| **Process Capability** | Performance metrics | `rust/knhk-workflow-engine/src/performance/mod.rs` |

---

### MEASURE Phase

#### Performance Metrics → Code Files

| Metric | Measurement Code | File Path |
|--------|------------------|-----------|
| **Hot Path Ticks** | RDTSC measurement | `rust/knhk-hot/src/lib.rs` |
| **Performance Benchmarks** | Benchmark tests | `rust/knhk-workflow-engine/tests/performance/` |
| **Process Capability (Cp/Cpk)** | Statistical analysis | `rust/knhk-workflow-engine/src/validation/process_mining.rs` |
| **Weaver Validation** | Schema validation | `rust/knhk-workflow-engine/src/integration/weaver.rs` |
| **Test Coverage** | Coverage analysis | `rust/knhk-workflow-engine/src/testing/coverage.rs` |

#### Defect Tracking → Code Files

| Defect Type | Detection Code | File Path |
|-------------|----------------|-----------|
| **Clippy Errors** | Pre-commit hooks | `.git/hooks/pre-commit` |
| **Unwrap/Expect** | Static analysis | `rust/knhk-workflow-engine/src/lib.rs:54-55` |
| **Compilation Warnings** | Cargo check | `Makefile` (check targets) |
| **Test Failures** | Test execution | `rust/knhk-workflow-engine/tests/` |

---

### ANALYZE Phase

#### Root Cause Analysis → Code Files

| Root Cause | Analysis Code | File Path |
|------------|---------------|-----------|
| **Clippy Errors** | Clippy configuration | `rust/knhk-workflow-engine/src/lib.rs:54-55` |
| **Chicago TDD Crash** | Test framework | `rust/knhk-workflow-engine/src/testing/chicago_tdd.rs` |
| **Integration Tests** | Test infrastructure | `rust/knhk-workflow-engine/tests/integration/` |
| **Unwrap Usage** | Error handling | `rust/knhk-workflow-engine/src/error/mod.rs` |
| **Performance Issues** | Hot path analysis | `rust/knhk-workflow-engine/src/performance/aot.rs` |

#### Pareto Analysis → Code Files

| Category | Analysis Code | File Path |
|----------|---------------|-----------|
| **80/20 Analysis** | Pattern usage | `rust/knhk-workflow-engine/src/patterns/mod.rs` |
| **Critical Path** | Hot path operations | `rust/knhk-workflow-engine/src/performance/aot.rs` |
| **Error Distribution** | Error handling | `rust/knhk-workflow-engine/src/error/mod.rs` |

---

### IMPROVE Phase

#### Fixes → Code Files

| Fix | Implementation | File Path |
|-----|----------------|-----------|
| **Fix Clippy Errors** | Code refactoring | `rust/knhk-workflow-engine/src/` (all files) |
| **Fix Chicago TDD** | Test framework | `rust/knhk-workflow-engine/src/testing/chicago_tdd.rs` |
| **Fix Integration Tests** | Test infrastructure | `rust/knhk-workflow-engine/tests/integration/` |
| **Remove Unwrap** | Error handling | `rust/knhk-workflow-engine/src/error/mod.rs` |
| **Optimize Performance** | Hot path optimization | `rust/knhk-workflow-engine/src/performance/aot.rs` |

#### Optimizations → Code Files

| Optimization | Implementation | File Path |
|--------------|----------------|-----------|
| **Hot Path ≤8 ticks** | SIMD operations | `rust/knhk-hot/src/lib.rs` |
| **Zero-Copy Operations** | Reference usage | `rust/knhk-workflow-engine/src/executor/` |
| **Branchless Operations** | Constant-time execution | `rust/knhk-workflow-engine/src/performance/aot.rs` |
| **Ingress Validation** | Guard constraints at data entry | `rust/knhk-workflow-engine/src/security/guards.rs`, `rust/knhk-workflow-engine/src/services/admission.rs` |

**Architecture**: Centralized validation at ingress. Pure execution in hot path:
- **Ingress**: `knhk-workflow-engine` - ALL data enters here, domain logic and validation
  - `create_case` (admission_gate), `register_workflow` (deadlock detector), API/CLI
  - Guards: `security/guards.rs` - ONLY place for input validation
- **Execution**: `knhk-hot` - Pure execution, NO checks, assumes pre-validated inputs
  - Hot path operations, ring buffers, kernels
  - All validation happens in knhk-workflow-engine before calling knhk-hot
- **Principle**: knhk-workflow-engine = ingress validation, knhk-hot = pure execution

---

### CONTROL Phase

#### Control Mechanisms → Code Files

| Control | Implementation | File Path |
|---------|----------------|-----------|
| **CI/CD Gates** | GitHub Actions | `.github/workflows/` |
| **Pre-commit Hooks** | Validation | `.git/hooks/pre-commit` |
| **Pre-push Hooks** | Validation | `.git/hooks/pre-push` |
| **SPC Charts** | DFLSS CLI | `rust/knhk-dflss/src/commands/charts.rs` |
| **Metrics Collection** | DFLSS CLI | `rust/knhk-dflss/src/commands/metrics.rs` |
| **Process Capability** | DFLSS CLI | `rust/knhk-dflss/src/commands/capability.rs` |
| **CTQ Validation** | DFLSS CLI | `rust/knhk-dflss/src/commands/validation.rs` |
| **Automated Testing** | Test suite | `rust/knhk-workflow-engine/tests/` |

---

## SIPOC → Code Mapping

### SUPPLIERS → Code Dependencies

| Supplier | Dependency | File Path |
|----------|------------|-----------|
| **Rust Compiler** | `rustc`, `cargo` | `rust/Cargo.toml` |
| **OpenTelemetry** | `opentelemetry` crates | `rust/knhk-workflow-engine/Cargo.toml` |
| **Weaver** | External tool | `vendors/weaver/` |
| **Docker** | Container runtime | `Dockerfile` |

### INPUTS → Source Files

| Input | Source Files | File Path |
|-------|--------------|-----------|
| **Rust Source Code** | All Rust files | `rust/knhk-workflow-engine/src/` |
| **C Source Code** | C library files | `c/` |
| **OTel Schemas** | Schema files | `vendors/weaver/registry/` |
| **Configuration** | Config files | `rust/knhk-config/` |

### PROCESS → Implementation Modules

| Process Step | Implementation Module | File Path |
|--------------|----------------------|-----------|
| **DEFINE** | Documentation | `docs/v1/dflss/` |
| **MEASURE** | Metrics collection | `rust/knhk-workflow-engine/src/validation/` |
| **ANALYZE** | Analysis tools | `rust/knhk-workflow-engine/src/testing/` |
| **IMPROVE** | Code fixes | `rust/knhk-workflow-engine/src/` |
| **CONTROL** | CI/CD, hooks | `.github/workflows/`, `.git/hooks/` |

### OUTPUTS → Artifacts

| Output | Artifact Location | File Path |
|-------|-------------------|-----------|
| **Documentation** | DFLSS docs | `docs/v1/dflss/` |
| **Code Artifacts** | Source code | `rust/knhk-workflow-engine/src/` |
| **Test Results** | Test reports | `reports/` |
| **Evidence** | Evidence archive | `docs/v1/evidence/` |

### CUSTOMERS → API Interfaces

| Customer | API Interface | File Path |
|----------|---------------|-----------|
| **End Users** | REST API | `rust/knhk-workflow-engine/src/api/rest/` |
| **Developers** | CLI | `rust/knhk-cli/src/` |
| **Services** | gRPC API | `rust/knhk-workflow-engine/src/api/grpc.rs` |

---

## VOC → Code Mapping

### Customer Need: "Tests must prove features work"

| VOC Statement | Implementation | Code Reference |
|---------------|----------------|----------------|
| **Weaver Validation** | Schema validation | `rust/knhk-workflow-engine/src/integration/weaver.rs` |
| **Functional Validation** | Test execution | `rust/knhk-workflow-engine/tests/` |
| **Evidence Collection** | Test reports | `reports/` |

### Customer Need: "Zero overhead performance"

| VOC Statement | Implementation | Code Reference |
|---------------|----------------|----------------|
| **≤8 Tick Requirement** | Hot path operations | `rust/knhk-hot/src/lib.rs` |
| **RDTSC Measurement** | Performance benchmarks | `rust/knhk-workflow-engine/tests/performance/` |
| **Performance Validation** | Validation framework | `rust/knhk-workflow-engine/src/validation/process_mining.rs` |

### Customer Need: "Production-ready quality"

| VOC Statement | Implementation | Code Reference |
|---------------|----------------|----------------|
| **DoD Compliance** | Validation framework | `rust/knhk-workflow-engine/src/validation/mod.rs` |
| **Zero Warnings** | Clippy configuration | `rust/knhk-workflow-engine/src/lib.rs:54-55` |
| **Error Handling** | Error management | `rust/knhk-workflow-engine/src/error/mod.rs` |

---

## CTQ → Code Mapping

### CTQ 1: Weaver Validation (100% pass rate)

| CTQ Metric | Implementation | Code Reference |
|------------|----------------|----------------|
| **Static Validation** | Schema checking | `rust/knhk-workflow-engine/src/integration/weaver.rs` |
| **Live Validation** | Runtime validation | `rust/knhk-workflow-engine/src/integration/weaver.rs` |
| **Schema Registry** | OTel schemas | `registry/` |
| **Weaver Examples** | Validation examples | `rust/knhk-workflow-engine/examples/weaver_real_jtbd_validation.rs` |
| **OTEL Live Check** | Live validation | `rust/knhk-otel/examples/weaver_live_check.rs` |

**Key Files**:
- ```1:267:rust/knhk-workflow-engine/src/integration/weaver.rs```
- ```1:50:registry/knhk-attributes.yaml```
- ```1:100:rust/knhk-workflow-engine/examples/weaver_real_jtbd_validation.rs```
- ```1:100:rust/knhk-otel/examples/weaver_live_check.rs```

### CTQ 2: Performance (≤8 ticks)

| CTQ Metric | Implementation | Code Reference |
|------------|----------------|----------------|
| **Hot Path Operations** | Hot path engine | `rust/knhk-hot/src/lib.rs` |
| **RDTSC Measurement** | Cycle counting | `c/tools/knhk_bench.c` |
| **CONSTRUCT8 Implementation** | Warm path CONSTRUCT8 | `rust/knhk-warm/src/construct8.rs` |
| **Performance Benchmarks** | Benchmark tests | `rust/knhk-hot/benches/cycle_bench.rs` |
| **Tick Budget Validation** | Reflex stage | `rust/knhk-etl/src/reflex.rs` |

**Key Files**:
- ```1:100:rust/knhk-hot/src/lib.rs```
- ```1:220:c/tools/knhk_bench.c```
- ```1:173:rust/knhk-warm/src/construct8.rs```
- ```1:100:rust/knhk-hot/benches/cycle_bench.rs```
- ```1:200:rust/knhk-etl/src/reflex.rs``` (tick_budget: u32 = 8)

### CTQ 3: DoD Compliance (≥85%)

| CTQ Metric | Implementation | Code Reference |
|------------|----------------|----------------|
| **Validation Framework** | Validation module | `rust/knhk-workflow-engine/src/validation/mod.rs` |
| **Test Coverage** | Coverage analysis | `rust/knhk-workflow-engine/src/testing/coverage.rs` |
| **DoD Checklist** | Documentation | `docs/v1/definition-of-done/fortune5-production.md` |

**Key Files**:
- ```1:100:rust/knhk-workflow-engine/src/validation/mod.rs```
- ```1:50:rust/knhk-workflow-engine/src/testing/coverage.rs```

### CTQ 4: Zero Warnings

| CTQ Metric | Implementation | Code Reference |
|------------|----------------|----------------|
| **Clippy Configuration** | Lint settings | `rust/knhk-workflow-engine/src/lib.rs:54-55` |
| **Pre-commit Hooks** | Validation | `.git/hooks/pre-commit` |
| **Pre-push Hooks** | Validation | `.git/hooks/pre-push` |

**Key Files**:
- ```54:55:rust/knhk-workflow-engine/src/lib.rs```
- ```1:50:.git/hooks/pre-commit```

### CTQ 5: Process Capability (Cpk ≥1.67)

| CTQ Metric | Implementation | Code Reference |
|------------|----------------|----------------|
| **Statistical Analysis** | Process mining | `rust/knhk-workflow-engine/src/validation/process_mining.rs` |
| **Metrics Collection** | Performance metrics | `rust/knhk-workflow-engine/src/performance/mod.rs` |
| **SPC Charts** | Control charts | `rust/knhk-workflow-engine/src/validation/process_mining.rs` |

**Key Files**:
- ```1:100:rust/knhk-workflow-engine/src/validation/process_mining.rs```
- ```1:50:rust/knhk-workflow-engine/src/performance/mod.rs```

---

## Critical Blockers → Code Files

### Blocker 1: Clippy Errors (15+)

| Error Type | Affected Files | Code Reference |
|------------|----------------|----------------|
| **Deprecated API** | SPARQL queries | `rust/knhk-workflow-engine/src/data/gateway.rs:167-181` |
| **Unused Variables** | Multiple files | `rust/knhk-workflow-engine/src/` (various) |
| **Type Complexity** | Pattern code | `rust/knhk-workflow-engine/src/patterns/` |

**Key Files**:
- ```167:181:rust/knhk-workflow-engine/src/data/gateway.rs```
- ```1:50:rust/knhk-workflow-engine/src/patterns/mod.rs```

### Blocker 2: Chicago TDD Crash

| Issue | Affected Code | Code Reference |
|-------|---------------|----------------|
| **Test Framework** | Chicago TDD implementation | `rust/knhk-workflow-engine/src/testing/chicago_tdd.rs` |
| **Memory Safety** | Test execution | `rust/knhk-workflow-engine/tests/chicago_tdd_*.rs` |
| **Test Tools** | Chicago TDD tools | `rust/chicago-tdd-tools/` |
| **E2E Tests** | End-to-end validation | `rust/knhk-workflow-engine/tests/chicago_tdd_financial_e2e.rs` |

**Key Files**:
- ```1:1471:rust/knhk-workflow-engine/src/testing/chicago_tdd.rs```
- ```1:100:rust/knhk-workflow-engine/tests/chicago_tdd_difficult_patterns.rs```
- ```1:50:rust/chicago-tdd-tools/src/lib.rs```
- ```1:100:rust/knhk-workflow-engine/tests/chicago_tdd_financial_e2e.rs```

### Blocker 3: Integration Tests

| Issue | Affected Code | Code Reference |
|-------|---------------|----------------|
| **Compilation Errors** | Test infrastructure | `rust/knhk-workflow-engine/tests/integration/` |
| **Missing Dependencies** | Test setup | `rust/knhk-workflow-engine/tests/integration/` |

**Key Files**:
- ```1:50:rust/knhk-workflow-engine/tests/integration/mod.rs```

### Blocker 4: Unwrap in Hot Path

| Issue | Affected Code | Code Reference |
|-------|---------------|----------------|
| **Unwrap Usage** | Hot path operations | `rust/knhk-hot/src/lib.rs` |
| **Error Handling** | Error management | `rust/knhk-workflow-engine/src/error/mod.rs` |
| **Reflex Stage** | Tick budget validation | `rust/knhk-etl/src/reflex.rs` |
| **Warm Path** | CONSTRUCT8 error handling | `rust/knhk-warm/src/construct8.rs` |

**Key Files**:
- ```1:100:rust/knhk-hot/src/lib.rs```
- ```1:50:rust/knhk-workflow-engine/src/error/mod.rs```
- ```1:200:rust/knhk-etl/src/reflex.rs```
- ```1:173:rust/knhk-warm/src/construct8.rs```

---

## Implementation Modules → DFLSS Phases

### Core Engine Module

| Module | DFLSS Phase | Code Reference |
|--------|-------------|----------------|
| **WorkflowEngine** | IMPROVE | `rust/knhk-workflow-engine/src/executor/engine.rs` |
| **StateManager** | CONTROL | `rust/knhk-workflow-engine/src/state/manager.rs` |
| **PatternRegistry** | IMPROVE | `rust/knhk-workflow-engine/src/patterns/mod.rs` |

### Validation Module

| Module | DFLSS Phase | Code Reference |
|--------|-------------|----------------|
| **ValidationFramework** | MEASURE | `rust/knhk-workflow-engine/src/validation/mod.rs` |
| **ProcessMining** | ANALYZE | `rust/knhk-workflow-engine/src/validation/process_mining.rs` |
| **CoverageAnalysis** | MEASURE | `rust/knhk-workflow-engine/src/testing/coverage.rs` |

### Performance Module

| Module | DFLSS Phase | Code Reference |
|--------|-------------|----------------|
| **HotPath** | IMPROVE | `rust/knhk-hot/src/lib.rs` |
| **WarmPath** | IMPROVE | `rust/knhk-warm/src/warm_path.rs` |
| **CONSTRUCT8** | IMPROVE | `rust/knhk-warm/src/construct8.rs` |
| **Benchmarks** | MEASURE | `rust/knhk-hot/benches/cycle_bench.rs` |
| **C Benchmarks** | MEASURE | `c/tools/knhk_bench.c` |

### Integration Module

| Module | DFLSS Phase | Code Reference |
|--------|-------------|----------------|
| **WeaverIntegration** | CONTROL | `rust/knhk-workflow-engine/src/integration/weaver.rs` |
| **OtelIntegration** | CONTROL | `rust/knhk-workflow-engine/src/integration/otel.rs` |
| **Fortune5Integration** | CONTROL | `rust/knhk-workflow-engine/src/integration/fortune5.rs` |

---

## Test Files → DFLSS Phases

### Chicago TDD Tests

| Test File | DFLSS Phase | Code Reference |
|-----------|-------------|----------------|
| **Basic Patterns** | ANALYZE | `rust/knhk-workflow-engine/tests/chicago_tdd_basic_patterns.rs` |
| **Difficult Patterns** | ANALYZE | `rust/knhk-workflow-engine/tests/chicago_tdd_difficult_patterns.rs` |
| **Test Framework** | IMPROVE | `rust/knhk-workflow-engine/src/testing/chicago_tdd.rs` |

### Performance Tests

| Test File | DFLSS Phase | Code Reference |
|-----------|-------------|----------------|
| **Hot Path Benchmarks** | MEASURE | `rust/knhk-workflow-engine/tests/performance/hot_path.rs` |
| **Performance Validation** | MEASURE | `rust/knhk-workflow-engine/tests/performance/` |

### Integration Tests

| Test File | DFLSS Phase | Code Reference |
|-----------|-------------|----------------|
| **API Tests** | CONTROL | `rust/knhk-workflow-engine/tests/integration/api.rs` |
| **Workflow Tests** | CONTROL | `rust/knhk-workflow-engine/tests/integration/workflow.rs` |

---

## Documentation → Code Mapping

### DFLSS Documents → Implementation

| Document | Related Code | Code Reference |
|----------|--------------|----------------|
| **PROJECT_CHARTER.md** | Core engine | `rust/knhk-workflow-engine/src/executor/engine.rs` |
| **SIPOC.md** | Process modules | `rust/knhk-workflow-engine/src/` |
| **SYNTHETIC_VOC.md** | API interfaces | `rust/knhk-workflow-engine/src/api/` |
| **define/PHASE_SUMMARY.md** | Parser, compiler | `rust/knhk-workflow-engine/src/parser/`, `rust/knhk-workflow-engine/src/compiler/` |
| **measure/PHASE_SUMMARY.md** | Validation, metrics | `rust/knhk-workflow-engine/src/validation/`, `rust/knhk-workflow-engine/src/performance/` |
| **analyze/PHASE_SUMMARY.md** | Analysis tools | `rust/knhk-workflow-engine/src/testing/` |
| **improve/PHASE_SUMMARY.md** | Code fixes | `rust/knhk-workflow-engine/src/` |
| **control/PHASE_SUMMARY.md** | CI/CD, hooks | `.github/workflows/`, `.git/hooks/` |

---

## Quick Reference: Code Locations

### Core Implementation

- **Workflow Engine**: `rust/knhk-workflow-engine/src/executor/engine.rs`
- **State Management**: `rust/knhk-workflow-engine/src/state/`
- **Pattern Registry**: `rust/knhk-workflow-engine/src/patterns/`
- **API Layer**: `rust/knhk-workflow-engine/src/api/`

### Validation & Testing

- **Validation Framework**: `rust/knhk-workflow-engine/src/validation/`
- **Chicago TDD**: `rust/knhk-workflow-engine/src/testing/chicago_tdd.rs`
- **Test Suite**: `rust/knhk-workflow-engine/tests/`
- **Performance Tests**: `rust/knhk-workflow-engine/tests/performance/`

### DFLSS Tools

- **DFLSS CLI**: `rust/knhk-dflss/src/main.rs`
- **Metrics Collection**: `rust/knhk-dflss/src/commands/metrics.rs`
- **SPC Charts**: `rust/knhk-dflss/src/commands/charts.rs`
- **Process Capability**: `rust/knhk-dflss/src/commands/capability.rs`
- **CTQ Validation**: `rust/knhk-dflss/src/commands/validation.rs`
- **Internal Modules**: `rust/knhk-dflss/src/internal/` (chart, capability, quality, statistics, rules)

### Performance & Optimization

- **Hot Path**: `rust/knhk-hot/src/lib.rs`
- **Warm Path**: `rust/knhk-warm/src/warm_path.rs`
- **CONSTRUCT8**: `rust/knhk-warm/src/construct8.rs`
- **C Benchmarks**: `c/tools/knhk_bench.c`
- **Rust Benchmarks**: `rust/knhk-hot/benches/cycle_bench.rs`
- **Tick Budget**: `rust/knhk-etl/src/reflex.rs` (tick_budget: u32 = 8)

### Integration & Control

- **Weaver Integration**: `rust/knhk-workflow-engine/src/integration/weaver.rs`
- **OTEL Integration**: `rust/knhk-workflow-engine/src/integration/otel.rs`
- **CI/CD**: `.github/workflows/`
- **Git Hooks**: `.git/hooks/`

---

## Direct File Mappings

### DFLSS Documents → Implementation Files

| DFLSS Document | Key Implementation Files | Code References |
|----------------|-------------------------|----------------|
| **PROJECT_CHARTER.md** | `rust/knhk-workflow-engine/src/lib.rs`, `rust/knhk-cli/src/main.rs` | ```1:142:rust/knhk-workflow-engine/src/lib.rs```, ```1:100:rust/knhk-cli/src/main.rs``` |
| **SIPOC.md** | `rust/knhk-workflow-engine/src/`, `c/src/` | ```1:69:rust/knhk-workflow-engine/src/executor/engine.rs```, ```1:100:c/src/knhk_core.c``` |
| **SYNTHETIC_VOC.md** | `rust/knhk-cli/src/commands/`, `rust/knhk-workflow-engine/src/api/` | ```1:100:rust/knhk-cli/src/commands/validate.rs```, ```1:50:rust/knhk-workflow-engine/src/api/rest/mod.rs``` |
| **define/PHASE_SUMMARY.md** | `rust/knhk-workflow-engine/src/parser/`, `rust/knhk-workflow-engine/src/executor/` | ```1:141:rust/knhk-workflow-engine/src/parser/mod.rs```, ```1:69:rust/knhk-workflow-engine/src/executor/engine.rs``` |
| **measure/PHASE_SUMMARY.md** | `rust/knhk-hot/benches/`, `c/tools/knhk_bench.c`, `rust/knhk-workflow-engine/src/integration/weaver.rs` | ```1:100:rust/knhk-hot/benches/cycle_bench.rs```, ```1:220:c/tools/knhk_bench.c```, ```1:267:rust/knhk-workflow-engine/src/integration/weaver.rs``` |
| **analyze/PHASE_SUMMARY.md** | `rust/knhk-workflow-engine/src/testing/chicago_tdd.rs`, `rust/knhk-workflow-engine/tests/` | ```1:1471:rust/knhk-workflow-engine/src/testing/chicago_tdd.rs```, ```1:100:rust/knhk-workflow-engine/tests/chicago_tdd_tools_integration.rs``` |
| **improve/PHASE_SUMMARY.md** | `rust/knhk-hot/src/`, `rust/knhk-warm/src/`, `rust/knhk-etl/src/` | ```1:33:rust/knhk-hot/src/lib.rs```, ```1:173:rust/knhk-warm/src/construct8.rs```, ```1:200:rust/knhk-etl/src/reflex.rs``` |
| **control/PHASE_SUMMARY.md** | `.github/workflows/`, `scripts/weaver-validate-all-43-patterns.sh` | ```1:50:.github/workflows/ci.yml```, ```1:100:scripts/weaver-validate-all-43-patterns.sh``` |

### Weaver Validation Files

| Validation Type | File Path |
|----------------|-----------|
| **Weaver Integration** | `rust/knhk-workflow-engine/src/integration/weaver.rs` |
| **Weaver Examples** | `rust/knhk-workflow-engine/examples/weaver_real_jtbd_validation.rs` |
| **OTEL Live Check** | `rust/knhk-otel/examples/weaver_live_check.rs` |
| **Weaver Scripts** | `scripts/weaver-validate-all-43-patterns.sh` |
| **Schema Registry** | `registry/knhk-attributes.yaml`, `registry/knhk-beat-v1.yaml` |

### Chicago TDD Files

| Test Type | File Path |
|-----------|-----------|
| **Test Framework** | `rust/knhk-workflow-engine/src/testing/chicago_tdd.rs` |
| **Test Tools** | `rust/chicago-tdd-tools/src/lib.rs` |
| **E2E Tests** | `rust/knhk-workflow-engine/tests/chicago_tdd_financial_e2e.rs` |
| **Pattern Tests** | `rust/knhk-workflow-engine/tests/chicago_tdd_difficult_patterns.rs` |
| **Integration Tests** | `rust/knhk-cli/tests/chicago_tdd_otel_e2e.rs` |

### Performance Files

| Performance Component | File Path |
|----------------------|-----------|
| **Hot Path** | `rust/knhk-hot/src/lib.rs` |
| **Warm Path** | `rust/knhk-warm/src/warm_path.rs` |
| **CONSTRUCT8** | `rust/knhk-warm/src/construct8.rs` |
| **C Benchmarks** | `c/tools/knhk_bench.c` |
| **Rust Benchmarks** | `rust/knhk-hot/benches/cycle_bench.rs` |
| **Tick Budget** | `rust/knhk-etl/src/reflex.rs` (line 22-23: `tick_budget: u32 = 8`) |

---

## Direct File-to-LOC Mapping

**Complete mapping of DFLSS documentation files to code files with line counts**

### DFLSS Documentation Files (docs/v1/dflss/)

| File | LOC | Description |
|------|-----|-------------|
| `CODE_MAPPING.md` | 452 | Direct mapping of DFLSS documentation to code files |
| `SYNTHETIC_VOC.md` | 411 | Voice of Customer analysis |
| `README.md` | 384 | DFLSS overview and DMEDI methodology |
| `measure/PHASE_SUMMARY.md` | 376 | MEASURE phase deliverables |
| `PROJECT_CHARTER.md` | 323 | Project scope, goals, team |
| `SIPOC.md` | 314 | Suppliers, Inputs, Process, Outputs, Customers |
| `define/PHASE_SUMMARY.md` | 264 | DEFINE phase deliverables |
| **Total Documentation** | **2,574** | All DFLSS documentation files |

### Core Workflow Engine Files (rust/knhk-workflow-engine/src/)

| File | LOC | Description |
|------|-----|-------------|
| `src/testing/chicago_tdd.rs` | 1,470 | Chicago TDD framework |
| `src/performance/aot.rs` | 414 | AOT kernel (≤8 ticks requirement) |
| `src/state/manager.rs` | 324 | StateManager implementation |
| `src/patterns/mod.rs` | 320 | PatternRegistry |
| `src/state/store.rs` | 240 | StateStore implementation |
| `src/integration/weaver.rs` | 266 | Weaver integration (static + live validation) |
| `src/data/gateway.rs` | 307 | SPARQL queries, data gateway |
| `src/validation/process_mining.rs` | 206 | Process capability (Cp/Cpk), statistical analysis |
| `src/testing/coverage.rs` | 176 | Test coverage analysis |
| `src/lib.rs` | 142 | Main library (Clippy configuration) |
| `src/parser/mod.rs` | 141 | WorkflowParser |
| `src/executor/engine.rs` | 75 | WorkflowEngine core implementation |
| `src/validation/mod.rs` | 41 | Validation framework |
| **Total Workflow Engine** | **197 files** | **35,286 LOC** |

### Hot Path Files (rust/knhk-hot/)

| File | LOC | Description |
|------|-----|-------------|
| `src/lib.rs` | 33 | RDTSC measurement, hot path operations (≤8 ticks) |

### CLI Files (rust/knhk-cli/)

| File | LOC | Description |
|------|-----|-------------|
| `src/` | 70 files | CLI implementation |
| **Total CLI** | **70 files** | **10,626 LOC** |

### C Library Files (c/src/)

| File | LOC | Description |
|------|-----|-------------|
| `src/` | 25 files | C library implementation (core, kernels, simd, etc.) |
| **Total C Library** | **25 files** | **3,959 LOC** |

### Erlang Files (erlang/knhk_rc/src/)

| File | LOC | Description |
|------|-----|-------------|
| `src/` | 15 files | Erlang implementation |
| **Total Erlang** | **15 files** | **1,037 LOC** |

### Summary Statistics

| Category | Files | LOC | Description |
|----------|-------|-----|-------------|
| **DFLSS Documentation** | 7 | 2,574 | All DFLSS documentation |
| **Rust Workflow Engine** | 197 | 35,286 | Core workflow engine |
| **Rust CLI** | 70 | 10,626 | Command-line interface |
| **Rust DFLSS CLI** | 23 | 2,760 | DFLSS metrics and SPC charts |
| **Rust Hot Path** | 1 | 33 | Performance-critical operations |
| **C Library** | 25 | 3,959 | C implementation |
| **Erlang** | 15 | 1,037 | Erlang implementation |
| **Total Code** | 331 | 53,701 | All implementation files |
| **Grand Total** | 338 | 56,275 | Documentation + Code |

## Direct DFLSS Document → Code File Mapping

### Complete File-to-Code Reference

| DFLSS Document | Section | Code File | Line Range | Type |
|----------------|---------|-----------|------------|------|
| **README.md** | DMEDI Methodology | `rust/knhk-workflow-engine/src/lib.rs` | 1-142 | Library entry point |
| **README.md** | CTQ Requirements | `rust/knhk-workflow-engine/src/validation/mod.rs` | 1-41 | Validation framework |
| **PROJECT_CHARTER.md** | Core Engine | `rust/knhk-workflow-engine/src/executor/engine.rs` | 28-69 | `WorkflowEngine` struct |
| **PROJECT_CHARTER.md** | Workflow Parser | `rust/knhk-workflow-engine/src/parser/mod.rs` | 1-141 | `WorkflowParser` |
| **PROJECT_CHARTER.md** | State Management | `rust/knhk-workflow-engine/src/state/manager.rs` | 1-324 | `StateManager` |
| **PROJECT_CHARTER.md** | State Store | `rust/knhk-workflow-engine/src/state/store.rs` | 1-240 | `StateStore` |
| **PROJECT_CHARTER.md** | Pattern Registry | `rust/knhk-workflow-engine/src/patterns/mod.rs` | 1-320 | `PatternRegistry` |
| **PROJECT_CHARTER.md** | API Layer | `rust/knhk-workflow-engine/src/api/rest/mod.rs` | 1-50 | REST API |
| **PROJECT_CHARTER.md** | CLI | `rust/knhk-cli/src/main.rs` | 1-100 | CLI entry point |
| **SIPOC.md** | Process Steps | `rust/knhk-workflow-engine/src/executor/case.rs` | 1-200 | Case execution |
| **SIPOC.md** | Input Processing | `rust/knhk-workflow-engine/src/parser/mod.rs` | 1-141 | Workflow parsing |
| **SIPOC.md** | Output Generation | `rust/knhk-workflow-engine/src/executor/task.rs` | 1-200 | Task execution |
| **SYNTHETIC_VOC.md** | Weaver Validation | `rust/knhk-workflow-engine/src/integration/weaver.rs` | 15-267 | `WeaverIntegration` |
| **SYNTHETIC_VOC.md** | Performance ≤8 ticks | `rust/knhk-hot/src/lib.rs` | 1-33 | Hot path operations |
| **SYNTHETIC_VOC.md** | RDTSC Measurement | `c/include/knhk/pmu.h` | 12-34 | `knhk_pmu_rdtsc()` |
| **SYNTHETIC_VOC.md** | Cycle Counting | `rust/knhk-hot/src/cycle_counter.rs` | 11-29 | `read_cycles()` |
| **SYNTHETIC_VOC.md** | Tick Budget | `rust/knhk-workflow-engine/src/performance/tick_budget.rs` | 8-56 | `HOT_PATH_TICK_BUDGET` |
| **define/PHASE_SUMMARY.md** | CTQ Definition | `rust/knhk-workflow-engine/src/validation/mod.rs` | 1-41 | Validation framework |
| **define/PHASE_SUMMARY.md** | Baseline Metrics | `rust/knhk-workflow-engine/src/validation/process_mining.rs` | 1-206 | Process capability |
| **define/PHASE_SUMMARY.md** | Problem Statement | `rust/knhk-workflow-engine/src/error/mod.rs` | 1-100 | Error types |
| **measure/PHASE_SUMMARY.md** | Weaver Metrics | `rust/knhk-workflow-engine/src/integration/weaver.rs` | 24-206 | `WeaverIntegration::validate()` |
| **measure/PHASE_SUMMARY.md** | Performance RDTSC | `rust/knhk-hot/src/cycle_counter.rs` | 11-62 | Cycle counting |
| **measure/PHASE_SUMMARY.md** | Performance Benchmarks | `rust/knhk-hot/benches/cycle_bench.rs` | 1-100 | Benchmark tests |
| **measure/PHASE_SUMMARY.md** | C Benchmarks | `c/tools/knhk_bench.c` | 1-220 | C performance tests |
| **measure/PHASE_SUMMARY.md** | Process Capability | `rust/knhk-workflow-engine/src/validation/process_mining.rs` | 1-206 | Cp/Cpk calculation |
| **measure/PHASE_SUMMARY.md** | Test Coverage | `rust/knhk-workflow-engine/src/testing/coverage.rs` | 1-176 | Coverage analysis |
| **measure/PHASE_SUMMARY.md** | Defect Tracking | `.git/hooks/pre-commit` | 1-50 | Pre-commit validation |
| **analyze/PHASE_SUMMARY.md** | Root Cause Analysis | `rust/knhk-workflow-engine/src/testing/chicago_tdd.rs` | 1-1471 | Chicago TDD framework |
| **analyze/PHASE_SUMMARY.md** | Chicago TDD Tests | `rust/knhk-workflow-engine/tests/chicago_tdd_tools_integration.rs` | 1-100 | Integration tests |
| **analyze/PHASE_SUMMARY.md** | Error Analysis | `rust/knhk-workflow-engine/src/error/mod.rs` | 1-100 | Error handling |
| **improve/PHASE_SUMMARY.md** | Hot Path Optimization | `rust/knhk-hot/src/lib.rs` | 1-33 | Hot path engine |
| **improve/PHASE_SUMMARY.md** | CONSTRUCT8 | `rust/knhk-warm/src/construct8.rs` | 1-173 | CONSTRUCT8 implementation |
| **improve/PHASE_SUMMARY.md** | Tick Budget | `rust/knhk-etl/src/reflex.rs` | 22-23 | `tick_budget: u32 = 8` |
| **improve/PHASE_SUMMARY.md** | Performance AOT | `rust/knhk-workflow-engine/src/performance/aot.rs` | 1-414 | AOT kernel |
| **control/PHASE_SUMMARY.md** | CI/CD Gates | `.github/workflows/ci.yml` | 1-50 | CI pipeline |
| **control/PHASE_SUMMARY.md** | Pre-commit Hooks | `.git/hooks/pre-commit` | 1-50 | Pre-commit validation |
| **control/PHASE_SUMMARY.md** | Pre-push Hooks | `.git/hooks/pre-push` | 1-50 | Pre-push validation |
| **control/PHASE_SUMMARY.md** | SPC Charts | `rust/knhk-dflss/src/commands/charts.rs` | 1-420 | X-bar/R, p, c charts |
| **control/PHASE_SUMMARY.md** | SPC Scripts | `rust/knhk-dflss/src/commands/` | All | Rust CLI implementation |
| **control/PHASE_SUMMARY.md** | Metrics Collection | `rust/knhk-dflss/src/commands/metrics.rs` | 1-343 | Quality, performance, Weaver |
| **control/PHASE_SUMMARY.md** | Process Capability | `rust/knhk-dflss/src/commands/capability.rs` | 1-265 | Cp, Cpk, Sigma calculation |
| **control/PHASE_SUMMARY.md** | CTQ Validation | `rust/knhk-dflss/src/commands/validation.rs` | 1-332 | Weaver, performance, quality, DoD |
| **measure/PHASE_SUMMARY.md** | Process Capability | `rust/knhk-dflss/src/internal/capability.rs` | 1-150 | ProcessCapability implementation |
| **measure/PHASE_SUMMARY.md** | Metrics Collection | `rust/knhk-dflss/src/internal/quality.rs` | 1-200 | QualityCollector implementation |

## Revision History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2025-11-09 | Initial code mapping creation |
| 1.1 | 2025-11-09 | Updated with verified file paths and code references |
| 1.2 | 2025-01-27 | Added direct file-to-LOC mapping with complete statistics (315 files, 53,515 LOC) |
| 1.3 | 2025-01-27 | Added comprehensive DFLSS document → code file mapping with line references |
| 1.4 | 2025-01-27 | Added knhk-dflss Rust CLI crate (23 files, 2,760 LOC) replacing Python SPC scripts |

---

**Code Mapping Complete** ✅  
**All DFLSS Documentation Mapped to Code Files**

