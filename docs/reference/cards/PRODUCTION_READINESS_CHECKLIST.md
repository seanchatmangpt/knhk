# Production Readiness Checklist

**Purpose**: One-page verification checklist for KNHK production deployments
**Estimated Time**: 1.5-2 hours
**Target**: Zero failures before deployment

---

## 1. Build Quality (Baseline)

**All items must pass before proceeding:**

- [ ] `cd /home/user/knhk/rust && cargo build --workspace --release` succeeds
- [ ] Zero compilation warnings in output
- [ ] `cd /home/user/knhk/c && make` succeeds (C hot path layer)
- [ ] C library links successfully with Rust bindings
- [ ] `cargo clippy --workspace -- -D warnings` shows zero issues
- [ ] `cargo fmt --all --check` shows no formatting issues
- [ ] All dependency versions locked in Cargo.lock

**Pass Criteria**: All 7 items checked ✅

---

## 2. Code Quality Standards

**Verify production-ready code:**

- [ ] No `.unwrap()` or `.expect()` in production code paths
- [ ] All public functions use `Result<T, E>` error handling
- [ ] All traits remain `dyn` compatible (no async trait methods)
- [ ] No `println!` in production code (use `tracing::info!` instead)
- [ ] No fake `Ok(())` returns from stub implementations
- [ ] All `TODO`, `FIXME`, `HACK` comments addressed or documented
- [ ] Error messages are descriptive and actionable
- [ ] All `unsafe` blocks have safety documentation

**Pass Criteria**: All 8 items checked ✅

---

## 3. Telemetry & Observability (MANDATORY)

**Weaver validation is the source of truth:**

- [ ] **`weaver registry check -r /home/user/knhk/registry/` passes** (schema valid)
- [ ] **`weaver registry live-check --registry /home/user/knhk/registry/` passes** (runtime telemetry valid)
- [ ] All spans/metrics/logs defined in OTEL schema
- [ ] Strategic instrumentation pyramid followed (few spans, more metrics, many logs)
- [ ] OTLP exporter configured for production collector
- [ ] Telemetry overhead measured (≤5% CPU, ≤10MB memory)
- [ ] Sampling configured appropriately (e.g., 10% for traces)
- [ ] No telemetry in hot path critical sections (≤8 ticks)

**Pass Criteria**: First 2 items are MANDATORY ✅, all 8 items for full readiness

---

## 4. Performance Compliance

**Hot path must meet Chatman Constant (≤8 ticks):**

- [ ] `make test-performance-v04` passes (verifies ≤8 ticks)
- [ ] ASK operations: ≤1.5 ns measured
- [ ] COUNT operations: ≤1.5 ns measured
- [ ] COMPARE operations: ≤1.5 ns measured
- [ ] VALIDATE operations: ≤2.0 ns measured
- [ ] CONSTRUCT8 operations: ≤500 µs (warm path)
- [ ] No heap allocations in hot path
- [ ] Branchless C engine verified (zero branch mispredicts)

**Pass Criteria**: All 8 items checked ✅

---

## 5. Testing Coverage (Chicago TDD)

**Comprehensive test suite with real collaborators:**

- [ ] `cargo test --workspace` passes 100%
- [ ] `make test-chicago-v04` passes (C hot path tests)
- [ ] `make test-integration-v2` passes (integration tests)
- [ ] `make test-enterprise` passes (enterprise use cases)
- [ ] Test coverage ≥90% for core modules
- [ ] All tests follow AAA pattern (Arrange, Act, Assert)
- [ ] Tests use real collaborators (no mocks for core logic)
- [ ] Edge cases tested (empty inputs, large datasets, errors)

**Pass Criteria**: All 8 items checked ✅

---

## 6. Functional Validation (NOT Just `--help`)

**Actually execute commands with real arguments:**

- [ ] **Hot path queries execute successfully** (not just `knhk --help`)
- [ ] **Warm path CONSTRUCT8 produces output** (run actual query)
- [ ] **Workflow engine executes sample workflow** (end-to-end test)
- [ ] **OTLP exporter sends telemetry** (check collector logs)
- [ ] **CLI commands work with real arguments** (not just help text)
- [ ] **Integration tests run against real components** (not mocks)
- [ ] **Performance benchmarks produce valid results** (measure actual time)

**Pass Criteria**: All 7 items checked ✅ - Must actually RUN, not just show help

---

## 7. Security & Configuration

**Production security hardening:**

- [ ] No secrets in code, environment variables, or configuration files
- [ ] TLS/mTLS configured for network connections
- [ ] Authentication/authorization implemented
- [ ] Input validation on all external inputs
- [ ] Rate limiting configured
- [ ] Resource limits set (memory, CPU, connections)
- [ ] Audit logging enabled
- [ ] Security scanning completed (e.g., `cargo audit`)

**Pass Criteria**: All 8 items checked ✅

---

## 8. Deployment Readiness

**Infrastructure and deployment preparation:**

- [ ] Docker image builds successfully
- [ ] Kubernetes manifests validated
- [ ] Health check endpoints implemented (`/health`, `/ready`)
- [ ] Graceful shutdown implemented (SIGTERM handling)
- [ ] Configuration externalized (12-factor app)
- [ ] Documentation updated (deployment guide, runbooks)
- [ ] Monitoring dashboards configured
- [ ] Alerting rules defined

**Pass Criteria**: All 8 items checked ✅

---

## Quick Validation Script

```bash
#!/bin/bash
# Production readiness quick check

set -e

echo "=== Build Quality ==="
cd /home/user/knhk/rust && cargo build --workspace --release
cd /home/user/knhk/c && make
cd /home/user/knhk/rust && cargo clippy --workspace -- -D warnings

echo "=== Weaver Validation (MANDATORY) ==="
weaver registry check -r /home/user/knhk/registry/
weaver registry live-check --registry /home/user/knhk/registry/

echo "=== Performance Compliance ==="
cd /home/user/knhk && make test-performance-v04

echo "=== Testing Coverage ==="
cd /home/user/knhk/rust && cargo test --workspace
cd /home/user/knhk && make test-chicago-v04
cd /home/user/knhk && make test-integration-v2

echo "✅ All checks passed!"
```

---

## Critical Hierarchy (Remember!)

1. **Weaver Validation** = Source of truth (proves feature works)
2. **Compilation + Clippy** = Code quality baseline (proves code is valid)
3. **Traditional Tests** = Supporting evidence (can have false positives)

**If Weaver validation fails, the feature DOES NOT WORK, regardless of test results.**

---

## Final Sign-Off

- [ ] **ALL 8 sections completed** (56 total checks)
- [ ] **Weaver validation passes** (MANDATORY)
- [ ] **Performance benchmarks meet targets** (≤8 ticks)
- [ ] **All tests pass** (no failures, no warnings)
- [ ] **Security review completed**
- [ ] **Deployment guide updated**

**Deployment Approved By**: ________________
**Date**: ________________
**Version**: ________________

---

**See Also**:
- [Performance Guide](/home/user/knhk/docs/PERFORMANCE.md)
- [Testing Guide](/home/user/knhk/docs/TESTING.md)
- [Production Guide](/home/user/knhk/docs/PRODUCTION.md)
