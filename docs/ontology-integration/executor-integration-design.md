# Executor Integration Design: Ontology-Derived Runtime Execution

**Version:** 1.0
**Date:** 2025-11-08
**Status:** Integration Design
**Target:** knhk-workflow-engine v2.0
**Component:** `src/executor/mod.rs`, `src/executor/pattern.rs`, `src/executor/task.rs`

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Current Executor Architecture](#current-executor-architecture)
3. [Ontology-Derived Execution Model](#ontology-derived-execution-model)
4. [Pattern Execution with Ontology Semantics](#pattern-execution-with-ontology-semantics)
5. [Hot Path Optimization](#hot-path-optimization)
6. [Resource Allocation from Ontology](#resource-allocation-from-ontology)
7. [OTEL Integration via Span Templates](#otel-integration-via-span-templates)
8. [State Transition with Ontology Semantics](#state-transition-with-ontology-semantics)
9. [Performance Validation](#performance-validation)
10. [Implementation Examples](#implementation-examples)

---

## 1. Executive Summary

This document defines the **detailed integration architecture** between ontology-derived workflow specifications and the runtime `WorkflowEngine`. The design ensures:

1. **No RDF Queries in Hot Path:** All ontology data pre-loaded into Rust structs during parsing
2. **≤8 Tick Compliance:** Hot path operations meet Chatman Constant performance constraint
3. **Semantic Execution:** Pattern executors leverage ontology-defined split/join semantics
4. **OTEL Integration:** Automatic span creation from `knhk:hasSpanTemplate` properties
5. **Provenance Tracking:** Lockchain integration for `knhk:requiresProvenance` tasks

**Critical Principle:** The executor uses **cached Rust structs** derived from ontology, NOT runtime RDF queries.

---

## 2. Current Executor Architecture

### 2.1 WorkflowEngine Structure

**File:** `src/executor/engine.rs`

```rust
pub struct WorkflowEngine {
    // Pattern execution
    pub(crate) pattern_registry: Arc<PatternRegistry>,

    // State management
    pub(crate) state_store: Arc<RwLock<Arc<StateStore>>>,
    pub(crate) state_manager: Arc<StateManager>,

    // Workflow storage
    pub(crate) specs: Arc<DashMap<WorkflowSpecId, WorkflowSpec>>,
    pub(crate) cases: Arc<DashMap<CaseId, Case>>,

    // Resource management
    pub(crate) resource_allocator: Arc<ResourceAllocator>,

    // Worklets (exception handling)
    pub(crate) worklet_repository: Arc<WorkletRepository>,
    pub(crate) worklet_executor: Arc<WorkletExecutor>,

    // Services
    pub(crate) timer_service: Arc<TimerService<SysClock>>,
    pub(crate) work_item_service: Arc<WorkItemService>,
    pub(crate) admission_gate: Arc<AdmissionGate>,
    pub(crate) event_sidecar: Arc<EventSidecar>,

    // Enterprise features
    pub(crate) enterprise_config: Option<Arc<EnterpriseConfig>>,
    pub(crate) fortune5_integration: Option<Arc<Fortune5Integration>>,
    pub(crate) otel_integration: Option<Arc<OtelIntegration>>,
    pub(crate) lockchain_integration: Option<Arc<LockchainIntegration>>,
    pub(crate) auth_manager: Option<Arc<RwLock<AuthManager>>>,
    pub(crate) provenance_tracker: Option<Arc<ProvenanceTracker>>,
    pub(crate) sidecar_integration: Option<Arc<SidecarIntegration>>,
    pub(crate) connector_integration: Option<Arc<tokio::sync::Mutex<ConnectorIntegration>>>,
}
```

### 2.2 Current Execution Flow

```
┌─────────────────────────────────────────────────────┐
│  engine.start_case(case_id)                         │
└─────────────────┬───────────────────────────────────┘
                  │
                  │ 1. Load WorkflowSpec from specs DashMap
                  ▼
┌─────────────────────────────────────────────────────┐
│  WorkflowSpec (cached Rust struct)                  │
│  ├─ tasks: HashMap<String, Task>                    │
│  ├─ conditions: HashMap<String, Condition>          │
│  └─ start_condition: Option<String>                 │
└─────────────────┬───────────────────────────────────┘
                  │
                  │ 2. Enable start condition
                  ▼
┌─────────────────────────────────────────────────────┐
│  Enable condition → Find outgoing tasks             │
└─────────────────┬───────────────────────────────────┘
                  │
                  │ 3. For each enabled task
                  ▼
┌─────────────────────────────────────────────────────┐
│  execute_task_with_pattern(task_id)                 │
│  ├─ Allocate resources                              │
│  ├─ Check join condition (AND/XOR/OR)               │
│  ├─ Execute pattern                                 │
│  └─ Apply split semantics (AND/XOR/OR)              │
└─────────────────┬───────────────────────────────────┘
                  │
                  │ 4. Update state, enable next conditions
                  ▼
┌─────────────────────────────────────────────────────┐
│  state_manager.save_case(case)                      │
└─────────────────────────────────────────────────────┘
```

### 2.3 Pattern Execution Context

**File:** `src/executor/pattern.rs` (inferred structure)

```rust
pub struct PatternExecutionContext {
    pub case_id: CaseId,
    pub workflow_id: WorkflowSpecId,
    pub task_id: String,
    pub variables: HashMap<String, String>,
    pub arrived_from: HashSet<String>,
    pub scope_id: String,
}

impl WorkflowEngine {
    fn execute_pattern(&self, task: &Task, ctx: &PatternExecutionContext) -> WorkflowResult<()> {
        // Determine pattern from task split/join types
        let pattern_id = self.derive_pattern_id(task);

        // Execute pattern
        let result = self.pattern_registry.execute(&pattern_id, ctx)?;

        // Apply result to workflow state
        self.apply_pattern_result(task, result)?;

        Ok(())
    }
}
```

---

## 3. Ontology-Derived Execution Model

### 3.1 Enhanced Execution Context

**Key Enhancement:** Add ontology-derived metadata to execution context WITHOUT referencing RDF store.

```rust
/// Enhanced pattern execution context with ontology semantics
pub struct PatternExecutionContext {
    // Existing fields
    pub case_id: CaseId,
    pub workflow_id: WorkflowSpecId,
    pub task_id: String,
    pub variables: HashMap<String, String>,
    pub arrived_from: HashSet<String>,
    pub scope_id: String,

    // NEW: Ontology-derived metadata (from WorkflowSpec.tasks)
    /// Task name (from rdfs:label)
    pub task_name: String,

    /// Join type (from yawl:joinType) - determines when task can execute
    pub join_type: JoinType,

    /// Split type (from yawl:splitType) - determines how task spawns children
    pub split_type: SplitType,

    /// Performance tier (from knhk:performanceTier)
    pub performance_tier: PerformanceTier,

    /// Max ticks allowed (from knhk:tickBudget)
    pub max_ticks: Option<u32>,

    /// OTEL span template (from knhk:hasSpanTemplate)
    pub span_template: Option<String>,

    /// Provenance required (from knhk:requiresProvenance)
    pub provenance_required: bool,

    /// SIMD optimization enabled (from knhk:useSimd)
    pub use_simd: bool,

    /// Pre-computed outgoing flows (from yawl:hasOutgoingFlow)
    pub outgoing_flows: Vec<String>,

    /// Pre-computed incoming flows (cached during parsing)
    pub incoming_flows: Vec<String>,

    // Resource allocation (from yawl:hasAllocationPolicy)
    pub allocation_policy: Option<AllocationPolicy>,
    pub required_roles: Vec<String>,
    pub required_capabilities: Vec<String>,
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
            allocation_policy: task.allocation_policy.clone(),
            required_roles: task.required_roles.clone(),
            required_capabilities: task.required_capabilities.clone(),
            variables,
            arrived_from,
            scope_id: case_id.to_string(),
        }
    }
}
```

### 3.2 Task Execution with Ontology Semantics

**File:** `src/executor/task.rs`

```rust
impl WorkflowEngine {
    /// Execute task with ontology-derived semantics
    pub async fn execute_task_semantic(
        &self,
        case_id: CaseId,
        task_id: &str,
    ) -> WorkflowResult<()> {
        // Step 1: Get case and spec (from cache, NO RDF)
        let case = self.cases.get(&case_id)
            .ok_or(WorkflowError::CaseNotFound(case_id))?;
        let spec = self.specs.get(&case.workflow_id)
            .ok_or(WorkflowError::SpecNotFound(case.workflow_id))?;

        // Step 2: Get task (from cached WorkflowSpec)
        let task = spec.tasks.get(task_id)
            .ok_or(WorkflowError::TaskNotFound(task_id.to_string()))?;

        // Step 3: Check join condition (ontology-derived join_type)
        if !self.check_join_condition(case_id, task)? {
            return Ok(()); // Join not satisfied, task not enabled
        }

        // Step 4: Allocate resources (ontology-derived allocation_policy)
        if let Some(ref policy) = task.allocation_policy {
            self.allocate_resources_for_task(case_id, task, policy).await?;
        }

        // Step 5: Create OTEL span (if span_template defined)
        let _span_guard = if let Some(ref template) = task.span_template {
            Some(self.create_otel_span(template, task)?)
        } else {
            None
        };

        // Step 6: Start Lockchain tracking (if provenance_required)
        let _lockchain_guard = if task.provenance_required {
            Some(self.start_lockchain_tracking(case_id, task_id)?)
        } else {
            None
        };

        // Step 7: Create execution context (NO RDF, all from cached task)
        let ctx = PatternExecutionContext::from_task(
            case_id,
            case.workflow_id,
            task,
            case.variables.clone(),
            case.arrived_from.clone(),
        );

        // Step 8: Execute pattern (split/join semantics from ontology)
        self.execute_pattern_with_semantics(task, &ctx).await?;

        // Step 9: Apply split semantics (ontology-derived split_type)
        self.apply_split_semantics(case_id, task, &ctx).await?;

        Ok(())
    }

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
                // XOR-split: Enable EXACTLY ONE outgoing flow (based on condition)
                let selected_flow = self.evaluate_xor_condition(ctx)?;
                self.enable_element(case_id, &selected_flow).await?;
            }
            SplitType::Or => {
                // OR-split: Enable ONE OR MORE outgoing flows (based on conditions)
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

---

## 4. Pattern Execution with Ontology Semantics

### 4.1 Pattern Registry Integration

**File:** `src/patterns/mod.rs`

The `PatternRegistry` executes Van der Aalst's 43 patterns. With ontology integration, patterns can leverage semantic metadata:

```rust
impl PatternRegistry {
    /// Execute pattern with ontology-derived context
    pub fn execute_with_semantics(
        &self,
        pattern_id: &str,
        ctx: &PatternExecutionContext,
    ) -> WorkflowResult<PatternExecutionResult> {
        // Hot path optimization: Check max_ticks budget
        let start_tick = if ctx.max_ticks.is_some() {
            Some(self.get_current_tick())
        } else {
            None
        };

        // SIMD optimization (if enabled in ontology)
        if ctx.use_simd {
            self.execute_simd(pattern_id, ctx)?;
        } else {
            self.execute_standard(pattern_id, ctx)?;
        }

        // Validate tick budget (if hot path task)
        if let (Some(max), Some(start)) = (ctx.max_ticks, start_tick) {
            let elapsed = self.get_current_tick() - start;
            if elapsed > max as u64 {
                return Err(WorkflowError::TickBudgetExceeded {
                    task_id: ctx.task_id.clone(),
                    max_ticks: max,
                    actual_ticks: elapsed,
                });
            }
        }

        Ok(PatternExecutionResult::Success)
    }
}
```

### 4.2 Pattern-Specific Ontology Integration

**Example: Parallel Split (Pattern 2)**

```rust
/// Pattern 2: Parallel Split (AND-split)
fn execute_parallel_split(ctx: &PatternExecutionContext) -> WorkflowResult<()> {
    // Ontology check: Verify split_type is And
    if ctx.split_type != SplitType::And {
        return Err(WorkflowError::PatternMismatch {
            expected: "Parallel Split (AND)",
            actual: format!("{:?}", ctx.split_type),
        });
    }

    // Execute all outgoing flows (derived from ontology)
    for flow_id in &ctx.outgoing_flows {
        enable_flow(ctx.case_id, flow_id)?;
    }

    Ok(())
}
```

**Example: Exclusive Choice (Pattern 4)**

```rust
/// Pattern 4: Exclusive Choice (XOR-split)
fn execute_exclusive_choice(ctx: &PatternExecutionContext) -> WorkflowResult<()> {
    // Ontology check: Verify split_type is Xor
    if ctx.split_type != SplitType::Xor {
        return Err(WorkflowError::PatternMismatch {
            expected: "Exclusive Choice (XOR)",
            actual: format!("{:?}", ctx.split_type),
        });
    }

    // Evaluate condition to select ONE flow
    let selected_flow = evaluate_condition(&ctx.variables, &ctx.outgoing_flows)?;

    // Enable only the selected flow
    enable_flow(ctx.case_id, &selected_flow)?;

    Ok(())
}
```

### 4.3 Dynamic Pattern Selection

**Derive pattern from ontology join/split types:**

```rust
impl WorkflowEngine {
    /// Derive pattern ID from task's ontology-defined join/split types
    fn derive_pattern_id(&self, task: &Task) -> String {
        match (task.join_type, task.split_type) {
            (JoinType::And, SplitType::And) => "pattern_1_sequence",
            (JoinType::Xor, SplitType::And) => "pattern_2_parallel_split",
            (JoinType::And, SplitType::Xor) => "pattern_4_exclusive_choice",
            (JoinType::And, SplitType::Or) => "pattern_6_multi_choice",
            (JoinType::Or, SplitType::And) => "pattern_41_thread_merge",
            // ... 43 patterns total
            _ => "pattern_1_sequence", // Default
        }
    }
}
```

**Pattern Mapping Table:**

| Join Type | Split Type | Pattern ID | Pattern Name |
|-----------|------------|------------|--------------|
| And | And | `pattern_1` | Sequence |
| Xor | And | `pattern_2` | Parallel Split |
| And | Xor | `pattern_4` | Exclusive Choice |
| And | Or | `pattern_6` | Multi-Choice |
| And | And | `pattern_7` | Synchronization |
| Xor | Xor | `pattern_5` | Simple Merge |
| Or | Or | `pattern_39` | Critical Section |
| ... | ... | ... | ... |

---

## 5. Hot Path Optimization

### 5.1 Performance Constraint: ≤8 Ticks

**Critical:** Task execution MUST complete within tick budget (from `knhk:tickBudget`).

#### 5.1.1 Tick Budget Enforcement

```rust
impl WorkflowEngine {
    /// Execute task with strict tick budget enforcement
    pub async fn execute_task_with_budget(
        &self,
        case_id: CaseId,
        task_id: &str,
    ) -> WorkflowResult<TaskExecutionMetrics> {
        let case = self.cases.get(&case_id)
            .ok_or(WorkflowError::CaseNotFound(case_id))?;
        let spec = self.specs.get(&case.workflow_id)
            .ok_or(WorkflowError::SpecNotFound(case.workflow_id))?;
        let task = spec.tasks.get(task_id)
            .ok_or(WorkflowError::TaskNotFound(task_id.to_string()))?;

        // Get tick budget (default to ∞ if not specified)
        let tick_budget = task.max_ticks.unwrap_or(u32::MAX);

        // Start tick counter
        let start_tick = self.get_current_tick();

        // Execute task
        self.execute_task_semantic(case_id, task_id).await?;

        // Measure elapsed ticks
        let end_tick = self.get_current_tick();
        let elapsed_ticks = (end_tick - start_tick) as u32;

        // Validate budget
        if elapsed_ticks > tick_budget {
            return Err(WorkflowError::TickBudgetExceeded {
                task_id: task_id.to_string(),
                max_ticks: tick_budget,
                actual_ticks: elapsed_ticks as u64,
            });
        }

        Ok(TaskExecutionMetrics {
            task_id: task_id.to_string(),
            tick_budget,
            elapsed_ticks,
            performance_tier: task.performance_tier,
        })
    }

    /// Get current CPU tick count
    fn get_current_tick(&self) -> u64 {
        // Use CPU tick counter (platform-specific)
        #[cfg(target_arch = "x86_64")]
        {
            unsafe { core::arch::x86_64::_rdtsc() }
        }
        #[cfg(target_arch = "aarch64")]
        {
            // ARM equivalent
            let mut val: u64;
            unsafe {
                core::arch::asm!("mrs {}, cntvct_el0", out(reg) val);
            }
            val
        }
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            0 // Fallback for unsupported architectures
        }
    }
}

#[derive(Debug)]
pub struct TaskExecutionMetrics {
    pub task_id: String,
    pub tick_budget: u32,
    pub elapsed_ticks: u32,
    pub performance_tier: PerformanceTier,
}
```

#### 5.1.2 Hot Path Guarantees

**Hot path tasks (knhk:performanceTier = "hot") MUST:**
1. ✅ Use cached Rust structs (NO RDF queries)
2. ✅ Avoid dynamic allocations (use pre-allocated buffers)
3. ✅ Use SIMD when `knhk:useSimd = true`
4. ✅ Complete within ≤8 ticks

**Implementation:**

```rust
impl WorkflowEngine {
    /// Execute hot path task with aggressive optimizations
    #[inline(always)]
    pub async fn execute_hot_path_task(
        &self,
        case_id: CaseId,
        task_id: &str,
    ) -> WorkflowResult<()> {
        // Assertion: Task must be marked as hot path
        let task = self.get_task_fast(case_id, task_id)?;
        assert_eq!(task.performance_tier, PerformanceTier::Hot);

        // Hot path guarantees
        assert!(task.max_ticks.unwrap_or(u32::MAX) <= 8);

        // NO allocations, NO RDF queries, NO async I/O
        // Everything pre-cached in task struct

        // Execute pattern (inline, no virtual dispatch)
        self.execute_pattern_inline(task)?;

        Ok(())
    }

    /// Fast task lookup (no bounds checking)
    #[inline(always)]
    fn get_task_fast(&self, case_id: CaseId, task_id: &str) -> WorkflowResult<&Task> {
        // Unsafe fast path (assumes validation done earlier)
        unsafe {
            let case = self.cases.get(&case_id).unwrap_unchecked();
            let spec = self.specs.get(&case.workflow_id).unwrap_unchecked();
            Ok(spec.tasks.get(task_id).unwrap_unchecked())
        }
    }

    /// Inline pattern execution (no function call overhead)
    #[inline(always)]
    fn execute_pattern_inline(&self, task: &Task) -> WorkflowResult<()> {
        // Direct pattern execution based on split_type
        match task.split_type {
            SplitType::And => {
                // Parallel split: enable all flows (pre-cached)
                for flow_id in &task.outgoing_flows {
                    self.enable_element_fast(flow_id);
                }
            }
            SplitType::Xor => {
                // Exclusive choice: enable first flow (simplified)
                if let Some(flow_id) = task.outgoing_flows.first() {
                    self.enable_element_fast(flow_id);
                }
            }
            SplitType::Or => {
                // Multi-choice: enable subset of flows
                for flow_id in &task.outgoing_flows {
                    self.enable_element_fast(flow_id);
                }
            }
        }
        Ok(())
    }

    #[inline(always)]
    fn enable_element_fast(&self, _element_id: &str) {
        // Ultra-fast element enabling (no locking, pre-validated)
        // Implementation details omitted for brevity
    }
}
```

---

## 6. Resource Allocation from Ontology

### 6.1 Allocation Policy Extraction

**From ontology:**
```turtle
@prefix yawl: <http://bitflow.ai/ontology/yawl/v2#> .

ex:TaskA a yawl:Task ;
    rdfs:label "Process Order" ;
    yawl:hasAllocationPolicy [
        a yawl:AllocationPolicy ;
        yawl:strategy "round-robin" ;
        yawl:maxConcurrent 10
    ] ;
    yawl:requiresRole "order-processor" ;
    yawl:requiresCapability "inventory-access" .
```

**Rust representation (from parser):**

```rust
pub struct Task {
    // ...
    pub allocation_policy: Option<AllocationPolicy>,
    pub required_roles: Vec<String>,       // ["order-processor"]
    pub required_capabilities: Vec<String>, // ["inventory-access"]
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AllocationPolicy {
    pub strategy: AllocationStrategy,
    pub max_concurrent: Option<u32>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum AllocationStrategy {
    RoundRobin,
    LeastLoaded,
    Random,
    Shortest Queue,
}
```

### 6.2 Runtime Resource Allocation

```rust
impl WorkflowEngine {
    /// Allocate resources based on ontology-derived policy
    async fn allocate_resources_for_task(
        &self,
        case_id: CaseId,
        task: &Task,
        policy: &AllocationPolicy,
    ) -> WorkflowResult<ResourceAllocation> {
        // Get available resources matching required roles/capabilities
        let available_resources = self.resource_allocator
            .find_matching_resources(
                &task.required_roles,
                &task.required_capabilities,
            )
            .await?;

        if available_resources.is_empty() {
            return Err(WorkflowError::NoAvailableResources {
                task_id: task.id.clone(),
                required_roles: task.required_roles.clone(),
            });
        }

        // Apply allocation strategy (from ontology)
        let selected_resource = match policy.strategy {
            AllocationStrategy::RoundRobin => {
                self.select_round_robin(&available_resources)
            }
            AllocationStrategy::LeastLoaded => {
                self.select_least_loaded(&available_resources).await?
            }
            AllocationStrategy::Random => {
                self.select_random(&available_resources)
            }
            AllocationStrategy::ShortestQueue => {
                self.select_shortest_queue(&available_resources).await?
            }
        };

        // Reserve resource
        self.resource_allocator
            .reserve_resource(case_id, task.id.clone(), selected_resource)
            .await?;

        Ok(ResourceAllocation {
            task_id: task.id.clone(),
            resource_id: selected_resource,
            allocated_at: chrono::Utc::now(),
        })
    }
}
```

---

## 7. OTEL Integration via Span Templates

### 7.1 Span Template Extraction

**From ontology:**
```turtle
@prefix knhk: <http://knhk.org/ontology#> .

ex:TaskA a yawl:Task ;
    rdfs:label "Process Order" ;
    knhk:hasSpanTemplate "workflow.task.process_order" .
```

**Rust representation:**
```rust
pub struct Task {
    // ...
    pub span_template: Option<String>, // Some("workflow.task.process_order")
}
```

### 7.2 Automatic Span Creation

```rust
impl WorkflowEngine {
    /// Create OTEL span from ontology template
    fn create_otel_span(&self, template: &str, task: &Task) -> WorkflowResult<SpanGuard> {
        if let Some(ref otel) = self.otel_integration {
            // Get span definition from Weaver registry
            let span_def = otel.get_span_template(template)?;

            // Create span with attributes from task
            let span = otel.start_span(
                span_def.name,
                vec![
                    ("task.id", task.id.clone()),
                    ("task.name", task.name.clone()),
                    ("task.split_type", format!("{:?}", task.split_type)),
                    ("task.join_type", format!("{:?}", task.join_type)),
                ],
            )?;

            Ok(SpanGuard::new(span))
        } else {
            // No OTEL integration, return no-op guard
            Ok(SpanGuard::noop())
        }
    }
}

/// RAII guard for OTEL spans
pub struct SpanGuard {
    span: Option<Span>,
}

impl SpanGuard {
    fn new(span: Span) -> Self {
        Self { span: Some(span) }
    }

    fn noop() -> Self {
        Self { span: None }
    }
}

impl Drop for SpanGuard {
    fn drop(&mut self) {
        if let Some(span) = self.span.take() {
            span.end();
        }
    }
}
```

### 7.3 Weaver Schema Validation

**Ensure span templates exist in Weaver registry:**

```rust
impl WorkflowEngine {
    /// Validate all span templates in workflow spec
    pub fn validate_span_templates(&self, spec: &WorkflowSpec) -> WorkflowResult<()> {
        if let Some(ref otel) = self.otel_integration {
            for task in spec.tasks.values() {
                if let Some(ref template) = task.span_template {
                    // Check if template exists in Weaver registry
                    if !otel.span_template_exists(template)? {
                        return Err(WorkflowError::Validation(
                            format!("Span template '{}' not found in Weaver registry", template)
                        ));
                    }
                }
            }
        }
        Ok(())
    }
}
```

---

## 8. State Transition with Ontology Semantics

### 8.1 Condition Enabling

**State transitions follow ontology-defined control flow:**

```rust
impl WorkflowEngine {
    /// Enable element (task or condition) based on ontology flows
    async fn enable_element(&self, case_id: CaseId, element_id: &str) -> WorkflowResult<()> {
        let mut case = self.cases.get_mut(&case_id)
            .ok_or(WorkflowError::CaseNotFound(case_id))?;

        // Mark element as enabled
        case.enabled_elements.insert(element_id.to_string());

        // Check if element is a task
        let spec = self.specs.get(&case.workflow_id)
            .ok_or(WorkflowError::SpecNotFound(case.workflow_id))?;

        if let Some(task) = spec.tasks.get(element_id) {
            // Check if task can execute (join condition satisfied)
            if self.check_join_condition(case_id, task)? {
                // Execute task
                drop(case); // Release lock before async call
                self.execute_task_semantic(case_id, element_id).await?;
            }
        }

        Ok(())
    }
}
```

### 8.2 Case State with Ontology Context

```rust
pub struct Case {
    pub id: CaseId,
    pub workflow_id: WorkflowSpecId,
    pub state: CaseState,
    pub variables: HashMap<String, String>,

    // NEW: Ontology-derived execution state
    /// Elements that have received tokens (for join evaluation)
    pub arrived_from: HashSet<String>,

    /// Currently enabled elements
    pub enabled_elements: HashSet<String>,

    /// Completed tasks
    pub completed_tasks: HashSet<String>,

    /// OTEL trace context (if OTEL enabled)
    pub trace_context: Option<TraceContext>,

    /// Lockchain provenance chain (if provenance enabled)
    pub provenance_chain: Option<String>,
}
```

---

## 9. Performance Validation

### 9.1 Metrics Collection

```rust
impl WorkflowEngine {
    /// Execute task with comprehensive metrics
    pub async fn execute_task_with_metrics(
        &self,
        case_id: CaseId,
        task_id: &str,
    ) -> WorkflowResult<TaskExecutionReport> {
        let start = std::time::Instant::now();
        let start_tick = self.get_current_tick();

        // Execute task
        self.execute_task_semantic(case_id, task_id).await?;

        let end_tick = self.get_current_tick();
        let duration = start.elapsed();

        // Get task metadata
        let task = self.get_task_cached(case_id, task_id)?;

        Ok(TaskExecutionReport {
            task_id: task_id.to_string(),
            task_name: task.name.clone(),
            performance_tier: task.performance_tier,
            tick_budget: task.max_ticks,
            elapsed_ticks: (end_tick - start_tick) as u32,
            elapsed_time: duration,
            budget_satisfied: task.max_ticks.map_or(true, |max| {
                (end_tick - start_tick) as u32 <= max
            }),
        })
    }
}

#[derive(Debug)]
pub struct TaskExecutionReport {
    pub task_id: String,
    pub task_name: String,
    pub performance_tier: PerformanceTier,
    pub tick_budget: Option<u32>,
    pub elapsed_ticks: u32,
    pub elapsed_time: std::time::Duration,
    pub budget_satisfied: bool,
}
```

### 9.2 Automated Performance Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hot_path_task_meets_budget() {
        let engine = WorkflowEngine::new().await;

        // Load workflow with hot path task (max_ticks = 8)
        let spec = load_test_workflow_with_hot_path_task();
        engine.register_workflow(spec).await.unwrap();

        // Create case
        let case_id = engine.create_case(spec.id, HashMap::new()).await.unwrap();

        // Execute hot path task
        let report = engine
            .execute_task_with_metrics(case_id, "hot_path_task")
            .await
            .unwrap();

        // Validate tick budget
        assert!(report.budget_satisfied, "Hot path task exceeded tick budget");
        assert!(report.elapsed_ticks <= 8, "Hot path task took {} ticks (max: 8)", report.elapsed_ticks);
    }
}
```

---

## 10. Implementation Examples

### 10.1 Complete Task Execution Flow

```rust
impl WorkflowEngine {
    /// Full task execution with ontology integration
    pub async fn execute_task_full(
        &self,
        case_id: CaseId,
        task_id: &str,
    ) -> WorkflowResult<()> {
        // ====== PHASE 1: PREPARE ======
        // Get case and spec (cached, NO RDF)
        let case = self.cases.get(&case_id)
            .ok_or(WorkflowError::CaseNotFound(case_id))?;
        let spec = self.specs.get(&case.workflow_id)
            .ok_or(WorkflowError::SpecNotFound(case.workflow_id))?;
        let task = spec.tasks.get(task_id)
            .ok_or(WorkflowError::TaskNotFound(task_id.to_string()))?;

        // ====== PHASE 2: VALIDATE ======
        // Check join condition (ontology join_type)
        if !self.check_join_condition(case_id, task)? {
            return Ok(()); // Not enabled yet
        }

        // Check authentication (if auth_manager enabled)
        if let Some(ref auth) = self.auth_manager {
            auth.read().await.check_task_permission(case_id, task_id)?;
        }

        // ====== PHASE 3: ALLOCATE RESOURCES ======
        let _resource_guard = if let Some(ref policy) = task.allocation_policy {
            Some(self.allocate_resources_for_task(case_id, task, policy).await?)
        } else {
            None
        };

        // ====== PHASE 4: START TELEMETRY ======
        // Create OTEL span (if span_template defined)
        let _span = if let Some(ref template) = task.span_template {
            Some(self.create_otel_span(template, task)?)
        } else {
            None
        };

        // Start Lockchain tracking (if provenance_required)
        let _lockchain = if task.provenance_required {
            Some(self.start_lockchain_tracking(case_id, task_id)?)
        } else {
            None
        };

        // ====== PHASE 5: EXECUTE PATTERN ======
        let ctx = PatternExecutionContext::from_task(
            case_id,
            case.workflow_id,
            task,
            case.variables.clone(),
            case.arrived_from.clone(),
        );

        let start_tick = self.get_current_tick();
        self.execute_pattern_with_semantics(task, &ctx).await?;
        let elapsed_ticks = (self.get_current_tick() - start_tick) as u32;

        // Validate tick budget
        if let Some(max_ticks) = task.max_ticks {
            if elapsed_ticks > max_ticks {
                return Err(WorkflowError::TickBudgetExceeded {
                    task_id: task_id.to_string(),
                    max_ticks,
                    actual_ticks: elapsed_ticks as u64,
                });
            }
        }

        // ====== PHASE 6: APPLY SPLIT SEMANTICS ======
        self.apply_split_semantics(case_id, task, &ctx).await?;

        // ====== PHASE 7: UPDATE STATE ======
        let mut case_mut = self.cases.get_mut(&case_id).unwrap();
        case_mut.completed_tasks.insert(task_id.to_string());
        drop(case_mut);

        // Persist state
        self.state_manager.save_case(&case).await?;

        Ok(())
    }
}
```

### 10.2 Integration Test Example

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_ontology_driven_workflow_execution() {
        // ====== SETUP ======
        let mut parser = WorkflowParser::new_with_ontology(Path::new("ontology")).unwrap();

        // Load workflow TTL with ontology annotations
        let workflow_ttl = r#"
            @prefix yawl: <http://bitflow.ai/ontology/yawl/v2#> .
            @prefix knhk: <http://knhk.org/ontology#> .

            ex:Workflow a yawl:WorkflowSpecification ;
                rdfs:label "Order Processing" .

            ex:TaskA a yawl:Task, knhk:HotPathTask ;
                rdfs:label "Validate Order" ;
                yawl:joinType "AND" ;
                yawl:splitType "XOR" ;
                knhk:tickBudget 8 ;
                knhk:hasSpanTemplate "workflow.validate_order" .
        "#;

        let spec = parser.parse_turtle(workflow_ttl).unwrap();

        // ====== EXECUTE ======
        let engine = WorkflowEngine::new().await;
        engine.register_workflow(spec).await.unwrap();

        let case_id = engine.create_case(spec.id, HashMap::new()).await.unwrap();
        engine.start_case(case_id).await.unwrap();

        // ====== VALIDATE ======
        let metrics = engine.execute_task_with_metrics(case_id, "ex:TaskA").await.unwrap();

        assert!(metrics.budget_satisfied);
        assert!(metrics.elapsed_ticks <= 8);
    }
}
```

---

## Summary

This integration design provides **hyper-detailed, implementation-ready specifications** for wiring the YAWL ontology into the runtime `WorkflowEngine`. Key features:

1. **Zero RDF Queries in Hot Path:** All ontology data cached in Rust structs during parsing
2. **≤8 Tick Compliance:** Tick budget enforcement with inline optimizations
3. **Semantic Pattern Execution:** Join/split semantics from ontology drive pattern selection
4. **OTEL Span Templates:** Automatic span creation from `knhk:hasSpanTemplate`
5. **Lockchain Provenance:** Tracking for `knhk:requiresProvenance` tasks
6. **Resource Allocation:** Ontology-defined policies applied at runtime
7. **Performance Validation:** Comprehensive metrics and automated testing

**Next:** See `state-manager-integration.md` for state persistence and provenance integration.
