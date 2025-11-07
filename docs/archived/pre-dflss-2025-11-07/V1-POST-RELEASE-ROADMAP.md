# KNHK Post-V1.0 Roadmap & Technical Debt Backlog

**Document Version**: 1.0
**Created**: 2025-01-06
**Owner**: Strategic Planning Agent (#11)
**Status**: Living Document

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Technical Debt Inventory](#technical-debt-inventory)
3. [V1.1 Release Plan](#v11-release-plan)
4. [V1.2 Release Plan](#v12-release-plan)
5. [V2.0 Vision](#v20-vision)
6. [Continuous Improvement Strategy](#continuous-improvement-strategy)
7. [Appendices](#appendices)

---

## 1. Executive Summary

### 1.1 Purpose

This document provides a strategic roadmap for KNHK development post-v1.0 release, balancing new features, technical debt remediation, and continuous improvement.

### 1.2 Principles

- **Weaver Validation First**: All changes must pass Weaver schema validation
- **Performance Budget Preservation**: Hot path ≤8 ticks, warm path ≤500ms
- **No Breaking Changes (Minor Versions)**: Maintain API stability in v1.x
- **Evidence-Based Planning**: Technical debt prioritized by actual pain points
- **80/20 Focus**: Prioritize high-impact, low-effort improvements

### 1.3 Release Cadence

| Release | Timeline | Focus |
|---------|----------|-------|
| **v1.0** | ✅ Complete | Core 8-beat functionality, ETL pipeline, connectors |
| **v1.1** | +2-4 weeks | Bug fixes, polish, documentation improvements |
| **v1.2** | +2-3 months | Enhanced observability, additional templates, minor features |
| **v1.3** | +4-6 months | Community-driven features, performance optimizations |
| **v2.0** | +6-12 months | Major architectural improvements, breaking changes |

---

## 2. Technical Debt Inventory

### 2.1 Active Technical Debt (From Codebase Analysis)

#### 🔴 HIGH PRIORITY (Address in v1.1)

**TD-001: Policy Engine Rego Implementation**
- **Location**: `rust/knhk-validation/src/policy_engine.rs:163, 211`
- **Issue**: Rego policy loading and evaluation stubbed out
- **Impact**: Policy engine limited to built-in policies only
- **Code**:
  ```rust
  // TODO: Implement Rego policy loading
  // TODO: Evaluate Rego policy
  ```
- **Remediation**:
  - Integrate `rego-rs` crate (when available) or use OPA REST API
  - Implement policy file loading and caching
  - Add Rego policy evaluation integration tests
- **Effort**: 2-3 days
- **Risk**: Medium (external dependency on rego-rs stability)

**TD-002: Sidecar Service Proto Implementation**
- **Location**: `rust/knhk-sidecar/src/server.rs:10, 107`, `rust/knhk-sidecar/src/lib.rs:14`
- **Issue**: Service implementation temporarily disabled due to proto mismatches
- **Impact**: gRPC service methods not fully functional
- **Code**:
  ```rust
  // TODO: Re-enable when service.rs is fixed
  // TODO: Fix service.rs - has proto schema mismatches and missing dependencies
  ```
- **Remediation**:
  - Align proto definitions with tonic-build generated code
  - Fix missing dependencies in Cargo.toml
  - Re-enable service methods
  - Add integration tests for gRPC endpoints
- **Effort**: 1-2 days
- **Risk**: Low (well-understood proto/tonic integration)

#### 🟡 MEDIUM PRIORITY (Address in v1.2)

**TD-003: Lockchain Git Commit Integration**
- **Location**: `rust/knhk-lockchain/src/lib.rs:133-134`
- **Issue**: Git commit requires external git2 crate, currently manual
- **Impact**: Provenance commits require manual git operations
- **Code**:
  ```rust
  // Note: Actual Git commit would require git2 crate
  // For now, we write files that can be committed manually or via external tool
  ```
- **Remediation**:
  - Add `git2` crate as optional feature
  - Implement automatic commit on receipt storage
  - Add integration tests for Git operations
- **Effort**: 2 days
- **Risk**: Low (git2 is mature and well-documented)

**TD-004: Oxigraph Deprecated APIs**
- **Location**: Multiple files using `Query::parse()` and `store.query()`
- **Issue**: Using deprecated oxigraph APIs (documented in `docs/deprecated-apis.md`)
- **Impact**: Risk of breaking changes in future oxigraph releases
- **Remediation**:
  - Monitor oxigraph release notes for migration path
  - Create migration guide when new APIs available
  - Update all query execution paths
- **Effort**: 1-2 days (once migration path is available)
- **Risk**: Medium (blocked by upstream oxigraph team)

#### 🟢 LOW PRIORITY (Address in v1.3+)

**TD-005: Connector Lifecycle Test Coverage**
- **Location**: `rust/knhk-connectors/src/kafka.rs`, `rust/knhk-connectors/src/salesforce.rs`
- **Issue**: No dedicated tests for `start()`/`stop()` lifecycle methods
- **Impact**: Lifecycle behavior not regression-tested
- **Remediation**:
  - Add unit tests for connector startup/shutdown
  - Add integration tests for connection pooling
  - Add stress tests for rapid start/stop cycles
- **Effort**: 1 day
- **Risk**: Low (implementations already working)

**TD-006: Sidecar Integration Tests**
- **Location**: `rust/knhk-sidecar/src/server.rs`
- **Issue**: No end-to-end tests for sidecar server startup and request handling
- **Impact**: Server behavior not fully validated
- **Remediation**:
  - Add gRPC client integration tests
  - Add end-to-end request/response tests
  - Add Weaver live-check integration tests
- **Effort**: 1 day
- **Risk**: Low (server already functional)

### 2.2 Documentation Debt

#### DOC-001: Default Trait Methods Documentation
- **Issue**: No clear documentation on when default trait methods are acceptable vs when they should be overridden
- **Remediation**: Add trait design guide to architecture docs
- **Effort**: 4 hours

#### DOC-002: Configuration Management Guide
- **Issue**: Configuration management is evolving, needs comprehensive guide
- **Remediation**: Write configuration best practices guide
- **Effort**: 4 hours

#### DOC-003: Performance Tuning Guide
- **Issue**: No guide for performance optimization and troubleshooting
- **Remediation**: Write performance tuning guide with benchmarking tools
- **Effort**: 6 hours

### 2.3 Performance Debt

#### PERF-001: CONSTRUCT8 Tick Budget Violation
- **Location**: Hot path CONSTRUCT8 operation
- **Issue**: Takes 41-83 ticks (10-20ns), exceeds 8-tick budget
- **Impact**: CONSTRUCT8 must route to warm path
- **Remediation**:
  - Profile SIMD operation bottlenecks
  - Optimize memory layout for cache efficiency
  - Consider AOT code generation for fixed templates
- **Effort**: 3-5 days
- **Risk**: High (requires deep performance engineering)

### 2.4 Test Infrastructure Debt

#### TEST-001: FFI Symbol Linking in Tests
- **Issue**: C library FFI symbols are `static inline`, not exported for test linking
- **Impact**: Some test files blocked by linker errors
- **Remediation**:
  - Create separate test-only FFI export layer
  - Or compile C library with test exports enabled
- **Effort**: 1 day
- **Risk**: Low (build system change only)

### 2.5 Known Limitations (Documented, Not Bugs)

These are design decisions, not technical debt:

✅ **Acceptable Fallback Behaviors**:
- Timestamp fallbacks to 0 on clock errors (acceptable for telemetry)
- Default config when config file missing (acceptable for development)
- Generated service methods from tonic-build (expected code generation)

---

## 3. V1.1 Release Plan (Bug Fixes & Polish)

**Timeline**: 2-4 weeks post-v1.0
**Focus**: Critical bug fixes, documentation improvements, performance polish
**Release Type**: Patch/Minor (v1.1.0)

### 3.1 Goals

- Fix critical bugs discovered during v1.0 adoption
- Complete high-priority technical debt (TD-001, TD-002)
- Improve documentation based on user feedback
- Enhance error messages and diagnostics
- Add missing test coverage

### 3.2 Feature List

#### Critical Bug Fixes
- [ ] Fix any P0/P1 bugs reported during v1.0 rollout
- [ ] Resolve any Weaver validation violations in production
- [ ] Fix memory leaks or resource exhaustion issues
- [ ] Address security vulnerabilities (if any)

#### Technical Debt Resolution
- [ ] **TD-001**: Implement Rego policy engine integration
- [ ] **TD-002**: Re-enable sidecar service proto implementation
- [ ] **TEST-001**: Fix FFI symbol linking for test suites

#### Documentation Improvements
- [ ] Add troubleshooting guide (common issues and solutions)
- [ ] Improve quick start guide with more examples
- [ ] Document all CLI commands with usage examples
- [ ] Add configuration file reference
- [ ] Create migration guide from any pre-v1.0 versions

#### Enhanced Error Messages
- [ ] Add error codes to all error types (e.g., `KNHK-E001`)
- [ ] Improve error context (include relevant state in error messages)
- [ ] Add suggestions for common errors (e.g., "Did you mean...?")
- [ ] Enhance OTEL span error attributes

#### Additional Test Coverage
- [ ] **TD-005**: Add connector lifecycle tests
- [ ] **TD-006**: Add sidecar integration tests
- [ ] Add chaos engineering tests (random failures, network partitions)
- [ ] Add performance regression test suite

#### Performance Optimizations
- [ ] Profile hot path operations (identify optimization opportunities)
- [ ] Optimize memory allocations in warm path
- [ ] Improve query cache hit rates
- [ ] Reduce OTEL span overhead

### 3.3 Success Criteria

- [ ] All P0/P1 bugs fixed (0 critical bugs)
- [ ] Test coverage ≥90% (critical paths)
- [ ] Documentation completeness ≥95% (all features documented)
- [ ] User satisfaction ≥4.0/5.0 (based on feedback)
- [ ] Performance regression: 0 regressions vs v1.0

### 3.4 Release Checklist

- [ ] All v1.1 features merged to main
- [ ] All tests passing (100%)
- [ ] Weaver registry check passes
- [ ] Weaver live-check passes (0 violations)
- [ ] Performance benchmarks meet SLO
- [ ] Security audit complete
- [ ] Documentation updated
- [ ] CHANGELOG generated
- [ ] Release notes drafted

---

## 4. V1.2 Release Plan (Minor Features)

**Timeline**: 2-3 months post-v1.0
**Focus**: Enhanced observability, additional policy templates, extended connector features
**Release Type**: Minor (v1.2.0)

### 4.1 Goals

- Enhance observability dashboards and metrics
- Add policy templates for common use cases
- Extend Kafka/Salesforce connector capabilities
- Improve developer experience
- Add configuration management features

### 4.2 Feature List

#### Enhanced Observability (HIGH PRIORITY)
- [ ] **Grafana Dashboards**: Pre-built dashboards for KNHK metrics
  - Hot path latency (p50, p95, p99) over time
  - Warm path latency distribution
  - Error rate and types
  - Connector throughput and lag
  - Cache hit rates
  - Receipt generation rate
- [ ] **Jaeger Trace Templates**: Pre-configured trace queries
- [ ] **Alert Rules**: Prometheus alert rules for SLO violations
- [ ] **Health Check API**: Detailed health check endpoint (JSON response)
- [ ] **Metrics Aggregation**: Aggregate metrics across multiple KNHK instances

#### Policy Templates (HIGH PRIORITY)
- [ ] **Guard Policy Templates**:
  - `max_run_len_8` - Enforce 8-beat constraint
  - `performance_budget_8ticks` - Enforce hot path budget
  - `schema_validation` - Enforce IRI format constraints
  - `resource_limits` - Enforce memory/CPU limits
- [ ] **Receipt Validation Templates**:
  - `hash_verification` - Verify `hash(A) = hash(μ(O))`
  - `provenance_chain` - Verify receipt chain integrity
  - `timestamp_monotonicity` - Verify timestamp ordering
- [ ] **Custom Policy DSL**: Simple DSL for policy authoring (alternative to Rego)

#### Extended Connector Features (MEDIUM PRIORITY)
- [ ] **Kafka Connector Enhancements**:
  - Schema Registry integration (Avro, Protobuf)
  - Exactly-once semantics (transactional producer)
  - Dead letter queue (DLQ) for failed messages
  - Backpressure handling
  - Metrics per topic/partition
- [ ] **Salesforce Connector Enhancements**:
  - Bulk API support (large data transfers)
  - CDC (Change Data Capture) support
  - Retry with exponential backoff
  - Field-level security handling
  - Metrics per object type
- [ ] **New Connector Stubs**: SAP, NetSuite, Workday (basic implementations)

#### Configuration Management (MEDIUM PRIORITY)
- [ ] **Configuration Validation**: `knhk config validate` command
- [ ] **Configuration Reloading**: Hot reload on config file change
- [ ] **Environment Profiles**: dev/staging/production profiles
- [ ] **Secret Management**: Integration with vault/secrets manager
- [ ] **Configuration Templates**: Example configurations for common setups

#### Developer Experience Improvements (LOW PRIORITY)
- [ ] **CLI Auto-completion**: Bash/Zsh completion scripts
- [ ] **Interactive Mode**: `knhk interactive` REPL for queries
- [ ] **Verbose Logging**: `--verbose` flag for debugging
- [ ] **Dry Run Mode**: `--dry-run` flag to preview operations
- [ ] **JSON Output Mode**: `--json` flag for machine-readable output

### 4.3 Non-Goals for V1.2

These features are explicitly **out of scope** for v1.2 (deferred to v1.3 or v2.0):

- ❌ Multi-region active-active replication (v2.0)
- ❌ W1/C1 algorithm improvements (v2.0)
- ❌ Event routing redesign (v2.0)
- ❌ GraphQL API (v1.3+)
- ❌ Browser/WebAssembly support (out of scope)

### 4.4 Success Criteria

- [ ] 5+ Grafana dashboards available
- [ ] 10+ policy templates available
- [ ] Kafka connector supports exactly-once semantics
- [ ] Salesforce connector supports Bulk API
- [ ] Configuration hot reload works without downtime
- [ ] User satisfaction ≥4.2/5.0

---

## 5. V2.0 Vision (Major Features)

**Timeline**: 6-12 months post-v1.0
**Focus**: Major architectural improvements, breaking changes, advanced features
**Release Type**: Major (v2.0.0)

### 5.1 Strategic Goals

- Enable global distribution (multi-region active-active)
- Improve cold path query performance (10x improvement)
- Redesign event routing for better scalability
- Support advanced RDF features (reasoning, entailment)

### 5.2 Major Features (Under Discussion)

#### Multi-Region Active-Active Replication (HIGH PRIORITY)
**Motivation**: Support global deployments with local low-latency access

**Design**:
- Distributed Merkle DAG for receipt synchronization
- CRDT-based eventual consistency for knowledge graph state
- Conflict-free concurrent updates
- Regional sharding with global routing

**Challenges**:
- Consensus protocol selection (Raft, Paxos, or custom)
- Network partition handling
- CAP theorem tradeoffs (AP vs CP)
- Receipt ordering across regions

**Effort**: 8-12 weeks
**Risk**: High (distributed systems complexity)

#### W1/C1 Algorithm Improvements (MEDIUM PRIORITY)
**Motivation**: Improve warm/cold path query performance by 10x

**Design**:
- Query plan optimization (cost-based optimizer)
- Join reordering and pushdown optimization
- Index selection (automatic index recommendations)
- Caching at multiple levels (parsed queries, query plans, results)
- SIMD-optimized join algorithms

**Challenges**:
- Query optimizer complexity
- Index maintenance overhead
- Cache invalidation correctness

**Effort**: 6-8 weeks
**Risk**: Medium (well-understood database optimization)

#### Event Routing Redesign (MEDIUM PRIORITY)
**Motivation**: Improve scalability and flexibility of event routing

**Current Limitations**:
- Fixed routing topology (single bus)
- No content-based routing
- Limited backpressure handling

**Proposed Design**:
- Content-based routing (route by predicate, subject, etc.)
- Multiple routing topologies (pub/sub, message queue, stream)
- Backpressure-aware routing (dynamic load balancing)
- Circuit breaker per route

**Effort**: 4-6 weeks
**Risk**: Medium (requires careful migration planning)

#### Advanced RDF Features (LOW PRIORITY)
**Motivation**: Support reasoning and entailment for knowledge graph inference

**Features**:
- RDFS entailment (subclass, subproperty inference)
- OWL reasoning (basic level, no full OWL 2)
- SHACL shapes with reasoning
- Materialized views for inferred triples

**Challenges**:
- Performance impact on hot path (must route to cold path)
- Correctness of reasoning algorithms
- Incremental reasoning (avoid full re-materialization)

**Effort**: 8-12 weeks
**Risk**: High (requires expertise in semantic web)

### 5.3 Breaking Changes in V2.0

**API Changes**:
- Remove deprecated `Query::parse()` (migrate to new oxigraph API)
- Change configuration file format (TOML → YAML for consistency)
- Rename CLI commands (standardize naming conventions)
- Change OTEL span attribute names (align with OpenTelemetry semantic conventions)

**Behavioral Changes**:
- Default caching behavior (enable by default)
- Error handling strategy (change from Result to custom error types)
- Receipt format (add versioning for forward compatibility)

**Migration Guide**:
- Provide automated migration tool (`knhk migrate v1-to-v2`)
- Document all breaking changes in detail
- Offer 6-month deprecation period for v1.x APIs

### 5.4 V2.0 Research & Experimentation

**Research Areas** (may or may not be included in v2.0):

1. **Zero-Copy Serialization**:
   - Use `serde_json` alternatives (e.g., `simd-json`)
   - Custom binary serialization format
   - Protobuf for all data transfer

2. **WASM for User-Defined Functions**:
   - Allow users to define custom hooks in WASM
   - Sandbox untrusted code execution
   - Performance comparison with native Rust

3. **GPU Acceleration for Graph Queries**:
   - Use CUDA/OpenCL for large graph traversals
   - Cost/benefit analysis (GPU overhead vs speedup)

4. **Formal Verification of Hot Path**:
   - Use TLA+ or Alloy to verify correctness
   - Prove that hot path operations are O(1) bounded

---

## 6. Continuous Improvement Strategy

### 6.1 Regular Validation Cadence

**Weekly** (During Active Development):
- [ ] Run all test suites
- [ ] Run performance benchmarks
- [ ] Check for new TODOs in codebase
- [ ] Review open issues and PRs

**Monthly** (Maintenance):
- [ ] Weaver registry validation (ensure schema is current)
- [ ] Dependency updates (cargo update, security audit)
- [ ] Performance regression testing
- [ ] Documentation review (fix broken links, outdated info)

**Quarterly** (Strategic):
- [ ] Roadmap review and adjustment
- [ ] Architecture review (identify emerging debt)
- [ ] Community feedback synthesis
- [ ] Competitive analysis (compare with similar projects)

### 6.2 Security Audit Process

**Schedule**:
- After each minor release (v1.1, v1.2, etc.)
- After any security-related change
- Annually (comprehensive third-party audit)

**Checklist**:
- [ ] `cargo audit` (known vulnerabilities)
- [ ] `cargo deny` (license compliance)
- [ ] Static analysis (clippy with security lints)
- [ ] Fuzzing (libFuzzer for hot path operations)
- [ ] Penetration testing (for sidecar service)

### 6.3 Dependency Update Strategy

**Policy**:
- Update dependencies monthly (unless security fix required)
- Pin major versions (allow minor/patch updates)
- Test all updates in CI/CD before merging
- Document breaking changes from dependencies

**Process**:
1. Run `cargo update` to check for updates
2. Review release notes for breaking changes
3. Update Cargo.toml with new versions
4. Run full test suite
5. Update documentation if APIs changed

### 6.4 Community Feedback Integration

**Channels**:
- GitHub Issues (bug reports, feature requests)
- GitHub Discussions (questions, ideas)
- User surveys (quarterly)
- Office hours (monthly)

**Process**:
1. **Triage**: Label and prioritize all feedback weekly
2. **Synthesis**: Identify common themes monthly
3. **Planning**: Incorporate high-priority feedback into roadmap
4. **Communication**: Respond to all feedback within 48 hours
5. **Recognition**: Credit contributors in release notes

**Metrics**:
- Time to first response (target: <48 hours)
- Issue resolution time (target: <7 days for P0, <30 days for P1)
- User satisfaction score (target: ≥4.0/5.0)

---

## 7. Appendices

### Appendix A: Version Numbering Policy

KNHK follows [Semantic Versioning 2.0](https://semver.org/):

**MAJOR.MINOR.PATCH** (e.g., 1.2.3)

- **MAJOR**: Breaking changes (API, behavior, configuration)
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

**Examples**:
- `v1.0.0` → `v1.0.1`: Bug fix (patch)
- `v1.0.1` → `v1.1.0`: New feature (minor)
- `v1.1.0` → `v2.0.0`: Breaking change (major)

### Appendix B: Deprecation Policy

**Deprecation Timeline**:
1. **Announcement**: Deprecation announced in release notes
2. **Warning Period**: 6 months minimum (warnings in logs/docs)
3. **Removal**: Deprecated feature removed in next major version

**Example**:
- v1.0: Feature `X` works normally
- v1.1: Feature `X` deprecated (warning added)
- v1.2-v1.9: Feature `X` still works (warning continues)
- v2.0: Feature `X` removed (breaking change)

### Appendix C: Release Naming Convention

**Official Release Names**:
- Major versions: Named after cities (v1.0 "Chicago", v2.0 "Boston")
- Minor versions: Named after neighborhoods (v1.1 "Wicker Park", v1.2 "Lincoln Park")

**Internal Codenames** (for development branches):
- v1.1: "Polaris" (bug fixes, polish)
- v1.2: "Aurora" (enhanced observability)
- v1.3: "Nebula" (community features)
- v2.0: "Cosmos" (major architectural improvements)

### Appendix D: Priority Matrix

**Feature Prioritization** (RICE framework):

| Feature | Reach | Impact | Confidence | Effort | RICE Score | Priority |
|---------|-------|--------|------------|--------|------------|----------|
| Rego policy engine | 80% | 3 | 100% | 2 | 120 | P0 |
| Proto service fix | 60% | 2 | 100% | 1 | 120 | P0 |
| Grafana dashboards | 90% | 3 | 100% | 3 | 90 | P1 |
| Kafka exactly-once | 70% | 2 | 80% | 4 | 28 | P2 |
| Multi-region replication | 30% | 3 | 50% | 12 | 3.75 | P3 |

**Legend**:
- **Reach**: % of users impacted
- **Impact**: 1 (low) - 3 (high)
- **Confidence**: % confidence in estimates
- **Effort**: Person-weeks
- **RICE Score**: (Reach × Impact × Confidence) / Effort

### Appendix E: Technical Debt Burn-Down Plan

**Goal**: Reduce technical debt by 50% by v1.3, 80% by v2.0

```
Technical Debt Burn-Down (by item count)

v1.0  |████████████████████| 6 items (baseline)
v1.1  |███████████████     | 5 items (-1: TD-001, TD-002 resolved)
v1.2  |███████████         | 4 items (-1: TD-003 resolved)
v1.3  |███████             | 3 items (-1: TD-004, TD-005 resolved)
v2.0  |███                 | 1 item  (-2: major refactor eliminates TD-006)
```

**Tracking Metrics**:
- Number of TODO comments in codebase
- Number of deprecation warnings
- Technical debt "interest" (time spent working around debt)

---

## Document History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2025-01-06 | Strategic Planning Agent (#11) | Initial roadmap created |

---

## Next Steps

**Immediate Actions** (within 1 week):
1. [ ] Review and approve this roadmap document
2. [ ] Prioritize v1.1 feature list with team
3. [ ] Create GitHub milestones for v1.1, v1.2
4. [ ] Schedule v1.1 release planning meeting

**Short-term Actions** (within 1 month):
1. [ ] Begin work on v1.1 critical bug fixes
2. [ ] Start v1.2 design discussions (observability dashboards)
3. [ ] Establish continuous improvement processes
4. [ ] Launch community feedback channels

**Long-term Actions** (within 6 months):
1. [ ] Complete v1.1 and v1.2 releases
2. [ ] Begin v2.0 design and prototyping
3. [ ] Achieve 50% technical debt reduction
4. [ ] Grow user base and gather feedback for v2.0 planning

---

**Remember**: This roadmap is a living document. It will evolve based on user feedback, technical discoveries, and strategic priorities. Review and update quarterly.
