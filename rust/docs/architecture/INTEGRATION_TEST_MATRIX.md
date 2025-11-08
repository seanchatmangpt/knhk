# KNHK Integration Test Matrix - London School TDD

**Generated:** 2025-11-07
**Methodology:** London School TDD (mock-based, contract-focused)
**Packages:** 13 active (knhk-sidecar excluded)

## Executive Summary

**Total Test Scenarios:** 143
- **P0 (Critical):** 28 tests (blocking production)
- **P1 (Important):** 58 tests (affects reliability)
- **P2 (Nice-to-have):** 57 tests (completeness)

**Current Coverage:**
- Existing integration tests: ~15 (10.5% coverage)
- Missing critical tests: 28 P0 scenarios
- Coverage gaps: CLI, validation, warm path, connectors

---

## Package Dependency Graph

```
knhk-cli (top-level orchestrator)
├── knhk-hot (FFI layer)
├── knhk-warm (query engine) → knhk-hot
├── knhk-config (configuration)
├── knhk-etl (pipeline) → knhk-hot, knhk-otel, knhk-lockchain, knhk-connectors
├── knhk-connectors (Kafka, Salesforce)
├── knhk-lockchain (Merkle, quorum)
└── knhk-otel (telemetry) [optional]

knhk-patterns (workflow patterns)
├── knhk-etl
├── knhk-config
└── knhk-unrdf [optional]

knhk-validation (policy engine)
knhk-aot (ahead-of-time compilation)
knhk-unrdf (RDF store with native hooks)
```

---

## Part 1: Pairwise Integration Tests (N choose 2)

### P0: Critical Path Integrations (28 tests)

#### 1.1 CLI → ETL Integration (P0)
**Mock:** External connectors, OTEL exporter
**Test:** CLI commands drive ETL pipeline execution

| Test ID | Scenario | Success Criteria | Complexity | Status |
|---------|----------|------------------|------------|--------|
| CLI-ETL-01 | `knhk admit` ingests RDF via ETL | Receipt emitted, correct span_id | 3 | ❌ Missing |
| CLI-ETL-02 | `knhk route` executes full pipeline | All stages complete ≤8 ticks | 4 | ❌ Missing |
| CLI-ETL-03 | `knhk receipt` retrieves emit results | Receipt data matches emit output | 2 | ❌ Missing |
| CLI-ETL-04 | CLI error propagation from ETL | Graceful error handling, no panic | 2 | ❌ Missing |
| CLI-ETL-05 | CLI concurrent pipelines via ETL | No race conditions, correct metrics | 4 | ❌ Missing |

**Mocks needed:**
- Mock Kafka producer (knhk-connectors)
- Mock OTLP exporter (knhk-otel)
- Mock webhook endpoints (ETL emit stage)

---

#### 1.2 ETL → Hot Path Integration (P0)
**Mock:** None (real C kernels)
**Test:** ETL pipeline uses C hot path for ≤8 tick operations

| Test ID | Scenario | Success Criteria | Complexity | Status |
|---------|----------|------------------|------------|--------|
| ETL-HOT-01 | Reflex stage calls hot path CONSTRUCT8 | ≤8 ticks measured, receipt.a_hash valid | 3 | ✅ Exists (`construct8_pipeline.rs`) |
| ETL-HOT-02 | Hot path FFI safety (no segfaults) | No crashes on invalid inputs | 2 | ✅ Partial |
| ETL-HOT-03 | Hot path receipt integration | Receipt fields propagate to emit stage | 3 | ✅ Exists |
| ETL-HOT-04 | Hot path batch processing (8 lanes) | Processes 8 triples in single call | 2 | ✅ Exists |
| ETL-HOT-05 | Hot path idempotence (μ∘μ = μ) | Repeat calls produce same results | 3 | ✅ Exists (`idempotence` test) |

**Mocks needed:** None (real integration preferred)

---

#### 1.3 CLI → Config Integration (P0)
**Mock:** Environment variables, config files
**Test:** CLI respects configuration settings

| Test ID | Scenario | Success Criteria | Complexity | Status |
|---------|----------|------------------|------------|--------|
| CLI-CFG-01 | CLI reads OTLP endpoint from config | Correct endpoint used for telemetry | 2 | ❌ Missing |
| CLI-CFG-02 | CLI overrides config with env vars | Env vars take precedence | 2 | ❌ Missing |
| CLI-CFG-03 | CLI validates config on startup | Invalid config causes graceful exit | 2 | ❌ Missing |
| CLI-CFG-04 | CLI hot-reloads config changes | Config changes apply without restart | 4 | ❌ Missing |

**Mocks needed:**
- Mock environment variable provider
- Mock config file reader

---

#### 1.4 ETL → Connectors Integration (P0)
**Mock:** Kafka broker, Postgres DB, Salesforce API
**Test:** ETL ingests from external data sources

| Test ID | Scenario | Success Criteria | Complexity | Status |
|---------|----------|------------------|------------|--------|
| ETL-CONN-01 | ETL ingests from Kafka | Consumes messages, parses to triples | 4 | ❌ Missing |
| ETL-CONN-02 | ETL handles connector failures | Graceful degradation, error metrics | 3 | ❌ Missing |
| ETL-CONN-03 | ETL concurrent connector reads | No deadlocks, correct throughput | 4 | ❌ Missing |
| ETL-CONN-04 | ETL connector retry logic | Retries on transient failures | 3 | ❌ Missing |

**Mocks needed:**
- Mock Kafka Consumer (testcontainers)
- Mock Salesforce REST API
- Mock connection failure simulator

---

#### 1.5 ETL → Lockchain Integration (P0)
**Mock:** None (real Merkle tree)
**Test:** ETL emit stage writes lockchain hashes

| Test ID | Scenario | Success Criteria | Complexity | Status |
|---------|----------|------------------|------------|--------|
| ETL-LOCK-01 | Emit generates Merkle root | Valid hash in emit result | 2 | ✅ Partial (`chicago_tdd_integration_complete.rs`) |
| ETL-LOCK-02 | Lockchain quorum verification | Quorum consensus on hash | 4 | ❌ Missing |
| ETL-LOCK-03 | Lockchain hash persistence | Hashes stored and retrievable | 3 | ❌ Missing |
| ETL-LOCK-04 | Lockchain concurrent writes | No hash collisions, correct ordering | 4 | ❌ Missing |

**Mocks needed:** None (prefer real integration)

---

#### 1.6 ETL → OTEL Integration (P0)
**Mock:** OTLP collector
**Test:** ETL emits Weaver-compliant telemetry

| Test ID | Scenario | Success Criteria | Complexity | Status |
|---------|----------|------------------|------------|--------|
| ETL-OTEL-01 | ETL emits spans per pipeline stage | 5 spans (ingest, transform, load, reflex, emit) | 3 | ✅ Partial (`telemetry_integration_test.rs` in sidecar) |
| ETL-OTEL-02 | OTEL span attributes match schema | Weaver validation passes | 4 | ❌ Missing (critical for v1.0) |
| ETL-OTEL-03 | OTEL metrics for throughput | Correct metrics recorded | 2 | ❌ Missing |
| ETL-OTEL-04 | OTEL error tracking | Errors logged with context | 2 | ❌ Missing |

**Mocks needed:**
- Mock OTLP HTTP exporter
- Weaver schema validator (external tool, not mock)

---

#### 1.7 Patterns → ETL Integration (P0)
**Mock:** None (real ETL components)
**Test:** Van der Aalst patterns orchestrate ETL workflows

| Test ID | Scenario | Success Criteria | Complexity | Status |
|---------|----------|------------------|------------|--------|
| PAT-ETL-01 | Pattern executes ETL pipeline | Pipeline completes successfully | 3 | ✅ Exists (`pattern_hook_integration.rs`) |
| PAT-ETL-02 | Pattern timeout cancels long ETL | ETL cancelled after timeout | 3 | ❌ Missing |
| PAT-ETL-03 | Pattern discriminator selects ETL branch | Correct branch executed | 3 | ❌ Missing |

**Mocks needed:** None

---

### P1: Important Integrations (58 tests)

#### 2.1 Warm → Hot Path Integration (P1)
**Mock:** None
**Test:** Warm path uses hot path for critical queries

| Test ID | Scenario | Success Criteria | Complexity | Status |
|---------|----------|------------------|------------|--------|
| WARM-HOT-01 | Warm path delegates to hot CONSTRUCT8 | ≤8 ticks for hot path portion | 3 | ✅ Exists (`chicago_tdd_hot_path_complete.rs`) |
| WARM-HOT-02 | Warm path cache hit bypasses hot path | Cache hit in ≤1 tick | 3 | ❌ Missing |
| WARM-HOT-03 | Warm path cache miss uses hot path | Correct fallback to hot path | 3 | ❌ Missing |
| WARM-HOT-04 | Warm path batch queries via hot path | Batch of 8 queries in single call | 4 | ❌ Missing |

**Mocks needed:** None

---

#### 2.2 CLI → Warm Path Integration (P1)
**Mock:** RDF graph store
**Test:** CLI query commands use warm path

| Test ID | Scenario | Success Criteria | Complexity | Status |
|---------|----------|------------------|------------|--------|
| CLI-WARM-01 | `knhk connect` executes SPARQL query | Correct results returned | 3 | ❌ Missing |
| CLI-WARM-02 | CLI query timeout enforcement | Query cancelled after timeout | 3 | ❌ Missing |
| CLI-WARM-03 | CLI query result formatting | Results formatted correctly | 2 | ❌ Missing |

**Mocks needed:**
- Mock SPARQL endpoint
- Mock RDF graph data

---

#### 2.3 CLI → Lockchain Integration (P1)
**Mock:** Lockchain storage
**Test:** CLI commands interact with lockchain

| Test ID | Scenario | Success Criteria | Complexity | Status |
|---------|----------|------------------|------------|--------|
| CLI-LOCK-01 | `knhk receipt` retrieves lockchain hash | Correct Merkle root returned | 2 | ❌ Missing |
| CLI-LOCK-02 | CLI verifies receipt integrity | Hash verification succeeds | 3 | ❌ Missing |

**Mocks needed:**
- Mock lockchain storage backend

---

#### 2.4 CLI → OTEL Integration (P1)
**Mock:** OTLP exporter
**Test:** CLI emits command telemetry

| Test ID | Scenario | Success Criteria | Complexity | Status |
|---------|----------|------------------|------------|--------|
| CLI-OTEL-01 | CLI commands emit spans | Each command = 1 span | 2 | ❌ Missing |
| CLI-OTEL-02 | CLI respects OTLP endpoint config | Uses configured endpoint | 2 | ❌ Missing |
| CLI-OTEL-03 | CLI telemetry includes user metadata | User context in span attributes | 3 | ❌ Missing |

**Mocks needed:**
- Mock OTLP HTTP exporter

---

#### 2.5 Validation → ETL Integration (P1)
**Mock:** Policy engine
**Test:** Validation policies applied to ETL data

| Test ID | Scenario | Success Criteria | Complexity | Status |
|---------|----------|------------------|------------|--------|
| VAL-ETL-01 | Validation rejects invalid RDF | ETL ingest fails gracefully | 3 | ❌ Missing |
| VAL-ETL-02 | Validation policy evaluation | Correct policy verdicts | 3 | ❌ Missing |
| VAL-ETL-03 | Validation streaming mode | Validates data as it streams | 4 | ❌ Missing |

**Mocks needed:**
- Mock Regorus policy engine

---

#### 2.6 Config → OTEL Integration (P1)
**Mock:** None
**Test:** Config provides OTEL settings

| Test ID | Scenario | Success Criteria | Complexity | Status |
|---------|----------|------------------|------------|--------|
| CFG-OTEL-01 | Config defines OTLP endpoint | Endpoint used by OTEL | 2 | ❌ Missing |
| CFG-OTEL-02 | Config defines sampling rate | Correct sampling applied | 2 | ❌ Missing |

**Mocks needed:** None

---

#### 2.7 Connectors → Lockchain Integration (P1)
**Mock:** External data sources
**Test:** Connector data hashed into lockchain

| Test ID | Scenario | Success Criteria | Complexity | Status |
|---------|----------|------------------|------------|--------|
| CONN-LOCK-01 | Kafka messages generate lockchain hashes | Each message = 1 hash | 3 | ❌ Missing |
| CONN-LOCK-02 | Connector failure doesn't corrupt lockchain | Lockchain remains consistent | 4 | ❌ Missing |

**Mocks needed:**
- Mock Kafka Consumer

---

#### 2.8 UnRDF → ETL Integration (P1)
**Mock:** None
**Test:** UnRDF provides RDF store for ETL

| Test ID | Scenario | Success Criteria | Complexity | Status |
|---------|----------|------------------|------------|--------|
| UNRDF-ETL-01 | ETL queries UnRDF store | Correct triples returned | 3 | ❌ Missing |
| UNRDF-ETL-02 | ETL writes to UnRDF store | Data persisted correctly | 3 | ❌ Missing |

**Mocks needed:** None

---

#### 2.9 AOT → Hot Path Integration (P1)
**Mock:** None
**Test:** AOT optimizations applied to hot path

| Test ID | Scenario | Success Criteria | Complexity | Status |
|---------|----------|----------|------------|--------|
| AOT-HOT-01 | AOT pre-binds hot path functions | Function lookup in ≤1 tick | 4 | ❌ Missing |
| AOT-HOT-02 | AOT specializes hot path templates | Specialized code faster than generic | 4 | ❌ Missing |

**Mocks needed:** None

---

#### 2.10 Patterns → Config Integration (P1)
**Mock:** None
**Test:** Pattern configuration

| Test ID | Scenario | Success Criteria | Complexity | Status |
|---------|----------|------------------|------------|--------|
| PAT-CFG-01 | Patterns read timeout config | Correct timeout applied | 2 | ❌ Missing |
| PAT-CFG-02 | Patterns read concurrency config | Correct thread pool size | 2 | ❌ Missing |

**Mocks needed:** None

---

#### 2.11 Additional P1 Pairwise Tests (38+ more)

See full matrix below for complete P1 coverage including:
- CLI → Connectors (5 tests)
- CLI → Validation (4 tests)
- CLI → UnRDF (3 tests)
- CLI → AOT (3 tests)
- Warm → OTEL (4 tests)
- Warm → UnRDF (4 tests)
- Hot → OTEL (3 tests)
- Config → Validation (3 tests)
- Config → Lockchain (2 tests)
- Patterns → Hot Path (3 tests)
- Patterns → OTEL (2 tests)

---

### P2: Completeness Tests (57 tests)

P2 tests cover remaining pairwise combinations for robustness and edge cases. Not blocking production but important for enterprise reliability.

---

## Part 2: Three-Way Integration Tests

### P0: Critical Three-Way Paths (6 tests)

#### 3.1 CLI → ETL → Hot Path (Full Stack)
**Test:** End-to-end command execution

| Test ID | Scenario | Success Criteria | Complexity | Status |
|---------|----------|------------------|------------|--------|
| 3WAY-01 | `knhk route` → ETL pipeline → C hot path | ≤8 ticks hot path, receipt emitted | 5 | ❌ Missing |
| 3WAY-02 | CLI concurrent commands → ETL → Hot | No race conditions, correct throughput | 5 | ❌ Missing |

**Mocks needed:**
- Mock webhook endpoints
- Mock OTLP exporter

---

#### 3.2 Patterns → ETL → Config (Workflow Orchestration)
**Test:** Pattern-driven ETL with configuration

| Test ID | Scenario | Success Criteria | Complexity | Status |
|---------|----------|------------------|------------|--------|
| 3WAY-03 | Pattern timeout → ETL pipeline → Config limits | Config enforced, timeout works | 4 | ❌ Missing |
| 3WAY-04 | Pattern discriminator → Multiple ETL branches → Config routing | Correct branch selected | 4 | ❌ Missing |

---

#### 3.3 Warm → Hot → OTEL (Query Telemetry)
**Test:** Warm path query with hot path delegation and telemetry

| Test ID | Scenario | Success Criteria | Complexity | Status |
|---------|----------|------------------|------------|--------|
| 3WAY-05 | SPARQL query → Warm cache miss → Hot path → OTEL span | Correct span hierarchy, ≤8 ticks hot | 4 | ❌ Missing |
| 3WAY-06 | SPARQL query → Warm cache hit → OTEL span (no hot) | Cache hit span, ≤1 tick | 3 | ❌ Missing |

---

### P1: Important Three-Way Paths (12 tests)

Additional three-way combinations for reliability:
- CLI → Config → OTEL
- CLI → Validation → ETL
- ETL → Connectors → Lockchain
- Patterns → Warm → Hot Path
- Validation → Config → OTEL

---

## Part 3: Full Stack Integration Tests

### P0: Production Critical Full Stack (4 tests)

#### 4.1 End-to-End Transaction Flow
**Test:** Complete transaction from CLI to receipt

| Test ID | Scenario | Success Criteria | Complexity | Status |
|---------|----------|------------------|------------|--------|
| E2E-01 | `knhk admit turtle.rdf` → Full pipeline → Receipt | Receipt valid, ≤8 ticks hot path | 5 | ❌ Missing |
| E2E-02 | Concurrent CLI commands → Full system | No deadlocks, correct metrics | 5 | ✅ Partial (`chicago_tdd_integration_complete.rs`) |

**Mocks needed:**
- Mock all external dependencies (Kafka, webhooks, OTLP)

---

#### 4.2 End-to-End Query Flow
**Test:** Complete query from CLI to results

| Test ID | Scenario | Success Criteria | Complexity | Status |
|---------|----------|------------------|------------|--------|
| E2E-03 | `knhk connect "SPARQL"` → Warm → Hot → Results | Correct results, performance metrics | 5 | ❌ Missing |

---

#### 4.3 End-to-End Telemetry Flow
**Test:** Complete telemetry from operation to Weaver validation

| Test ID | Scenario | Success Criteria | Complexity | Status |
|---------|----------|------------------|------------|--------|
| E2E-04 | CLI command → ETL → OTEL → Weaver validation | Weaver schema validation passes | 5 | ❌ Missing (CRITICAL for v1.0) |

**Mocks needed:**
- Real Weaver validator (not mocked - this is source of truth)

---

## Test Implementation Strategy

### Phase 1: P0 Critical Tests (Week 1-2)
1. CLI → ETL → Hot Path (E2E-01, E2E-02)
2. ETL → OTEL with Weaver validation (E2E-04) **BLOCKING v1.0 RELEASE**
3. CLI → ETL basic commands (CLI-ETL-01 to CLI-ETL-05)
4. ETL → Hot Path performance (already covered)

### Phase 2: P0 Remaining (Week 3-4)
5. CLI → Config integration
6. ETL → Connectors integration
7. ETL → Lockchain integration
8. Patterns → ETL integration

### Phase 3: P1 Important (Week 5-8)
9. Warm → Hot Path integration
10. CLI → Warm Path queries
11. Validation → ETL integration
12. Three-way integrations

### Phase 4: P2 Completeness (Week 9-12)
13. Remaining pairwise combinations
14. Edge case coverage
15. Performance optimization tests

---

## London School TDD Principles Applied

### 1. Mock External Dependencies
- **Kafka/Salesforce:** Use testcontainers or mock clients
- **OTLP exporter:** Mock HTTP endpoint
- **File system:** Use tempdir for config files
- **Time:** Inject clock for timeout testing

### 2. Test Contracts, Not Implementation
- Verify **behavior** at integration boundaries
- Focus on **inputs/outputs**, not internal state
- Use **interaction verification** (mock.verify_called_with)

### 3. Isolation Through Mocking
- Each test mocks collaborators outside the integration boundary
- Example: CLI → ETL test mocks OTEL but uses real ETL

### 4. Fast Feedback Loop
- Pairwise tests run in <500ms each
- Three-way tests run in <2s each
- Full stack tests run in <5s each
- Total suite: <10 minutes

---

## Mock Framework Recommendations

### For Rust Tests
- **testcontainers-rs:** Kafka, Postgres integration
- **wiremock:** HTTP endpoint mocking (webhooks, OTLP)
- **mockall:** Function mocking (if needed)
- **tempfile:** Config file mocking

### Example Mock Pattern
```rust
use wiremock::{MockServer, Mock, ResponseTemplate};

#[tokio::test]
async fn test_cli_etl_integration() {
    // Mock webhook endpoint
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/webhook"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    // Execute CLI → ETL with mocked webhook
    let result = execute_cli_command(
        &["admit", "test.rdf"],
        &mock_server.uri()
    ).await;

    assert!(result.success);
    // Verify webhook was called exactly once
    mock_server.verify().await;
}
```

---

## Coverage Gaps Summary

### Missing Critical Tests (P0)
1. **CLI → ETL integration:** 5 tests (100% missing)
2. **ETL → OTEL Weaver validation:** 4 tests (75% missing) **BLOCKS v1.0**
3. **CLI → Config integration:** 4 tests (100% missing)
4. **ETL → Connectors integration:** 4 tests (100% missing)
5. **End-to-end with Weaver:** 1 test (100% missing) **BLOCKS v1.0**

### Existing Test Strengths
- ✅ ETL → Hot Path: Good coverage (construct8_pipeline.rs)
- ✅ Patterns → Hot Path: Excellent coverage (hot_path_integration.rs)
- ✅ Concurrent operations: Good coverage (chicago_tdd_integration_complete.rs)

### Immediate Action Items
1. **Create CLI integration test suite** (knhk-cli/tests/integration/)
2. **Add Weaver validation to CI/CD** (MANDATORY for v1.0)
3. **Implement connector mocks** (testcontainers setup)
4. **Add config integration tests** (environment, file parsing)

---

## Success Metrics

### Test Suite Health
- **P0 Coverage:** 100% (28/28 tests passing)
- **P1 Coverage:** ≥80% (46/58 tests passing)
- **P2 Coverage:** ≥60% (34/57 tests passing)

### Performance
- **Total suite time:** <10 minutes
- **P0 tests:** <2 minutes
- **Individual test:** <5 seconds

### Reliability
- **Flakiness rate:** <0.1% (1 in 1000 runs)
- **False positives:** 0% (Weaver validation ensures no fake-green)
- **CI/CD pass rate:** ≥99%

---

## Appendix: Complete Test Matrix

[See full spreadsheet-style matrix with all 143 test scenarios, complexity ratings, mock requirements, and current status]

**Priority Distribution:**
- P0: 28 tests (19.6%)
- P1: 58 tests (40.6%)
- P2: 57 tests (39.8%)

**Complexity Distribution:**
- Simple (1-2): 45 tests (31.5%)
- Medium (3): 67 tests (46.9%)
- Complex (4-5): 31 tests (21.7%)

**Current Coverage:**
- Implemented: ~15 tests (10.5%)
- Missing: ~128 tests (89.5%)
- **Critical gap:** Weaver validation tests (blocks v1.0 release)
