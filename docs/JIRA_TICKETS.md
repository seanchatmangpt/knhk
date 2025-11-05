# Jira Tickets: KNHK v1.0 Integration & Optimization

**Project**: KNHK  
**Epic**: KNHK-LSS-2024-001  
**Sprint**: Q1 2025  
**Methodology**: DMAIC (Lean Six Sigma)

---

## Epic Ticket

### EPIC-001: KNHK v1.0 Integration & Cold Path Optimization

**Issue Type**: Epic  
**Priority**: Highest  
**Labels**: `v1.0`, `integration`, `unrdf`, `dmaic`, `performance`  
**Components**: `integration`, `cold-path`, `unrdf-wrapper`  
**Affects Version**: v1.0  
**Fix Version**: v1.0  

**Summary**: Eliminate all integration gaps between KNHK and unrdf, achieving 100% feature parity for cold path operations while maintaining ≤8 tick hot path performance

**Description**:
```
KNHK v0.4.0 is production-ready for hot path operations (≤8 ticks) but has critical gaps in cold path integration with unrdf. To achieve v1.0 readiness, we must eliminate integration gaps, optimize query routing, and ensure seamless hot/warm/cold path coordination.

**Current State**:
- ✅ Hot path: 18/19 operations achieving ≤8 ticks
- ⚠️ 9 critical integration gaps identified
- ⚠️ Cold path queries unoptimized (no caching, batching)
- ⚠️ Separate implementations (lockchain, OTEL) create technical debt

**Desired State**:
- ✅ Full SPARQL 1.1 support via unrdf cold path integration
- ✅ Complete SHACL validation wrapper
- ✅ ACID transaction management
- ✅ RDF serialization (Turtle, JSON-LD, N-Quads)
- ✅ Complete hook management lifecycle
- ✅ Unified lockchain with aligned hash algorithms
- ✅ Unified OTEL observability
- ✅ Optimized query routing with caching and batching
```

**Acceptance Criteria**:
- [ ] 9/9 critical gaps resolved (100%)
- [ ] Hot path maintains ≤8 ticks (0% degradation)
- [ ] Cold path p95 latency ≤500ms (50% improvement)
- [ ] 100% of unrdf features wrapped/accessible
- [ ] 90%+ integration test coverage
- [ ] Query cache hit rate ≥50%
- [ ] Zero integration errors in production

**Business Value**: Enable 80% of enterprise use cases through unified hot/cold path architecture. ROI: 3:1 (value:cost ratio).

**Dependencies**: 
- unrdf v3.0.0 API stability
- Node.js runtime availability
- Testing infrastructure

**Risks**:
- Hot path performance degradation (Low probability, Critical impact)
- unrdf API changes (Medium probability, High impact)
- Timeline delays (Medium probability, Medium impact)

**Story Points**: 100 (Epic-level estimate)

---

## Story Tickets

### STORY-001: SHACL Validation Wrapper (P0)

**Issue Type**: Story  
**Priority**: Highest  
**Labels**: `p0`, `shacl`, `validation`, `critical`  
**Components**: `integration`, `unrdf-wrapper`  
**Sprint**: Sprint 1 (Weeks 5-6)  
**Story Points**: 13  

**Summary**: Wrap unrdf's full SHACL validation for cold path validation

**Description**:
```
KNHK currently has micro-shape validation in hot path (limited to datatype checks), but lacks full SHACL validation wrapper for cold path operations. unrdf provides complete SHACL Core specification via rdf-validate-shacl.

**Current Gap**:
- KNHK: Micro-shapes only (VALIDATE_DATATYPE_SP/SPO)
- unrdf: Full SHACL validation with all constraint types
- Missing: Wrapper function to access unrdf SHACL validation

**Implementation**:
1. Wrap unrdf's `validateShacl()` function
2. Implement shape graph loading and management
3. Add validation result serialization (JSON format)
4. Add error handling and validation

**API Design**:
```c
int knhk_unrdf_validate_shacl(
    const char *data_turtle,
    const char *shapes_turtle,
    char *result_json,
    size_t result_size
);
```
```

**Acceptance Criteria**:
- [ ] `knhk_unrdf_validate_shacl()` function implemented
- [ ] Shape graph loading works correctly
- [ ] Validation results serialized to JSON
- [ ] Error handling for invalid inputs
- [ ] Integration tests passing
- [ ] Documentation updated

**Tasks**:
- [ ] Implement Rust wrapper function
- [ ] Add shape graph management
- [ ] Implement result serialization
- [ ] Add error handling
- [ ] Write integration tests
- [ ] Update documentation

**Dependencies**: None  
**Estimated Hours**: 40 hours

---

### STORY-002: Transaction Management Integration (P0)

**Issue Type**: Story  
**Priority**: Highest  
**Labels**: `p0`, `transaction`, `acid`, `critical`  
**Components**: `integration`, `unrdf-wrapper`  
**Sprint**: Sprint 1 (Weeks 5-6)  
**Story Points**: 13  

**Summary**: Wrap unrdf's ACID transaction manager for cold path operations

**Description**:
```
KNHK lacks ACID transaction support for cold path operations. unrdf provides TransactionManager with ACID guarantees, rollback support, and hook lifecycle integration.

**Current Gap**:
- KNHK: No transaction management (hot path is atomic by design)
- unrdf: Full ACID transaction support
- Missing: Transaction wrapper functions

**Implementation**:
1. Wrap unrdf's `executeTransaction()` function
2. Implement transaction begin/commit/rollback
3. Add transaction state management
4. Integrate with hook lifecycle

**API Design**:
```c
int knhk_unrdf_transaction_begin(void);
int knhk_unrdf_transaction_commit(int transaction_id, char *receipt_json, size_t receipt_size);
int knhk_unrdf_transaction_rollback(int transaction_id);
int knhk_unrdf_transaction_add(int transaction_id, const char *turtle_data);
int knhk_unrdf_transaction_remove(int transaction_id, const char *turtle_data);
```
```

**Acceptance Criteria**:
- [ ] Transaction begin/commit/rollback functions implemented
- [ ] ACID guarantees verified
- [ ] Rollback works correctly on errors
- [ ] Transaction state persists across operations
- [ ] Integration tests passing
- [ ] Documentation updated

**Tasks**:
- [ ] Implement transaction wrapper functions
- [ ] Add transaction state management
- [ ] Implement rollback logic
- [ ] Add error handling
- [ ] Write integration tests
- [ ] Update documentation

**Dependencies**: STORY-001 (SHACL validation)  
**Estimated Hours**: 40 hours

---

### STORY-003: Full SPARQL Query Type Support (P0)

**Issue Type**: Story  
**Priority**: Highest  
**Labels**: `p0`, `sparql`, `query`, `critical`  
**Components**: `integration`, `unrdf-wrapper`  
**Sprint**: Sprint 1 (Weeks 5-6)  
**Story Points**: 13  

**Summary**: Expand SPARQL query routing to support all query types (ASK, CONSTRUCT, DESCRIBE, UPDATE)

**Description**:
```
Currently only basic SELECT queries are routed to unrdf. Need to expand query routing to support all SPARQL 1.1 query types.

**Current Gap**:
- KNHK: Only basic SELECT queries wrapped
- unrdf: Full SPARQL 1.1 support (SELECT, ASK, CONSTRUCT, DESCRIBE, UPDATE)
- Missing: Query type detection and routing for all types

**Implementation**:
1. Implement query type detection (parse SPARQL to determine type)
2. Expand `knhk_unrdf_query()` to handle all query types
3. Add result parsing for each query type
4. Implement query-specific error handling

**API Design**:
```c
int knhk_unrdf_query_ask(const char *query, int *result);
int knhk_unrdf_query_construct(const char *query, char *result_json, size_t result_size);
int knhk_unrdf_query_describe(const char *query, char *result_json, size_t result_size);
int knhk_unrdf_query_update(const char *query, char *result_json, size_t result_size);
```
```

**Acceptance Criteria**:
- [ ] Query type detection works correctly
- [ ] All SPARQL query types supported (ASK, CONSTRUCT, DESCRIBE, UPDATE)
- [ ] Result parsing works for each query type
- [ ] Error handling for invalid queries
- [ ] Integration tests for each query type
- [ ] Documentation updated

**Tasks**:
- [ ] Implement query type detection
- [ ] Add ASK query wrapper
- [ ] Add CONSTRUCT query wrapper
- [ ] Add DESCRIBE query wrapper
- [ ] Add UPDATE query wrapper
- [ ] Add result parsing for each type
- [ ] Write integration tests
- [ ] Update documentation

**Dependencies**: None  
**Estimated Hours**: 40 hours

---

### STORY-004: RDF Serialization (P1)

**Issue Type**: Story  
**Priority**: High  
**Labels**: `p1`, `rdf`, `serialization`, `high-priority`  
**Components**: `integration`, `unrdf-wrapper`  
**Sprint**: Sprint 2 (Week 7)  
**Story Points**: 8  

**Summary**: Add RDF serialization support (Turtle, JSON-LD, N-Quads)

**Description**:
```
KNHK can parse RDF data but cannot serialize it. unrdf provides serialization functions (toTurtle, toJsonLd, toNQuads).

**Current Gap**:
- KNHK: Can parse Turtle/N-Quads (Raptor, rio_turtle)
- unrdf: Can serialize to Turtle, JSON-LD, N-Quads
- Missing: Serialization wrapper functions

**Implementation**:
Option A: Wrap unrdf serialization functions
Option B: Implement independent serialization (recommended for Turtle)

**API Design**:
```c
int knhk_unrdf_to_turtle(char *turtle_output, size_t output_size);
int knhk_unrdf_to_jsonld(char *jsonld_output, size_t output_size);
int knhk_unrdf_to_nquads(char *nquads_output, size_t output_size);
```
```

**Acceptance Criteria**:
- [ ] Turtle serialization works correctly
- [ ] JSON-LD serialization works correctly
- [ ] N-Quads serialization works correctly
- [ ] Output formats match unrdf standards
- [ ] Integration tests passing
- [ ] Documentation updated

**Tasks**:
- [ ] Implement Turtle serialization (wrap or implement)
- [ ] Implement JSON-LD serialization wrapper
- [ ] Implement N-Quads serialization wrapper
- [ ] Add format validation
- [ ] Write integration tests
- [ ] Update documentation

**Dependencies**: None  
**Estimated Hours**: 24 hours

---

### STORY-005: Hook Management Lifecycle (P1)

**Issue Type**: Story  
**Priority**: High  
**Labels**: `p1`, `hooks`, `lifecycle`, `high-priority`  
**Components**: `integration`, `unrdf-wrapper`  
**Sprint**: Sprint 2 (Week 7)  
**Story Points**: 8  

**Summary**: Expand hook management to support registration, deregistration, and lifecycle management

**Description**:
```
Currently only basic hook execution is available. Need to add full hook lifecycle management (register, deregister, list, persist).

**Current Gap**:
- KNHK: Basic hook execution only (`knhk_unrdf_execute_hook()`)
- unrdf: Full hook management (register, deregister, list, lifecycle)
- Missing: Hook management wrapper functions

**Implementation**:
1. Wrap unrdf's hook registration/deregistration functions
2. Implement hook persistence
3. Add hook listing functionality
4. Support all hook types (SHACL, delta, threshold, count, window)

**API Design**:
```c
int knhk_unrdf_register_hook(const char *hook_json, char *hook_id, size_t id_size);
int knhk_unrdf_deregister_hook(const char *hook_id);
int knhk_unrdf_list_hooks(char *hooks_json, size_t hooks_size);
```
```

**Acceptance Criteria**:
- [ ] Hook registration works correctly
- [ ] Hook deregistration works correctly
- [ ] Hook listing works correctly
- [ ] Hook persistence works
- [ ] All hook types supported
- [ ] Integration tests passing
- [ ] Documentation updated

**Tasks**:
- [ ] Implement hook registration wrapper
- [ ] Implement hook deregistration wrapper
- [ ] Implement hook listing wrapper
- [ ] Add hook persistence
- [ ] Support all hook types
- [ ] Write integration tests
- [ ] Update documentation

**Dependencies**: STORY-001 (SHACL validation)  
**Estimated Hours**: 24 hours

---

### STORY-006: Lockchain Hash Algorithm Alignment (P1)

**Issue Type**: Story  
**Priority**: High  
**Labels**: `p1`, `lockchain`, `hash`, `high-priority`  
**Components**: `integration`, `lockchain`  
**Sprint**: Sprint 2 (Week 7)  
**Story Points**: 8  

**Summary**: Align lockchain hash algorithms (SHA-256 vs SHA3-256) for unified receipt format

**Description**:
```
KNHK uses SHA-256 for lockchain, unrdf uses SHA3-256. Need to decide on unified approach and implement alignment.

**Current Gap**:
- KNHK: SHA-256 hashing (URDNA2015 canonicalization)
- unrdf: SHA3-256 hashing (Git-based lockchain)
- Issue: Cannot cross-reference receipts

**Decision Required**:
Option A: Migrate KNHK to SHA3-256
Option B: Migrate unrdf to SHA-256
Option C: Support both with adapter pattern

**Implementation**:
1. Decide on unified hash algorithm
2. Implement hash algorithm conversion/adapter
3. Unify receipt format
4. Update lockchain implementations

**Acceptance Criteria**:
- [ ] Unified hash algorithm decision made
- [ ] Receipt format unified
- [ ] Cross-referencing works correctly
- [ ] Migration path documented
- [ ] Integration tests passing
- [ ] Documentation updated

**Tasks**:
- [ ] Research hash algorithm options
- [ ] Make decision on unified algorithm
- [ ] Implement hash adapter/converter
- [ ] Unify receipt format
- [ ] Update lockchain implementations
- [ ] Write migration tests
- [ ] Update documentation

**Dependencies**: None  
**Estimated Hours**: 24 hours

---

### STORY-007: Unified OTEL Observability (P2)

**Issue Type**: Story  
**Priority**: Medium  
**Labels**: `p2`, `otel`, `observability`, `medium-priority`  
**Components**: `integration`, `observability`  
**Sprint**: Sprint 3 (Week 8)  
**Story Points**: 5  

**Summary**: Unify OTEL spans and metrics across hot/warm/cold paths

**Description**:
```
KNHK and unrdf have separate OTEL implementations. Need to unify spans/metrics for end-to-end observability.

**Current Gap**:
- KNHK: Independent OTEL integration (knhk-otel crate)
- unrdf: Independent OTEL integration (ObservabilityManager)
- Issue: Separate spans/metrics, no unified view

**Implementation**:
1. Integrate unrdf Observability class
2. Unify span creation across paths
3. Aggregate performance metrics
4. Ensure span linking works correctly

**Acceptance Criteria**:
- [ ] Unified OTEL spans across paths
- [ ] Metrics aggregated correctly
- [ ] Span linking works end-to-end
- [ ] Performance metrics visible
- [ ] Integration tests passing
- [ ] Documentation updated

**Tasks**:
- [ ] Integrate unrdf Observability class
- [ ] Unify span creation
- [ ] Aggregate metrics
- [ ] Add span linking
- [ ] Write integration tests
- [ ] Update documentation

**Dependencies**: STORY-002 (Transaction Management)  
**Estimated Hours**: 16 hours

---

### STORY-008: Query Optimization Exposure (P2)

**Issue Type**: Story  
**Priority**: Medium  
**Labels**: `p2`, `optimization`, `caching`, `medium-priority`  
**Components**: `integration`, `performance`  
**Sprint**: Sprint 3 (Week 8)  
**Story Points**: 5  

**Summary**: Expose unrdf optimization features (query caching, hook batching) via KNHK API

**Description**:
```
unrdf provides Dark Matter 80/20 optimizations (caching, batching) but they're not exposed via KNHK API.

**Current Gap**:
- unrdf: Query caching, hook batching, parallel execution
- KNHK: No access to optimization features
- Missing: Optimization API exposure

**Implementation**:
1. Expose query caching configuration
2. Enable hook batching
3. Add performance metrics collection
4. Document optimization usage

**Acceptance Criteria**:
- [ ] Query caching configurable
- [ ] Hook batching enabled
- [ ] Performance metrics collected
- [ ] Cache hit rate ≥50%
- [ ] Integration tests passing
- [ ] Documentation updated

**Tasks**:
- [ ] Expose caching configuration
- [ ] Enable hook batching
- [ ] Add metrics collection
- [ ] Write performance tests
- [ ] Update documentation

**Dependencies**: STORY-003 (Full SPARQL Support)  
**Estimated Hours**: 16 hours

---

### STORY-009: Architecture Refactoring (P0)

**Issue Type**: Story  
**Priority**: Highest  
**Labels**: `p0`, `architecture`, `refactoring`, `critical`  
**Components**: `integration`, `unrdf-wrapper`  
**Sprint**: Sprint 1 (Weeks 5-6)  
**Story Points**: 13  

**Summary**: Refactor unrdf integration from script-based to persistent instance architecture

**Description**:
```
Current implementation creates new unrdf instances per operation (script-based). Need to implement persistent instance with connection pooling.

**Current Gap**:
- Current: Script-based execution (temporary files, new instances)
- Desired: Persistent unrdf instance with connection pooling
- Missing: Instance management, connection pooling

**Implementation**:
1. Implement persistent unrdf system instance
2. Add connection pooling for Node.js processes
3. Enhance error handling with detailed error types
4. Refactor from script execution to direct API calls (if possible)

**Acceptance Criteria**:
- [ ] Persistent unrdf instance implemented
- [ ] Connection pooling works correctly
- [ ] Error handling improved
- [ ] Performance improved (50% reduction in overhead)
- [ ] Integration tests passing
- [ ] Documentation updated

**Tasks**:
- [ ] Design persistent instance architecture
- [ ] Implement instance management
- [ ] Add connection pooling
- [ ] Enhance error handling
- [ ] Refactor script execution
- [ ] Write integration tests
- [ ] Update documentation

**Dependencies**: None  
**Estimated Hours**: 40 hours

---

### STORY-010: Integration Test Suite (P0)

**Issue Type**: Story  
**Priority**: Highest  
**Labels**: `p0`, `testing`, `integration-tests`, `critical`  
**Components**: `testing`, `integration`  
**Sprint**: Sprint 1 (Weeks 5-6)  
**Story Points**: 8  

**Summary**: Create comprehensive integration test suite for unrdf integration

**Description**:
```
Need comprehensive integration tests covering all unrdf integration features to ensure 90%+ coverage.

**Test Coverage**:
- SHACL validation tests
- Transaction management tests
- SPARQL query type tests (all types)
- RDF serialization tests
- Hook management tests
- Lockchain integration tests
- OTEL observability tests
- Performance tests

**Acceptance Criteria**:
- [ ] Test suite covers all integration features
- [ ] Test coverage ≥90%
- [ ] Performance tests included
- [ ] Tests run in CI/CD pipeline
- [ ] Test documentation complete

**Tasks**:
- [ ] Design test suite structure
- [ ] Implement SHACL validation tests
- [ ] Implement transaction tests
- [ ] Implement SPARQL query tests
- [ ] Implement serialization tests
- [ ] Implement hook management tests
- [ ] Implement performance tests
- [ ] Add CI/CD integration
- [ ] Document test suite

**Dependencies**: STORY-001 through STORY-009  
**Estimated Hours**: 24 hours

---

## Subtasks Template

### Subtask Template: [Feature Name] Implementation

**Issue Type**: Subtask  
**Parent**: [STORY-XXX]  
**Summary**: Implement [specific feature component]

**Description**:
```
[Detailed description of subtask]
```

**Acceptance Criteria**:
- [ ] [Specific acceptance criteria]
- [ ] [Code review completed]
- [ ] [Tests written and passing]

**Estimated Hours**: [X] hours

---

## Sprint Planning

### Sprint 1: Critical Features (Weeks 5-6)
- STORY-001: SHACL Validation Wrapper (13 pts)
- STORY-002: Transaction Management (13 pts)
- STORY-003: Full SPARQL Support (13 pts)
- STORY-009: Architecture Refactoring (13 pts)
- STORY-010: Integration Test Suite (8 pts)

**Total**: 60 story points

### Sprint 2: High Priority Features (Week 7)
- STORY-004: RDF Serialization (8 pts)
- STORY-005: Hook Management (8 pts)
- STORY-006: Lockchain Alignment (8 pts)

**Total**: 24 story points

### Sprint 3: Optimization & Polish (Week 8)
- STORY-007: Unified OTEL (5 pts)
- STORY-008: Query Optimization (5 pts)

**Total**: 10 story points

### Sprint 4: Control & Deployment (Weeks 9-10)
- Documentation complete
- Production deployment
- Monitoring setup
- Training materials

**Total**: 6 story points (non-development)

---

## Labels Reference

**Priority Labels**:
- `p0` - Critical (Must Have)
- `p1` - High Priority (Should Have)
- `p2` - Medium Priority (Nice to Have)

**Feature Labels**:
- `shacl` - SHACL validation
- `transaction` - Transaction management
- `sparql` - SPARQL queries
- `rdf` - RDF operations
- `hooks` - Knowledge hooks
- `lockchain` - Lockchain/provenance
- `otel` - Observability
- `optimization` - Performance optimization
- `architecture` - Architecture changes
- `testing` - Testing-related

**Status Labels**:
- `critical` - Critical path
- `high-priority` - High priority
- `medium-priority` - Medium priority
- `blocked` - Blocked by dependencies
- `ready-for-review` - Ready for code review

---

## Jira Import Format

### CSV Format (for Jira import)

```csv
Issue Type,Summary,Description,Priority,Labels,Components,Story Points,Epic Link,Sprint
Epic,KNHK v1.0 Integration & Cold Path Optimization,"Eliminate all integration gaps between KNHK and unrdf",Highest,"v1.0,integration,unrdf",integration,100,EPIC-001,
Story,SHACL Validation Wrapper,"Wrap unrdf's full SHACL validation",Highest,"p0,shacl",integration,13,EPIC-001,Sprint 1
Story,Transaction Management Integration,"Wrap unrdf's ACID transaction manager",Highest,"p0,transaction",integration,13,EPIC-001,Sprint 1
Story,Full SPARQL Query Type Support,"Expand SPARQL query routing to all types",Highest,"p0,sparql",integration,13,EPIC-001,Sprint 1
Story,Architecture Refactoring,"Refactor unrdf integration architecture",Highest,"p0,architecture",integration,13,EPIC-001,Sprint 1
Story,Integration Test Suite,"Create comprehensive integration tests",Highest,"p0,testing",testing,8,EPIC-001,Sprint 1
Story,RDF Serialization,"Add RDF serialization support",High,"p1,rdf",integration,8,EPIC-001,Sprint 2
Story,Hook Management Lifecycle,"Expand hook management API",High,"p1,hooks",integration,8,EPIC-001,Sprint 2
Story,Lockchain Hash Algorithm Alignment,"Align lockchain hash algorithms",High,"p1,lockchain",lockchain,8,EPIC-001,Sprint 2
Story,Unified OTEL Observability,"Unify OTEL spans and metrics",Medium,"p2,otel",observability,5,EPIC-001,Sprint 3
Story,Query Optimization Exposure,"Expose unrdf optimization features",Medium,"p2,optimization",performance,5,EPIC-001,Sprint 3
```

---

## Jira REST API JSON Format

See `docs/jira-tickets-json.json` for complete JSON format suitable for Jira REST API import.

---

**Status**: Ready for Jira Import  
**Next Action**: Import tickets into Jira project

