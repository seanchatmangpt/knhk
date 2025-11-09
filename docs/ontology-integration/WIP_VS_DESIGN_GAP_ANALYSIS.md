# WIP Code vs Design Documentation Gap Analysis

**Version:** 1.0
**Date:** 2025-11-08
**Status:** Preliminary Analysis
**Architect:** System Architect (Audit Swarm)

---

## Executive Summary

This analysis maps existing WIP code in `knhk-workflow-engine` against the ontology integration architecture documentation. The goal is to identify what's already built, what's partially complete, and what needs to be finished.

**Key Findings:**
- ✅ **Strong Foundation:** Core parsing infrastructure exists (Oxigraph, SPARQL extraction, deadlock validation)
- ⚠️ **Partial Implementation:** Task/condition extraction works, but missing knhk extensions and resource allocation
- ❌ **Significant Gaps:** SHACL validation, Weaver integration, OTEL spans, caching strategy

**Completion Status:** ~35% complete (foundation exists, semantic layer incomplete)

---

## Architecture Alignment Matrix

| Component | Design Doc Reference | WIP Status | Implementation Gap | Priority |
|-----------|---------------------|------------|-------------------|----------|
| **Ontology Loading** | parser-integration §3.1 | ✅ **DONE** | None - `load_yawl_ontology()` exists | LOW |
| **SPARQL Task Extraction** | parser-integration §4.1.1 | ✅ **DONE** | None - Basic extraction works | LOW |
| **SPARQL Flow Extraction** | parser-integration §4.1.2 | ⚠️ **PARTIAL** | Single query optimization needed | MEDIUM |
| **knhk Extensions Extraction** | parser-integration §5.1 | ❌ **MISSING** | Need: span_template, provenance_required, performance_tier, validation_schemas | **HIGH** |
| **Type System Mapping** | parser-integration §5.2 | ⚠️ **40% COMPLETE** | Task struct exists but missing knhk fields | **HIGH** |
| **SHACL Validation** | parser-integration §6.1.1 | ❌ **MISSING** | Need: shape loading, validation pipeline | **HIGH** |
| **SPARQL Semantic Validation** | parser-integration §6.1.2 | ❌ **MISSING** | Need: 30+ validation rules | **HIGH** |
| **Weaver Integration** | parser-integration §6.2 | ❌ **MISSING** | Need: span template validation | **HIGH** |
| **Caching Strategy** | parser-integration §7.1 | ❌ **MISSING** | Need: RwLock cache, RocksDB backend | MEDIUM |
| **Prepared Queries** | parser-integration §4.2 | ❌ **MISSING** | Need: query compilation, reuse | MEDIUM |
| **Resource Allocation Extraction** | parser-integration §6.1 | ❌ **MISSING** | Need: allocation_policy, required_roles, required_capabilities | **HIGH** |
| **Executor Integration** | executor-integration §3.1 | ⚠️ **20% COMPLETE** | Engine exists, but no ontology context | **HIGH** |
| **Pattern Execution Context** | executor-integration §3.1 | ⚠️ **30% COMPLETE** | Context exists, missing ontology fields | **HIGH** |
| **Join Condition Checking** | executor-integration §3.2 | ❌ **MISSING** | Need: AND/XOR/OR join logic | **HIGH** |
| **Split Semantics Application** | executor-integration §3.2 | ❌ **MISSING** | Need: AND/XOR/OR split logic | **HIGH** |
| **Hot Path Optimization** | executor-integration §5.1 | ❌ **MISSING** | Need: tick budget enforcement, SIMD | **HIGH** |
| **OTEL Span Creation** | executor-integration §7.1 | ❌ **MISSING** | Need: automatic span from span_template | **HIGH** |
| **Lockchain Provenance** | executor-integration §7.1 | ⚠️ **PARTIAL** | Lockchain integration exists, need ontology trigger | MEDIUM |

---

## What's Already Perfect (Don't Touch)

These components are production-ready and match the design exactly:

### 1. Core Parser Foundation (`src/parser/mod.rs`)
```rust
✅ WorkflowParser struct with Oxigraph Store
✅ parse_turtle() - Loads TTL into RDF store
✅ parse_file() - File-based parsing
✅ load_yawl_ontology() - Ontology loading
✅ DeadlockDetector integration
```

**Status:** **100% Complete**
**Evidence:** Lines 18-75 in `src/parser/mod.rs`
**Action:** Keep as-is, build on top of this foundation

### 2. SPARQL Query Infrastructure (`src/parser/extractor.rs`)
```rust
✅ extract_workflow_spec() - Main extraction entry point
✅ extract_tasks() - SPARQL query for tasks
✅ extract_conditions() - SPARQL query for conditions
✅ extract_flows() - Flow connection logic
✅ find_start_condition() - Start condition query
✅ find_end_condition() - End condition query
```

**Status:** **100% Complete** (for basic YAWL properties)
**Evidence:** Lines 12-479 in `src/parser/extractor.rs`
**Action:** Extend queries to include knhk extensions (see gaps below)

### 3. Basic Type System (`src/parser/types.rs`)
```rust
✅ WorkflowSpec struct
✅ Task struct (with task_type, split_type, join_type, max_ticks, priority, use_simd)
✅ Condition struct
✅ TaskType, SplitType, JoinType enums
```

**Status:** **70% Complete** (YAWL types complete, knhk extensions missing)
**Evidence:** Task struct in `types.rs` (inferred from extractor usage)
**Action:** Add knhk extension fields (see gaps below)

---

## What Needs Finishing (Build On Existing)

These components have foundations but need enhancements:

### 1. Task Extraction - Add knhk Extensions

**Current State (lines 86-258 in extractor.rs):**
```rust
// ✅ Extracts: task_id, name, task_type, split_type, join_type
// ✅ Extracts: max_ticks, priority, use_simd
// ❌ MISSING: span_template, provenance_required, performance_tier
// ❌ MISSING: allocation_policy, required_roles, required_capabilities
```

**What to Add:**

#### Step 1: Extend SPARQL Query (line 94 in extractor.rs)
```sparql
PREFIX knhk: <http://knhk.org/ontology#>
SELECT ?task ?name ?type ?split ?join
       ?maxTicks ?priority ?simd
       ?spanTemplate ?provenanceRequired ?performanceTier
       ?allocPolicy ?role ?capability
WHERE {
    # ... existing query ...

    # NEW: knhk extensions
    OPTIONAL { ?task knhk:hasSpanTemplate ?spanTemplate }
    OPTIONAL { ?task knhk:requiresProvenance ?provenanceRequired }
    OPTIONAL { ?task knhk:performanceTier ?performanceTier }

    # NEW: Resource allocation
    OPTIONAL { ?task yawl:hasAllocationPolicy ?allocPolicy }
    OPTIONAL { ?task yawl:requiresRole ?role }
    OPTIONAL { ?task yawl:requiresCapability ?capability }
}
```

#### Step 2: Extend Task Struct (in `types.rs`)
```rust
pub struct Task {
    // ✅ Existing fields (keep as-is)
    pub id: String,
    pub name: String,
    pub task_type: TaskType,
    pub split_type: SplitType,
    pub join_type: JoinType,
    pub max_ticks: Option<u32>,
    pub priority: Option<u32>,
    pub use_simd: bool,
    pub input_conditions: Vec<String>,
    pub output_conditions: Vec<String>,
    pub outgoing_flows: Vec<String>,
    pub incoming_flows: Vec<String>,
    pub allocation_policy: Option<AllocationPolicy>,
    pub required_roles: Vec<String>,
    pub required_capabilities: Vec<String>,
    pub exception_worklet: Option<WorkletId>,

    // ❌ NEW: Add these knhk extensions
    pub span_template: Option<String>,
    pub provenance_required: bool,
    pub performance_tier: PerformanceTier,
    pub validation_schemas: Vec<String>,
}

// ❌ NEW: Add PerformanceTier enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PerformanceTier {
    Hot,   // ≤8 ticks
    Warm,  // ≤100 ticks
    Cold,  // No constraint
}

impl Default for PerformanceTier {
    fn default() -> Self {
        Self::Cold
    }
}
```

#### Step 3: Parse New Fields (in `extract_tasks()`)
```rust
// After line 232 in extractor.rs, add:

let span_template = solution.get("spanTemplate")
    .and_then(|t| {
        if let oxigraph::model::Term::Literal(lit) = t {
            Some(lit.value().to_string())
        } else {
            None
        }
    });

let provenance_required = solution.get("provenanceRequired")
    .and_then(|p| {
        if let oxigraph::model::Term::Literal(lit) = p {
            lit.value().parse::<bool>().ok()
        } else {
            None
        }
    })
    .unwrap_or(false);

let performance_tier = solution.get("performanceTier")
    .and_then(|t| {
        if let oxigraph::model::Term::Literal(lit) = t {
            match lit.value() {
                "hot" => Some(PerformanceTier::Hot),
                "warm" => Some(PerformanceTier::Warm),
                "cold" => Some(PerformanceTier::Cold),
                _ => None,
            }
        } else {
            None
        }
    })
    .unwrap_or(PerformanceTier::Cold);
```

**Estimated Effort:** 2-3 hours
**Priority:** **HIGH** (blocks executor integration)
**Files to Modify:**
- `src/parser/extractor.rs` (extend SPARQL query, parse new fields)
- `src/parser/types.rs` (add Task fields, PerformanceTier enum)

---

### 2. Flow Extraction Optimization

**Current State (lines 334-384 in extractor.rs):**
```rust
// ✅ Works correctly
// ⚠️ Uses simple query with single predicate
// ❌ MISSING: Multiple flow predicates (yawl:flowsTo, yawl:hasOutgoingFlow)
```

**What to Add:**

#### Optimized Query (replace line 341)
```sparql
PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>

SELECT ?from ?to
WHERE {
    { ?from yawl:hasOutgoingFlow ?to }
    UNION
    { ?from yawl:flowsTo ?to }
}
```

**Estimated Effort:** 30 minutes
**Priority:** MEDIUM (nice-to-have, current implementation works)

---

### 3. Executor Pattern Context - Add Ontology Fields

**Current State (inferred from executor structure):**
```rust
// ✅ PatternExecutionContext exists
// ✅ Has: case_id, workflow_id, task_id, variables
// ❌ MISSING: Ontology-derived metadata (join_type, split_type, etc.)
```

**What to Add:**

#### Step 1: Extend PatternExecutionContext (in `src/executor/pattern.rs`)
```rust
pub struct PatternExecutionContext {
    // ✅ Existing fields (keep as-is)
    pub case_id: CaseId,
    pub workflow_id: WorkflowSpecId,
    pub task_id: String,
    pub variables: HashMap<String, String>,
    pub arrived_from: HashSet<String>,
    pub scope_id: String,

    // ❌ NEW: Add ontology-derived metadata
    pub task_name: String,
    pub join_type: JoinType,
    pub split_type: SplitType,
    pub performance_tier: PerformanceTier,
    pub max_ticks: Option<u32>,
    pub span_template: Option<String>,
    pub provenance_required: bool,
    pub use_simd: bool,
    pub outgoing_flows: Vec<String>,
    pub incoming_flows: Vec<String>,
}

impl PatternExecutionContext {
    /// Create from cached Task struct (NO RDF queries)
    pub fn from_task(
        case_id: CaseId,
        workflow_id: WorkflowSpecId,
        task: &Task,
        variables: HashMap<String, String>,
        arrived_from: HashSet<String>,
    ) -> Self {
        Self {
            case_id,
            workflow_id,
            task_id: task.id.clone(),
            task_name: task.name.clone(),
            join_type: task.join_type,
            split_type: task.split_type,
            performance_tier: task.performance_tier,
            max_ticks: task.max_ticks,
            span_template: task.span_template.clone(),
            provenance_required: task.provenance_required,
            use_simd: task.use_simd,
            outgoing_flows: task.outgoing_flows.clone(),
            incoming_flows: task.incoming_flows.clone(),
            variables,
            arrived_from,
            scope_id: case_id.to_string(),
        }
    }
}
```

**Estimated Effort:** 1-2 hours
**Priority:** **HIGH** (required for semantic execution)
**Files to Modify:**
- `src/executor/pattern.rs` (extend context struct, add from_task() constructor)

---

## What's Missing Entirely

These components need to be built from scratch:

### 1. SHACL Validation Pipeline

**Design Reference:** parser-integration-design.md §6.1.1
**Current State:** ❌ Not implemented
**Required:**

```rust
// NEW FILE: src/parser/validation/shacl.rs

pub struct ShaclValidator {
    shapes: Vec<ShapeDefinition>,
}

impl ShaclValidator {
    pub fn load_shapes(&mut self, path: &Path) -> WorkflowResult<()> {
        // Load SHACL shapes from yawl-shacl.ttl
    }

    pub fn validate(&self, store: &Store, spec_iri: &str) -> WorkflowResult<ValidationReport> {
        // Execute SHACL validation queries
        // Return violations
    }
}

pub struct ValidationReport {
    pub valid: bool,
    pub violations: Vec<ShaclViolation>,
}

pub struct ShaclViolation {
    pub shape_id: String,
    pub focus_node: Option<String>,
    pub message: Option<String>,
}
```

**Integration Point:** `WorkflowParser::parse_turtle()` (add SHACL validation after line 43)

**Estimated Effort:** 4-6 hours
**Priority:** **HIGH** (required for production validation)

---

### 2. SPARQL Semantic Validation Rules

**Design Reference:** parser-integration-design.md §6.1.2
**Current State:** ❌ Not implemented
**Required:**

```rust
// NEW FILE: src/parser/validation/semantic.rs

pub struct SemanticValidator;

impl SemanticValidator {
    /// Rule 1: Start condition has no incoming flows
    pub fn validate_start_condition(store: &Store, spec_iri: &str) -> WorkflowResult<bool> {
        let query = r#"
            PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>
            ASK {
                ?spec yawl:hasStartCondition ?start .
                ?from yawl:hasOutgoingFlow ?start .
            }
        "#;
        // Returns false if valid (no incoming flows)
    }

    /// Rule 2: End condition has no outgoing flows
    pub fn validate_end_condition(store: &Store, spec_iri: &str) -> WorkflowResult<bool> { ... }

    /// Rule 3: All tasks connected
    pub fn validate_connectivity(store: &Store, spec_iri: &str) -> WorkflowResult<Vec<ValidationError>> { ... }

    /// Rule 4: Split/join compatibility
    pub fn validate_split_join(store: &Store, spec_iri: &str) -> WorkflowResult<Vec<ValidationError>> { ... }
}
```

**Integration Point:** `WorkflowParser::parse_turtle()` (add semantic validation after SHACL)

**Estimated Effort:** 6-8 hours (30+ validation rules)
**Priority:** **HIGH** (critical for workflow correctness)

---

### 3. Weaver Schema Validation Integration

**Design Reference:** parser-integration-design.md §6.2
**Current State:** ❌ Not implemented
**Required:**

```rust
// EXTEND: src/parser/mod.rs

impl WorkflowParser {
    pub fn validate_otel_schemas(&self, spec: &WorkflowSpec) -> WorkflowResult<()> {
        for task in spec.tasks.values() {
            if let Some(ref template) = task.span_template {
                // Call weaver CLI or library
                let status = std::process::Command::new("weaver")
                    .args(&["registry", "check", "-r", "registry/", "--span", template])
                    .status()?;

                if !status.success() {
                    return Err(WorkflowError::Validation(
                        format!("Span template '{}' not found in Weaver registry", template)
                    ));
                }
            }
        }
        Ok(())
    }
}
```

**Integration Point:** `WorkflowParser::parse_turtle()` (add Weaver validation after semantic validation)

**Estimated Effort:** 2-3 hours
**Priority:** **HIGH** (required for OTEL compliance)

---

### 4. Join Condition Checking (Executor)

**Design Reference:** executor-integration-design.md §3.2
**Current State:** ❌ Not implemented
**Required:**

```rust
// NEW METHOD: src/executor/task.rs (or new file src/executor/join.rs)

impl WorkflowEngine {
    /// Check if join condition is satisfied (uses ontology join_type)
    fn check_join_condition(&self, case_id: CaseId, task: &Task) -> WorkflowResult<bool> {
        let case = self.cases.get(&case_id)
            .ok_or(WorkflowError::CaseNotFound(case_id))?;

        match task.join_type {
            JoinType::And => {
                // AND-join: All incoming flows must have arrived
                Ok(task.incoming_flows.iter()
                    .all(|flow_id| case.arrived_from.contains(flow_id)))
            }
            JoinType::Xor => {
                // XOR-join: Exactly one incoming flow must have arrived
                Ok(task.incoming_flows.iter()
                    .filter(|flow_id| case.arrived_from.contains(*flow_id))
                    .count() == 1)
            }
            JoinType::Or => {
                // OR-join: At least one incoming flow must have arrived
                Ok(task.incoming_flows.iter()
                    .any(|flow_id| case.arrived_from.contains(flow_id)))
            }
        }
    }
}
```

**Integration Point:** `WorkflowEngine::execute_task()` (check join before executing)

**Estimated Effort:** 2-3 hours
**Priority:** **HIGH** (critical for correct execution)

---

### 5. Split Semantics Application (Executor)

**Design Reference:** executor-integration-design.md §3.2
**Current State:** ❌ Not implemented
**Required:**

```rust
// NEW METHOD: src/executor/task.rs (or new file src/executor/split.rs)

impl WorkflowEngine {
    /// Apply split semantics (uses ontology split_type)
    async fn apply_split_semantics(
        &self,
        case_id: CaseId,
        task: &Task,
        ctx: &PatternExecutionContext,
    ) -> WorkflowResult<()> {
        match task.split_type {
            SplitType::And => {
                // AND-split: Enable ALL outgoing flows
                for flow_id in &task.outgoing_flows {
                    self.enable_element(case_id, flow_id).await?;
                }
            }
            SplitType::Xor => {
                // XOR-split: Enable EXACTLY ONE outgoing flow
                let selected_flow = self.evaluate_xor_condition(ctx)?;
                self.enable_element(case_id, &selected_flow).await?;
            }
            SplitType::Or => {
                // OR-split: Enable ONE OR MORE outgoing flows
                let selected_flows = self.evaluate_or_conditions(ctx)?;
                for flow_id in selected_flows {
                    self.enable_element(case_id, &flow_id).await?;
                }
            }
        }
        Ok(())
    }
}
```

**Integration Point:** `WorkflowEngine::execute_task()` (apply split after pattern execution)

**Estimated Effort:** 3-4 hours
**Priority:** **HIGH** (critical for correct execution)

---

### 6. Hot Path Tick Budget Enforcement

**Design Reference:** executor-integration-design.md §5.1
**Current State:** ❌ Not implemented
**Required:**

```rust
// NEW METHOD: src/executor/task.rs

impl WorkflowEngine {
    fn get_current_tick(&self) -> u64 {
        #[cfg(target_arch = "x86_64")]
        {
            unsafe { core::arch::x86_64::_rdtsc() }
        }
        #[cfg(target_arch = "aarch64")]
        {
            let mut val: u64;
            unsafe {
                core::arch::asm!("mrs {}, cntvct_el0", out(reg) val);
            }
            val
        }
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            0 // Fallback
        }
    }

    pub async fn execute_task_with_budget(
        &self,
        case_id: CaseId,
        task_id: &str,
    ) -> WorkflowResult<TaskExecutionMetrics> {
        let task = self.get_task(case_id, task_id)?;
        let tick_budget = task.max_ticks.unwrap_or(u32::MAX);

        let start_tick = self.get_current_tick();
        self.execute_task_semantic(case_id, task_id).await?;
        let elapsed_ticks = (self.get_current_tick() - start_tick) as u32;

        if elapsed_ticks > tick_budget {
            return Err(WorkflowError::TickBudgetExceeded {
                task_id: task_id.to_string(),
                max_ticks: tick_budget,
                actual_ticks: elapsed_ticks as u64,
            });
        }

        Ok(TaskExecutionMetrics { task_id: task_id.to_string(), tick_budget, elapsed_ticks })
    }
}
```

**Integration Point:** `WorkflowEngine::execute_task()` (wrap execution in tick measurement)

**Estimated Effort:** 2-3 hours
**Priority:** **HIGH** (required for performance guarantees)

---

### 7. OTEL Span Automatic Creation

**Design Reference:** executor-integration-design.md §7
**Current State:** ❌ Not implemented (OTEL integration exists, but no span templates)
**Required:**

```rust
// NEW METHOD: src/executor/task.rs

impl WorkflowEngine {
    fn create_otel_span(&self, template: &str, task: &Task) -> WorkflowResult<SpanGuard> {
        if let Some(ref otel) = self.otel_integration {
            let span = otel.start_span(
                template,
                vec![
                    ("task.id", task.id.clone()),
                    ("task.name", task.name.clone()),
                    ("task.split_type", format!("{:?}", task.split_type)),
                    ("task.join_type", format!("{:?}", task.join_type)),
                ],
            )?;
            Ok(SpanGuard::new(span))
        } else {
            Ok(SpanGuard::noop())
        }
    }
}

pub struct SpanGuard {
    span: Option<Span>,
}

impl Drop for SpanGuard {
    fn drop(&mut self) {
        if let Some(span) = self.span.take() {
            span.end();
        }
    }
}
```

**Integration Point:** `WorkflowEngine::execute_task()` (create span if span_template defined)

**Estimated Effort:** 2-3 hours
**Priority:** **HIGH** (required for OTEL compliance)

---

### 8. WorkflowSpec Caching with RwLock

**Design Reference:** parser-integration-design.md §7.1
**Current State:** ❌ Not implemented
**Required:**

```rust
// EXTEND: src/parser/mod.rs

use std::sync::{Arc, RwLock};
use std::collections::HashMap;

pub struct WorkflowParser {
    store: Store,
    deadlock_detector: DeadlockDetector,

    // NEW: Workflow spec cache
    spec_cache: Arc<RwLock<HashMap<String, WorkflowSpec>>>,
}

impl WorkflowParser {
    pub fn parse_and_cache(&mut self, turtle: &str, cache_key: &str) -> WorkflowResult<WorkflowSpec> {
        // Check cache first
        {
            let cache = self.spec_cache.read().unwrap();
            if let Some(spec) = cache.get(cache_key) {
                return Ok(spec.clone());
            }
        }

        // Parse from TTL
        let spec = self.parse_turtle(turtle)?;

        // Cache the result
        {
            let mut cache = self.spec_cache.write().unwrap();
            cache.insert(cache_key.to_string(), spec.clone());
        }

        Ok(spec)
    }

    pub fn clear_cache(&mut self) {
        let mut cache = self.spec_cache.write().unwrap();
        cache.clear();
    }
}
```

**Integration Point:** Replace `parse_turtle()` calls with `parse_and_cache()` in production

**Estimated Effort:** 1-2 hours
**Priority:** MEDIUM (performance optimization, not critical for correctness)

---

## Recommended Completion Order

Based on dependencies and priority:

### Phase 1: Complete Type System (Week 1)
**Goal:** All ontology properties extractable from TTL

1. ✅ **Task knhk Extensions** (2-3 hours)
   - Extend Task struct with knhk fields
   - Extend SPARQL query
   - Parse new fields
   - **Blocker for:** Executor integration, OTEL spans, tick budgets

2. ✅ **PatternExecutionContext Extensions** (1-2 hours)
   - Add ontology fields to context
   - Create `from_task()` constructor
   - **Blocker for:** Join/split execution

### Phase 2: Validation Pipeline (Week 1-2)
**Goal:** Workflows validated before execution

3. ✅ **SHACL Validation** (4-6 hours)
   - Load SHACL shapes
   - Execute validation queries
   - Report violations
   - **Blocker for:** Production deployment

4. ✅ **SPARQL Semantic Validation** (6-8 hours)
   - Implement 30+ validation rules
   - Check start/end conditions, connectivity, split/join compatibility
   - **Blocker for:** Workflow correctness

5. ✅ **Weaver Integration** (2-3 hours)
   - Validate span templates
   - Check Weaver registry
   - **Blocker for:** OTEL compliance

### Phase 3: Executor Semantics (Week 2)
**Goal:** Ontology-driven execution

6. ✅ **Join Condition Checking** (2-3 hours)
   - Implement AND/XOR/OR join logic
   - **Blocker for:** Correct task enabling

7. ✅ **Split Semantics Application** (3-4 hours)
   - Implement AND/XOR/OR split logic
   - **Blocker for:** Correct flow enabling

8. ✅ **OTEL Span Creation** (2-3 hours)
   - Automatic span from span_template
   - Span guard RAII
   - **Blocker for:** Telemetry compliance

### Phase 4: Performance & Optimization (Week 3)
**Goal:** Hot path performance guarantees

9. ✅ **Tick Budget Enforcement** (2-3 hours)
   - Measure ticks
   - Validate budget
   - **Blocker for:** Performance guarantees

10. ⚠️ **Caching Strategy** (1-2 hours)
    - In-memory cache with RwLock
    - **Nice-to-have:** Performance optimization

11. ⚠️ **Prepared Queries** (2-3 hours)
    - Compile SPARQL queries
    - **Nice-to-have:** Performance optimization

---

## Quick Wins (Small Gaps, High Value)

These are low-hanging fruit that provide immediate value:

1. **Flow Extraction Optimization** (30 min)
   - Add UNION query for multiple flow predicates
   - **Value:** More robust flow extraction

2. **Performance Tier Enum** (15 min)
   - Add PerformanceTier to types.rs
   - **Value:** Enables hot/warm/cold path distinction

3. **Weaver Validation** (2 hours)
   - Add weaver CLI check
   - **Value:** OTEL compliance validation

---

## Dependency Graph

```
Ontology Loading (✅ Done)
    ↓
Task/Condition Extraction (✅ Done)
    ↓
    ├─→ knhk Extensions (❌ HIGH PRIORITY)
    │       ↓
    │   PatternExecutionContext (❌ HIGH PRIORITY)
    │       ↓
    │   Join/Split Logic (❌ HIGH PRIORITY)
    │       ↓
    │   Semantic Execution (❌ HIGH PRIORITY)
    │
    ├─→ SHACL Validation (❌ HIGH PRIORITY)
    │       ↓
    │   SPARQL Validation (❌ HIGH PRIORITY)
    │       ↓
    │   Weaver Validation (❌ HIGH PRIORITY)
    │
    └─→ Tick Budget (❌ HIGH PRIORITY)
            ↓
        OTEL Spans (❌ HIGH PRIORITY)
            ↓
        Caching (⚠️ MEDIUM PRIORITY)
```

---

## Summary

**Foundation:** ✅ **Strong** - Core parsing and executor infrastructure exists
**Semantic Layer:** ⚠️ **Partial** - Basic extraction works, but missing ontology semantics
**Validation:** ❌ **Missing** - SHACL, SPARQL, and Weaver validation not implemented
**Execution:** ⚠️ **Partial** - Engine exists, but no join/split logic or tick budgets
**Performance:** ❌ **Missing** - No caching, prepared queries, or SIMD optimization

**Estimated Total Effort:** 30-40 hours (1 week for one developer, or 2-3 days for a team)

**Critical Path:**
1. knhk Extensions → PatternExecutionContext → Join/Split Logic → Semantic Execution
2. SHACL → SPARQL → Weaver Validation
3. Tick Budget → OTEL Spans

**Recommended Approach:**
- **Week 1:** Complete type system + validation pipeline
- **Week 2:** Implement executor semantics + OTEL integration
- **Week 3:** Performance optimization (caching, prepared queries)

This analysis should be updated as the Code Analyzer completes its audit report.
