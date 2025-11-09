# YAWL to knhk Implementation Priorities

**⚠️ This document has been consolidated. See the [80/20 YAWL Integration Guide](YAWL_INTEGRATION.md) for the single source of truth.**

This file is kept for backward compatibility. All new documentation should reference the consolidated guide.

---

# YAWL to knhk Implementation Priorities

**Quick Reference Guide for Swarm Coordination**

---

## Critical Path (Sprint 1-2, ~10-15 days)

### 1. Multiple Instance Execution ⚠️ BLOCKER
**File**: `rust/knhk-workflow-engine/src/executor/task.rs:196-205`
**Problem**: MI patterns skip execution with debug message
**Impact**: Patterns 12-15 not fully functional
**Effort**: 2-3 days

**Implementation Steps**:
```rust
// 1. Design task spawning infrastructure
// 2. Add instance-specific data management
// 3. Implement parallel instance execution
// 4. Add MI synchronization logic
// 5. Test all MI patterns (12-15)
```

**Test Requirements**:
- ✅ Pattern 12: MI Without Synchronization
- ✅ Pattern 13: MI With Design-Time Knowledge
- ✅ Pattern 14: MI With Runtime Knowledge
- ✅ Pattern 15: MI Without Runtime Knowledge

---

### 2. Connector Framework ⚠️ BLOCKER
**File**: `rust/knhk-workflow-engine/src/executor/task.rs:158-162`
**Problem**: Automated tasks fail without connector integration
**Impact**: Cannot invoke external systems/services
**Effort**: 3-5 days

**Implementation Steps**:
```rust
// 1. Design connector interface trait
pub trait Connector {
    async fn invoke(&self, params: HashMap<String, Value>) -> Result<Value>;
}

// 2. Implement HTTP/gRPC connectors in knhk-connectors
// 3. Add connector registry to WorkflowEngine
// 4. Update task.rs to use connectors for automated tasks
// 5. Test external service invocation
```

**Test Requirements**:
- ✅ HTTP POST/GET invocation
- ✅ gRPC service calls
- ✅ Error handling & retry logic
- ✅ Timeout management

---

### 3. gRPC API Handlers ⚠️ HIGH
**File**: `rust/knhk-workflow-engine/src/api/grpc.rs`
**Problem**: Proto defined but handlers not implemented
**Impact**: gRPC interface non-functional
**Effort**: 2-3 days

**Implementation Steps**:
```rust
// 1. Implement WorkflowService trait from proto
// 2. Map gRPC methods to executor operations
// 3. Add error handling & conversion
// 4. Integration tests
// 5. Performance benchmarks
```

---

## High Priority (Sprint 3-4, ~10-12 days)

### 4. Worklet Execution Refactoring ⚠️ MEDIUM
**File**: `rust/knhk-workflow-engine/src/worklets/mod.rs:353`
**Problem**: Circular dependency prevents worklet execution
**Impact**: Exception handling via worklets incomplete
**Effort**: 3-4 days

**Current Code**:
```rust
// Line 353: Cannot access WorkflowEngine from WorkletExecutor
Err(WorkflowError::Internal(
    format!("Worklet execution requires WorkflowEngine integration...")
))
```

**Solution**:
```rust
// Use dependency injection pattern
pub struct WorkletExecutor {
    repository: Arc<WorkletRepository>,
    engine: Weak<WorkflowEngine>, // Weak reference breaks cycle
}

impl WorkletExecutor {
    pub async fn execute_worklet(
        &self,
        worklet_id: WorkletId,
        context: PatternExecutionContext,
    ) -> WorkflowResult<PatternExecutionResult> {
        let worklet = self.repository.get(worklet_id).await?;

        // Upgrade weak reference to access engine
        if let Some(engine) = self.engine.upgrade() {
            engine.execute_worklet_spec(&worklet.workflow_spec, context).await
        } else {
            Err(WorkflowError::Internal("Engine not available".into()))
        }
    }
}
```

---

### 5. Resource Calendar System ⚠️ MEDIUM
**Files**: `rust/knhk-workflow-engine/src/resource/allocation.rs`
**Problem**: No time-based resource availability
**Impact**: Cannot model shifts, rosters, time zones
**Effort**: 4-5 days

**Implementation Steps**:
```rust
// 1. Add Calendar trait
pub trait Calendar {
    fn is_available(&self, resource_id: ResourceId, time: DateTime<Utc>) -> bool;
}

// 2. Implement shift-based calendar
pub struct ShiftCalendar {
    shifts: HashMap<ResourceId, Vec<Shift>>,
}

// 3. Integrate with ResourceAllocator
// 4. Add calendar validation to allocation
// 5. Test time-constrained scenarios
```

---

### 6. SPARQL Query API ⚠️ MEDIUM
**Files**: `rust/knhk-workflow-engine/src/parser/mod.rs`
**Problem**: SPARQL capability exists but not exposed
**Impact**: No programmatic workflow structure queries
**Effort**: 2-3 days

**Implementation Steps**:
```rust
// 1. Add SPARQL query endpoint to REST API
POST /api/v1/workflows/:id/query
{
    "sparql": "SELECT ?task WHERE { ?task rdf:type yawl:Task }"
}

// 2. Expose oxigraph query interface
pub fn query_workflow(&self, sparql: &str) -> Result<QueryResults>;

// 3. Add query validation
// 4. Document SPARQL examples
// 5. Test complex queries
```

---

## Medium Priority (Sprint 5-8, ~15-20 days)

### 7. XPath-Equivalent Navigator ⚠️ LOW-MEDIUM
**Files**: New module `rust/knhk-workflow-engine/src/navigation/`
**Problem**: No path expressions for RDF workflow structures
**Impact**: Difficult to navigate workflow programmatically
**Effort**: 3-4 days

**Design**:
```rust
// RDF path expression syntax
"/workflow/tasks[role='manager']/join_type"
"/workflow/conditions[id='c1']/outgoing_flows"

// Implementation
pub struct WorkflowNavigator {
    spec: WorkflowSpec,
}

impl WorkflowNavigator {
    pub fn query(&self, path: &str) -> Result<Vec<Value>>;
}
```

---

### 8. Workflow Simulation ⚠️ LOW
**Files**: New module `rust/knhk-workflow-engine/src/simulation/`
**Problem**: No what-if analysis capabilities
**Impact**: Cannot test workflows before deployment
**Effort**: 5-7 days

**Design**:
```rust
pub struct WorkflowSimulator {
    engine: WorkflowEngine,
    mode: SimulationMode, // Deterministic, Probabilistic, Stress
}

pub enum SimulationMode {
    Deterministic { steps: usize },
    Probabilistic { distribution: Distribution },
    Stress { concurrent_cases: usize },
}
```

---

## Low Priority (Sprint 9-12, ~30-40 days)

### 9. Web-Based Monitoring Dashboard ⚠️ LOW
**Files**: New project `knhk-dashboard/`
**Problem**: No visual workflow monitoring
**Impact**: Must use CLI/API for monitoring
**Effort**: 10-15 days

**Technology Stack**:
- Frontend: React + TypeScript
- State: Redux + RTK Query
- Visualization: D3.js + react-flow
- Backend: knhk REST API + WebSocket

---

### 10. Graphical Workflow Editor ⚠️ LOW
**Files**: New project `knhk-editor/`
**Problem**: Must write Turtle/RDF manually
**Impact**: Higher learning curve for workflow authors
**Effort**: 15-20 days

**Features**:
- Drag-and-drop workflow builder
- Pattern palette
- Turtle code generation
- Live validation
- Version control integration

---

## Implementation Priority Matrix

| Feature | Effort | Impact | Priority | Sprint |
|---------|--------|--------|----------|--------|
| **MI Execution** | Medium | Critical | 🔴 P0 | 1 |
| **Connector Framework** | High | Critical | 🔴 P0 | 1-2 |
| **gRPC Handlers** | Medium | High | 🟠 P1 | 2 |
| **Worklet Execution** | Medium | Medium | 🟡 P2 | 3 |
| **Resource Calendar** | High | Medium | 🟡 P2 | 4 |
| **SPARQL Query API** | Low | Medium | 🟡 P2 | 3 |
| **XPath Navigator** | Medium | Low-Medium | 🟢 P3 | 5 |
| **Workflow Simulation** | High | Low | 🟢 P3 | 6-7 |
| **Monitoring Dashboard** | Very High | Low | ⚪ P4 | 9-10 |
| **Graphical Editor** | Very High | Low | ⚪ P4 | 11-12 |

---

## Sprint Planning

### Sprint 1 (Week 1-2)
- ✅ MI Execution (3 days)
- ✅ Connector Framework - Part 1 (2 days)

### Sprint 2 (Week 3-4)
- ✅ Connector Framework - Part 2 (3 days)
- ✅ gRPC Handlers (2 days)

### Sprint 3 (Week 5-6)
- ✅ Worklet Execution (4 days)
- ✅ SPARQL Query API (2 days)

### Sprint 4 (Week 7-8)
- ✅ Resource Calendar (5 days)

### Sprint 5-6 (Week 9-12)
- ✅ XPath Navigator (4 days)
- ✅ Workflow Simulation (6 days)

### Sprint 7-12 (Week 13-24)
- ⚠️ Optional: Dashboard + Editor

---

## Success Criteria

### P0 (Critical) - Must Have for 1.0
- [x] Core engine functional
- [ ] MI execution complete (Patterns 12-15)
- [ ] Connector framework operational
- [ ] Automated tasks functional
- [x] REST API complete
- [ ] gRPC API complete

### P1 (High) - Should Have for 1.0
- [x] Resource allocation functional
- [ ] Worklet exception handling complete
- [ ] Resource calendar support
- [x] All 43 patterns fully functional

### P2 (Medium) - Nice to Have for 1.1
- [ ] SPARQL query API
- [ ] XPath-equivalent navigation
- [ ] Workflow simulation

### P3 (Low) - Future Releases
- [ ] Web-based dashboard
- [ ] Graphical editor

---

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| **MI execution complexity** | Medium | High | Start early, allocate buffer time |
| **Connector interface design** | Medium | High | Review YAWL WSIF, design review |
| **Worklet circular dependency** | Low | Medium | Well-understood Rust pattern |
| **Resource calendar scope creep** | High | Medium | Limit to basic shift support v1 |
| **SPARQL security** | Medium | Low | Query validation, rate limiting |

---

## Dependencies

```
Sprint 1 (MI Execution) → Sprint 3 (Worklet Execution)
   └─ MI patterns needed for worklet testing

Sprint 1-2 (Connector Framework) → ALL automated workflows
   └─ Core infrastructure for external integration

Sprint 2 (gRPC Handlers) → Sprint 4+ (Advanced features)
   └─ Some features may prefer gRPC over REST

Sprint 3 (Worklet Execution) → Sprint 4 (Resource Calendar)
   └─ Worklets may need calendar-aware resource allocation
```

---

## Team Assignment Recommendations

### Backend Developer (Connector Framework)
- Design connector interface
- Implement HTTP/gRPC connectors
- Integration testing

### System Architect (MI Execution)
- Design task spawning model
- Architecture review for scalability
- Performance validation

### Production Validator (gRPC + Testing)
- Implement gRPC handlers
- End-to-end testing
- Production readiness validation

### Code Analyzer (Worklet Refactoring)
- Circular dependency resolution
- Code quality review
- Technical debt assessment

---

**Document Version**: 1.0
**Last Updated**: 2025-11-08
**Next Review**: After Sprint 1 completion
