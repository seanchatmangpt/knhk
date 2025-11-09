# ADR-007: 80/20 Feature Selection for Enterprise Migration

**Status:** Accepted
**Date:** 2025-11-08
**Deciders:** System Architect, Product Management, Enterprise Customers
**Technical Story:** Minimum Viable Product (MVP) for YAWL migration

## Context

YAWL has hundreds of features accumulated over 15+ years. Implementing everything would take years. We need to identify the critical 20% of features that deliver 80% of enterprise value.

## Decision Drivers

- **Time to Market:** Enterprise customers need migration path within 6 months
- **Resource Constraints:** Small team (4-6 engineers)
- **Enterprise Needs:** What do Fortune 500 companies actually use?
- **Technical Debt:** Avoid implementing features nobody uses
- **Competitive Advantage:** Focus on differentiation (sub-tick latency, safety)

## Analysis Methodology

### Data Sources

1. **YAWL Feature Usage Telemetry** (10 enterprise deployments)
   - Instrumented YAWL instances at 3 banks, 2 healthcare, 5 manufacturing
   - 6 months of usage data (Jan-Jun 2025)
   - 50,000+ workflow executions

2. **Enterprise Customer Interviews** (15 Fortune 500 companies)
   - CIOs, Enterprise Architects, Workflow Developers
   - "What YAWL features are business-critical?"
   - "What features would you sacrifice for 10x performance?"

3. **YAWL Source Code Analysis**
   - Code coverage from enterprise test suites
   - Dead code identification
   - API endpoint usage statistics

4. **Workflow Pattern Analysis** (500 enterprise workflows)
   - Pattern frequency distribution
   - Complexity vs usage correlation

---

## Feature Value Matrix

### YAWL Features by Enterprise Value

| Feature | Usage % | Implementation Complexity | ROI Score | Priority |
|---------|---------|--------------------------|-----------|----------|
| **Interface A: Core Engine** | | | | |
| Basic patterns (1-5) | 100% | LOW | ⭐⭐⭐⭐⭐ | MUST |
| Advanced branching (6-11) | 85% | MEDIUM | ⭐⭐⭐⭐⭐ | MUST |
| Multiple instance (12-15) | 70% | HIGH | ⭐⭐⭐⭐ | MUST |
| State-based (16-18) | 60% | MEDIUM | ⭐⭐⭐⭐ | SHOULD |
| Cancellation (19-25) | 45% | HIGH | ⭐⭐⭐ | SHOULD |
| Advanced control (26-39) | 25% | VERY HIGH | ⭐⭐ | DEFER |
| Trigger patterns (40-43) | 15% | MEDIUM | ⭐ | DEFER |
| **Interface B: Work Item & Resource** | | | | |
| 3-phase allocation | 95% | MEDIUM | ⭐⭐⭐⭐⭐ | MUST |
| Work item lifecycle (14 ops) | 95% | MEDIUM-HIGH | ⭐⭐⭐⭐⭐ | MUST |
| Capability filters | 90% | LOW | ⭐⭐⭐⭐⭐ | MUST |
| Role filters | 85% | LOW | ⭐⭐⭐⭐ | SHOULD |
| Separation of duties | 70% | MEDIUM | ⭐⭐⭐⭐ | SHOULD |
| Calendar/scheduling | 40% | HIGH | ⭐⭐ | DEFER |
| Advanced filters (8-10) | 20% | MEDIUM | ⭐ | DEFER |
| **Interface C: Exception Handling** | | | | |
| Basic error handling | 80% | LOW | ⭐⭐⭐⭐ | SHOULD |
| Simple rule engine | 50% | MEDIUM | ⭐⭐⭐ | SHOULD |
| RDR tree learning | 15% | VERY HIGH | ⭐ | DEFER |
| Worklet versioning | 10% | MEDIUM | ⭐ | DEFER |
| Exlet framework | 5% | HIGH | ⭐ | SKIP |
| **Interface D: Custom Services** | | | | |
| REST connector | 85% | LOW | ⭐⭐⭐⭐⭐ | MUST |
| SQL connector | 75% | MEDIUM | ⭐⭐⭐⭐ | SHOULD |
| WASM codelets | 60% | MEDIUM | ⭐⭐⭐⭐ | SHOULD |
| SOAP client | 40% | HIGH | ⭐⭐ | DEFER |
| Java codelets | 30% | VERY HIGH | ⭐ | SKIP |
| Service discovery | 15% | MEDIUM | ⭐ | DEFER |

---

## Decision Outcome

### MVP Feature Set (6-Month Timeline)

#### MUST-HAVE (Weeks 1-12) ⭐⭐⭐⭐⭐

**Interface A: Core Engine**
- ✅ Basic Control Flow (Patterns 1-5)
  - Sequence, Parallel Split, Synchronization, Exclusive Choice, Simple Merge
  - **Value:** 100% of workflows use these
  - **Complexity:** LOW (already implemented)

- ✅ Advanced Branching (Patterns 6-11)
  - Multi-Choice, Structured Sync Merge, Multi-Merge, Discriminator, Arbitrary Cycles
  - **Value:** 85% of workflows
  - **Complexity:** MEDIUM

- ✅ Multiple Instance (Patterns 12-15)
  - MI without Sync, MI with Design-Time Knowledge, MI with Runtime Knowledge
  - **Value:** 70% of workflows (parallel task distribution)
  - **Complexity:** HIGH (but critical for parallelism)

**Interface B: Work Item & Resource**
- 🔴 Work Item Lifecycle (14 operations)
  - create, offer, allocate, start, suspend, resume, complete, fail, etc.
  - **Value:** 95% of workflows have human tasks
  - **Complexity:** MEDIUM-HIGH
  - **Timeline:** Weeks 5-8

- 🔴 3-Phase Allocation
  - Offer → Allocate → Start
  - **Value:** 95% (core of resource management)
  - **Complexity:** MEDIUM
  - **Timeline:** Weeks 9-10

- 🔴 Capability & Role Filters
  - Match resources by skills and org structure
  - **Value:** 90% + 85% = essential filters
  - **Complexity:** LOW
  - **Timeline:** Week 11

**Interface D: Integration**
- 🔴 REST Connector
  - HTTP client for external APIs
  - **Value:** 85% of workflows integrate external systems
  - **Complexity:** LOW (reuse reqwest crate)
  - **Timeline:** Week 13

---

#### SHOULD-HAVE (Weeks 13-20) ⭐⭐⭐⭐

**Interface A: Core Engine**
- 🔴 State-Based Patterns (16-18)
  - Deferred Choice, Interleaved Parallel Routing, Milestone
  - **Value:** 60% of workflows
  - **Complexity:** MEDIUM
  - **Defer if timeline pressure**

- 🔴 Cancellation Patterns (19-25)
  - Cancel Activity, Cancel Case, Cancel Region
  - **Value:** 45% of workflows
  - **Complexity:** HIGH (complex state management)

**Interface B: Resource Management**
- 🔴 Separation of Duties Constraint
  - 4-eyes principle enforcement
  - **Value:** 70% of workflows (compliance-driven)
  - **Complexity:** MEDIUM
  - **Timeline:** Week 14

- 🔴 Basic Privilege Management
  - Can-do, can-pile, can-start permissions
  - **Value:** 60% of workflows
  - **Complexity:** LOW-MEDIUM
  - **Timeline:** Week 15

**Interface D: Integration**
- 🔴 SQL Connector
  - PostgreSQL, MySQL, Oracle integration
  - **Value:** 75% of workflows query databases
  - **Complexity:** MEDIUM (use sqlx crate)
  - **Timeline:** Weeks 16-17

- 🔴 WASM Codelets
  - Secure sandboxed execution
  - **Value:** 60% of workflows have custom logic
  - **Complexity:** MEDIUM (use wasmtime)
  - **Timeline:** Weeks 18-19

**Interface C: Exception Handling**
- 🔴 Basic Error Handling
  - Try-catch patterns, error propagation
  - **Value:** 80% of workflows
  - **Complexity:** LOW
  - **Timeline:** Week 20

---

#### DEFER TO v2.0 (Post-MVP) ⭐⭐

**Interface A: Advanced Patterns**
- Advanced Control (Patterns 26-39)
  - Critical Section, Interleaved Routing, Thread Merge, etc.
  - **Reason:** Only 25% usage, very high complexity

- Trigger Patterns (40-43)
  - Transient Trigger, Persistent Trigger
  - **Reason:** 15% usage, better handled by event system

**Interface B: Advanced Resource**
- Calendar/Scheduling Service
  - **Reason:** 40% usage, high complexity, workaround available
- Advanced Filters (8-10)
  - **Reason:** 20% usage, diminishing returns

**Interface C: Worklets**
- RDR Learning
  - **Reason:** 15% usage, very high complexity
- Worklet Versioning
  - **Reason:** 10% usage, can use git for versioning

**Interface D: Legacy Integration**
- SOAP Client
  - **Reason:** 40% usage but declining (REST replacing SOAP)
- Service Discovery
  - **Reason:** 15% usage, manual config sufficient

---

#### SKIP ENTIRELY ⭐

**Features to Permanently Skip:**
- Java Codelets (security risk, complexity)
  - **Replacement:** WASM codelets (safer, cross-language)
- Exlet Framework (5% usage)
  - **Replacement:** Use regular connectors
- Advanced Trigger Patterns (15% usage)
  - **Replacement:** Event-driven architecture
- YAWL Editor Integration (outdated UI)
  - **Replacement:** Build modern web UI for Turtle

---

## 80/20 Analysis Summary

### Coverage by Implementation Phase

| Phase | Timeline | Features | Enterprise Coverage | Effort (Person-Weeks) |
|-------|----------|----------|-------------------|---------------------|
| **MVP (MUST)** | Weeks 1-12 | 12 features | **82%** | 48 weeks (4 devs × 12 weeks) |
| **MVP + SHOULD** | Weeks 1-20 | 20 features | **92%** | 80 weeks (4 devs × 20 weeks) |
| **Full v2.0** | 12 months | 45 features | **98%** | 200 weeks (est.) |

**Key Insight:** Implementing 27% of YAWL features (12 of 45) covers 82% of enterprise workflows.

---

## Rationale

### Why Interface B is the Critical 20%

**Data Point:** Work item + resource features are used in **95% of enterprise workflows**
- Human tasks are ubiquitous (approvals, reviews, data entry)
- Resource allocation is compliance-critical (separation of duties, audit trail)
- Without Interface B, workflows stop (no way to execute human tasks)

**Decision:** Interface B is the #1 priority after core engine

### Why Defer Advanced Patterns (26-43)

**Data Point:** Patterns 26-43 are used in only **20% of workflows**
- Complex patterns (critical section, thread split) rarely needed
- Most workflows use basic control flow (1-25)
- When needed, can workaround with decomposition

**Decision:** Defer patterns 26-43 to v2.0, focus on 1-25

### Why WASM Over Java Codelets

**Security:** WASM is sandboxed by design (Java has classpath vulnerabilities)
**Performance:** WASM compiles to native code (no JIT warmup)
**Cross-Language:** WASM supports Rust, C, C++, AssemblyScript (not just Java)
**Rust Ecosystem:** wasmtime crate integrates seamlessly

**Decision:** Skip Java codelets, implement WASM (covers 90% of codelet use cases)

### Why Sled Over Full XQuery

**Data Point:** Only 25% of workflows use XQuery for data transformation
**Workaround:** Use serde_json + JSONPath (simpler, 80% coverage)
**Complexity:** Full XQuery requires Saxon JNI (adds JVM dependency)

**Decision:** Defer XQuery to v2.0, use JSON transformations initially

---

## Implementation Constraints

### Fixed Constraints

1. **Team Size:** 4-6 engineers (1 architect, 3-5 implementers)
2. **Timeline:** 6 months to MVP (enterprise migration deadline)
3. **Performance:** Sub-tick latency (<8 ticks) non-negotiable
4. **Quality:** Zero memory safety bugs (Rust compiler enforced)

### Variable Constraints

1. **Feature Scope:** MUST vs SHOULD vs DEFER
2. **Test Coverage:** 80% minimum (not 100%)
3. **Documentation:** Focus on migration guide (not every API)

---

## Success Metrics

### MVP Acceptance Criteria

**Functional:**
- [ ] 82% of enterprise workflows executable
- [ ] All MUST-HAVE features implemented
- [ ] YAWL XML import working
- [ ] Work item allocation end-to-end

**Performance:**
- [ ] Hot path <8 ticks (sub-tick latency)
- [ ] 10,000 concurrent cases supported
- [ ] <100ms p99 API latency

**Quality:**
- [ ] Zero memory safety bugs (enforced by Rust)
- [ ] 80%+ test coverage
- [ ] All Clippy warnings resolved

**Migration:**
- [ ] 10 enterprise customers migrated successfully
- [ ] Dual-run mode (YAWL + KNHK parallel) for validation

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Underestimate SHOULD-HAVE complexity | Medium | High | Phase implementation, cut features if needed |
| Enterprise customers need DEFER features | Low | High | Hybrid mode (YAWL for advanced, KNHK for core) |
| Performance regression | Low | Critical | Continuous benchmarking, sub-tick tests |
| Migration bugs | High | High | Dual-run mode, gradual cutover |

---

## References

- [Pareto Principle (80/20 Rule)](https://en.wikipedia.org/wiki/Pareto_principle)
- [YAWL Feature Usage Telemetry Report](/docs/analysis/yawl-telemetry-2025.pdf)
- [Enterprise Customer Interview Summary](/docs/research/customer-interviews-q1-2025.md)
- [Workflow Pattern Frequency Analysis](/docs/analysis/pattern-frequency.csv)

---

## Related Decisions

- ADR-001: Rust over Java (enables focusing on critical features)
- ADR-008: Interface B priority (the critical 20%)
- ADR-009: RDR vs simple rules (defer complexity)
