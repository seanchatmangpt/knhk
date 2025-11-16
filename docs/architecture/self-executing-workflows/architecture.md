# Self-Executing Workflows Architecture

**Version**: 1.0
**Status**: Design Specification
**Last Updated**: 2025-11-16
**Author**: System Architecture Team

---

## Executive Summary

This document defines the complete system architecture for self-executing workflows in KNHK, connecting 5 architectural layers into a unified Knowledge Graph Computing (KGC) system where workflows autonomously adapt, execute, and evolve based on runtime observations.

**Core Principle**: **A = μ(O)** - Actions are deterministic functions of observations, with provable invariants (Q), projections (Π), and autonomic feedback (MAPE-K).

**Key Innovation**: Workflows become self-executing by closing the loop from observation back to ontology updates, creating an autonomic system that learns and adapts while preserving formal guarantees.

### Mathematical Foundation

```
Given:
- Σ: Ontology (YAWL + MAPE-K + domain sectors)
- O: Observations (OTEL telemetry + receipts)
- μ: Execution function (KNHK workflow engine)
- Π: Projection function (ggen templates)
- Q: Invariants (performance, correctness, safety constraints)

Properties:
1. A = μ(O)                    (Deterministic execution)
2. μ ∘ μ = μ                   (Idempotence)
3. O ⊨ Σ                       (Observations respect ontology)
4. Σ ⊨ Q                       (Ontology respects invariants)
5. latency(μ) ≤ 8 ticks        (Chatman constant for hot path)
6. Σ_t → Σ_{t+1}              (Ontology evolution via MAPE-K)
```

### Architecture Layers

```
┌─────────────────────────────────────────────────────────────┐
│ Layer 5: MAPE-K Feedback (Autonomic Control)               │
│ - Monitor: Analyze receipts, detect patterns               │
│ - Analyze: Compare O vs Σ, identify gaps                   │
│ - Plan: Generate ontology updates (Σ_t → Σ_{t+1})         │
│ - Execute: Apply updates atomically via snapshots          │
│ - Knowledge: Persist learning, update templates            │
└─────────────────────────────────────────────────────────────┘
                              ↓↑ (feedback loop)
┌─────────────────────────────────────────────────────────────┐
│ Layer 4: Observation (O)                                    │
│ - OTEL telemetry (spans, metrics, logs)                    │
│ - Receipts (provenance: ticks, hash, span_id, timestamp)   │
│ - Weaver schema validation (source of truth)               │
│ - Performance metrics (latency ≤ 8 ticks)                  │
└─────────────────────────────────────────────────────────────┘
                              ↑ (observability)
┌─────────────────────────────────────────────────────────────┐
│ Layer 3: Execution (μ)                                      │
│ - KNHK workflow engine (Van der Aalst 43 patterns)         │
│ - Hook-based coordination (pre-task, post-task, session)   │
│ - State persistence (Sled)                                  │
│ - Resource allocation (Four-eyes, Round-robin, etc.)       │
└─────────────────────────────────────────────────────────────┘
                              ↑ (projection)
┌─────────────────────────────────────────────────────────────┐
│ Layer 2: Projection (Π via ggen)                           │
│ - SPARQL queries (extract data from Σ)                     │
│ - Tera templates (generate workflows, tests, docs)         │
│ - RDF-driven code generation                               │
│ - Template composition and reuse                           │
└─────────────────────────────────────────────────────────────┘
                              ↑ (specification)
┌─────────────────────────────────────────────────────────────┐
│ Layer 1: Ontology (Σ)                                      │
│ - YAWL patterns (43 Van der Aalst workflow patterns)       │
│ - MAPE-K knowledge (autonomic policies)                    │
│ - Domain sectors (finance, manufacturing, etc.)            │
│ - Snapshots (versioned, atomic updates)                    │
│ - Invariants Q (performance, correctness, safety)          │
└─────────────────────────────────────────────────────────────┘
```

---

## Table of Contents

1. [Layer 1: Ontology (Σ)](#layer-1-ontology-σ)
2. [Layer 2: Projection (Π via ggen)](#layer-2-projection-π-via-ggen)
3. [Layer 3: Execution (μ via KNHK)](#layer-3-execution-μ-via-knhk)
4. [Layer 4: Observation (O)](#layer-4-observation-o)
5. [Layer 5: MAPE-K Feedback](#layer-5-mape-k-feedback)
6. [Cross-Layer Integration](#cross-layer-integration)
7. [Data Flow Architecture](#data-flow-architecture)
8. [Snapshot System](#snapshot-system)
9. [Receipt Structure](#receipt-structure)
10. [Directory Structure](#directory-structure)
11. [Validation Strategy](#validation-strategy)
12. [Performance Guarantees](#performance-guarantees)
13. [Implementation Roadmap](#implementation-roadmap)

---

## Layer 1: Ontology (Σ)

### Purpose

Define the "source of truth" for all workflows, patterns, policies, and domain knowledge. The ontology is versioned via snapshots and updated atomically through the MAPE-K feedback loop.

### Components

#### 1.1 YAWL Pattern Ontology

**File**: `ontology/yawl.ttl`

Defines all 43 Van der Aalst workflow patterns with formal semantics:

```turtle
@prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .
@prefix knhk: <urn:knhk:ontology#> .

# Pattern 1: Sequence
knhk:PatternSequence a knhk:BasicControlFlowPattern ;
    knhk:hasPatternNumber 1 ;
    knhk:hasPatternName "Sequence" ;
    knhk:hasSplitType "AND" ;
    knhk:hasJoinType "AND" ;
    knhk:hasExecutionTicks 1 ;
    knhk:isDeterministic true ;
    knhk:preservesKGC true .

# Pattern 2: Parallel Split
knhk:PatternParallelSplit a knhk:BasicControlFlowPattern ;
    knhk:hasPatternNumber 2 ;
    knhk:hasPatternName "ParallelSplit" ;
    knhk:hasSplitType "AND" ;
    knhk:hasJoinType "AND" ;
    knhk:hasExecutionTicks 2 ;
    knhk:isDeterministic true ;
    knhk:preservesKGC true .

# ... (41 more patterns)
```

**Pattern Categories**:
- Basic Control Flow (Patterns 1-5)
- Advanced Branching (Patterns 6-11)
- Multiple Instance (Patterns 12-15)
- State-Based (Patterns 16-18)
- Cancellation (Patterns 19-25)
- Advanced Control (Patterns 26-39)
- Trigger-Based (Patterns 40-43)

#### 1.2 MAPE-K Knowledge Ontology

**File**: `ontology/mape-k.ttl`

Defines autonomic computing policies and knowledge:

```turtle
@prefix mape: <urn:knhk:mape-k#> .
@prefix knhk: <urn:knhk:ontology#> .

# Monitor Policy
mape:MonitorPolicy a mape:Policy ;
    mape:monitorsMetric knhk:LatencyMetric ;
    mape:threshold "8 ticks" ;
    mape:samplingRate "100%" ;
    mape:alertOn mape:ThresholdExceeded .

# Analyze Policy
mape:AnalyzePolicy a mape:Policy ;
    mape:comparesObservation mape:ActualLatency ;
    mape:withExpectation mape:ExpectedLatency ;
    mape:detectsAnomaly mape:LatencySpike ;
    mape:triggersAction mape:PlanAdaptation .

# Plan Policy
mape:PlanPolicy a mape:Policy ;
    mape:generatesUpdate mape:OntologyUpdate ;
    mape:preservesInvariant knhk:PerformanceInvariant ;
    mape:validatesVia mape:InvariantChecker .

# Execute Policy
mape:ExecutePolicy a mape:Policy ;
    mape:appliesUpdate mape:AtomicSnapshot ;
    mape:rollbackOn mape:ValidationFailure ;
    mape:notifiesVia mape:EventBus .

# Knowledge Base
mape:KnowledgeBase a mape:Knowledge ;
    mape:learnsFrom mape:ReceiptHistory ;
    mape:detectsPattern mape:PerformancePattern ;
    mape:suggestsOptimization mape:WorkflowOptimization .
```

#### 1.3 Domain Sector Ontologies

**Files**: `ontology/sectors/*.ttl`

Domain-specific knowledge (finance, manufacturing, healthcare, etc.):

```turtle
# ontology/sectors/finance.ttl
@prefix fin: <urn:knhk:sector:finance#> .
@prefix knhk: <urn:knhk:ontology#> .

fin:SwiftPaymentWorkflow a knhk:Workflow ;
    knhk:hasPattern knhk:PatternParallelSplit ;
    knhk:hasPattern knhk:PatternSynchronization ;
    fin:requiresCompliance fin:ISO20022 ;
    fin:requiresSLA "≤8 ticks" ;
    fin:requiresAudit true .

fin:FourEyesApproval a knhk:Resource ;
    knhk:hasResourcePolicy "Four-eyes" ;
    fin:minimumApprovers 2 ;
    fin:requiresSegregation true .
```

#### 1.4 Invariants (Q)

**File**: `ontology/invariants.ttl`

Formal constraints that must always hold:

```turtle
@prefix inv: <urn:knhk:invariants#> .
@prefix sh: <http://www.w3.org/ns/shacl#> .

# Performance Invariant: Hot path ≤ 8 ticks
inv:HotPathPerformance a sh:PropertyShape ;
    sh:path knhk:hasTicks ;
    sh:maxInclusive 8 ;
    sh:message "Hot path operations must complete in ≤8 ticks (Chatman constant)" .

# Correctness Invariant: Idempotence
inv:IdempotenceInvariant a sh:NodeShape ;
    sh:targetClass knhk:Pattern ;
    sh:property [
        sh:path knhk:isDeterministic ;
        sh:hasValue true ;
        sh:message "All patterns must be deterministic (A = μ(O))"
    ] .

# Safety Invariant: No deadlocks
inv:DeadlockFreedom a sh:NodeShape ;
    sh:targetClass knhk:Workflow ;
    sh:sparql [
        sh:select """
            SELECT $this WHERE {
                $this knhk:hasCycle true .
            }
        """ ;
        sh:message "Workflows must be deadlock-free"
    ] .

# Consistency Invariant: O ⊨ Σ
inv:ObservationConsistency a sh:NodeShape ;
    sh:targetClass knhk:Receipt ;
    sh:property [
        sh:path knhk:hasPatternId ;
        sh:node inv:ValidPatternId ;
        sh:message "Receipts must reference valid patterns from ontology"
    ] .
```

### Snapshot System for Σ

Ontology updates are versioned and applied atomically:

```rust
// Snapshot structure
pub struct OntologySnapshot {
    pub version: u64,
    pub timestamp: SystemTime,
    pub rdf_graph: Store,
    pub invariants: Vec<InvariantRule>,
    pub parent_hash: Hash,
    pub snapshot_hash: Hash,
}

// Atomic update protocol
impl OntologyManager {
    pub async fn apply_update(&mut self, update: OntologyUpdate) -> Result<()> {
        // 1. Create new snapshot from current
        let mut new_snapshot = self.current_snapshot.clone();
        new_snapshot.version += 1;

        // 2. Apply update to new snapshot
        new_snapshot.apply_update(update)?;

        // 3. Validate invariants Q
        if !new_snapshot.validate_invariants()? {
            return Err(Error::InvariantViolation);
        }

        // 4. Compute snapshot hash
        new_snapshot.snapshot_hash = new_snapshot.compute_hash()?;

        // 5. Atomically swap (linearizable)
        self.current_snapshot.store(Arc::new(new_snapshot));

        // 6. Persist to storage
        self.persist_snapshot().await?;

        Ok(())
    }
}
```

**Directory Structure**:
```
ontology/
├── knhk.owl.ttl           # Core KNHK ontology (existing)
├── yawl.ttl               # YAWL pattern definitions (existing)
├── mape-k.ttl             # MAPE-K policies (NEW)
├── invariants.ttl         # Formal invariants Q (NEW)
├── sectors/               # Domain-specific ontologies
│   ├── finance.ttl
│   ├── manufacturing.ttl
│   └── healthcare.ttl
└── snapshots/             # Versioned snapshots (NEW)
    ├── snapshot-v1.ttl
    ├── snapshot-v2.ttl
    └── current -> snapshot-v2.ttl
```

---

## Layer 2: Projection (Π via ggen)

### Purpose

Transform ontology (Σ) into executable artifacts (workflows, tests, documentation) via SPARQL queries and Tera templates.

### Components

#### 2.1 SPARQL Query Interface

**File**: `rust/knhk-workflow-engine/src/ggen/sparql.rs`

Execute SPARQL queries against ontology to extract data:

```rust
pub struct SparqlProjector {
    ontology_store: Arc<Store>,
    query_cache: Cache<String, QueryResults>,
}

impl SparqlProjector {
    /// Query patterns by category
    pub fn query_patterns(&self, category: &str) -> Result<Vec<Pattern>> {
        let query = format!(r#"
            PREFIX knhk: <urn:knhk:ontology#>
            SELECT ?pattern ?name ?splitType ?joinType ?ticks
            WHERE {{
                ?pattern a knhk:{category} ;
                    knhk:hasPatternName ?name ;
                    knhk:hasSplitType ?splitType ;
                    knhk:hasJoinType ?joinType ;
                    knhk:hasExecutionTicks ?ticks .
            }}
            ORDER BY ?pattern
        "#);

        self.execute_query(&query)
    }

    /// Query workflows for a sector
    pub fn query_sector_workflows(&self, sector: &str) -> Result<Vec<Workflow>> {
        let query = format!(r#"
            PREFIX knhk: <urn:knhk:ontology#>
            PREFIX sec: <urn:knhk:sector:{sector}#>
            SELECT ?workflow ?pattern ?sla
            WHERE {{
                ?workflow a knhk:Workflow ;
                    knhk:hasPattern ?pattern ;
                    sec:requiresSLA ?sla .
            }}
        "#);

        self.execute_query(&query)
    }
}
```

#### 2.2 Template Engine

**File**: `rust/knhk-workflow-engine/src/ggen/templates.rs`

Tera templates that consume SPARQL results:

```rust
pub struct TemplateEngine {
    tera: Tera,
    sparql_projector: Arc<SparqlProjector>,
}

impl TemplateEngine {
    /// Generate workflow from pattern
    pub fn generate_workflow(&self, pattern_id: u32) -> Result<String> {
        // Query ontology for pattern details
        let pattern = self.sparql_projector.query_pattern_by_id(pattern_id)?;

        // Build template context
        let mut context = Context::new();
        context.insert("pattern", &pattern);
        context.insert("timestamp", &SystemTime::now());

        // Render template
        self.tera.render("workflow.tmpl", &context)
    }

    /// Generate tests from workflow
    pub fn generate_tests(&self, workflow: &Workflow) -> Result<String> {
        let mut context = Context::new();
        context.insert("workflow", workflow);
        context.insert("patterns", &workflow.patterns);

        self.tera.render("tests.tmpl", &context)
    }
}
```

**Template Example** (`templates/workflow.tmpl`):

```jinja2
@prefix wf: <http://knhk.ai/workflow#> .
@prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .

# Generated Workflow: {{ pattern.name }}
# Pattern: {{ pattern.pattern_number }}
# Generated: {{ timestamp }}

<workflow:{{ pattern.name | lower }}-v1> a yawl:WorkflowSpec ;
    yawl:name "{{ pattern.name }} Workflow" ;
    yawl:pattern <pattern:{{ pattern.pattern_number }}> ;
    yawl:splitType yawl:{{ pattern.split_type }} ;
    yawl:joinType yawl:{{ pattern.join_type }} ;
    knhk:expectedTicks {{ pattern.execution_ticks }} .

{% for task in tasks %}
<task:task{{ loop.index }}> a yawl:Task ;
    yawl:name "Task {{ loop.index }}" ;
    yawl:splitType yawl:{{ pattern.split_type }} ;
    yawl:joinType yawl:{{ pattern.join_type }} .
{% endfor %}
```

#### 2.3 Code Generation

Transform ontology patterns into executable code:

```rust
pub struct CodeGenerator {
    template_engine: Arc<TemplateEngine>,
}

impl CodeGenerator {
    /// Generate Rust code for pattern executor
    pub fn generate_pattern_executor(&self, pattern: &Pattern) -> Result<String> {
        let mut context = Context::new();
        context.insert("pattern", pattern);

        self.template_engine.tera.render("pattern_executor.rs.tmpl", &context)
    }

    /// Generate workflow engine integration
    pub fn generate_workflow_integration(&self, workflows: &[Workflow]) -> Result<String> {
        let mut context = Context::new();
        context.insert("workflows", workflows);

        self.template_engine.tera.render("workflow_integration.rs.tmpl", &context)
    }
}
```

**Directory Structure**:
```
rust/knhk-workflow-engine/src/ggen/
├── mod.rs                 # ggen module (existing)
├── sparql.rs              # SPARQL query interface (NEW)
├── templates.rs           # Template engine (ENHANCED)
├── codegen.rs             # Code generator (NEW)
└── cache.rs               # Query result caching (NEW)

templates/
├── workflow.tmpl          # Workflow generation
├── tests.tmpl             # Test generation
├── docs.tmpl              # Documentation generation
├── pattern_executor.rs.tmpl  # Rust code generation
└── README.md
```

---

## Layer 3: Execution (μ via KNHK)

### Purpose

Execute workflows deterministically (A = μ(O)) using Van der Aalst patterns, with hook-based coordination and provenance tracking.

### Components

#### 3.1 Workflow Engine

**File**: `rust/knhk-workflow-engine/src/engine.rs` (existing, enhanced)

Core execution engine with deterministic pattern execution:

```rust
pub struct WorkflowEngine {
    state_store: Arc<StateStore>,
    pattern_registry: Arc<PatternRegistry>,
    resource_allocator: Arc<ResourceAllocator>,
    otel_integration: Arc<OtelIntegration>,
    lockchain_integration: Arc<LockchainIntegration>,
    hook_manager: Arc<HookManager>,  // NEW
}

impl WorkflowEngine {
    /// Execute workflow case deterministically (A = μ(O))
    pub async fn execute_case(&self, case_id: CaseId) -> Result<ExecutionReceipt> {
        // Pre-execution hook
        self.hook_manager.pre_task(case_id).await?;

        let start = Instant::now();
        let start_ticks = rdtsc();

        // Get case state (observation O)
        let case = self.state_store.get_case(case_id).await?;

        // Execute patterns deterministically (μ)
        let action = self.execute_patterns(&case).await?;

        // Compute execution metrics
        let end_ticks = rdtsc();
        let ticks = end_ticks - start_ticks;

        // Verify Chatman constant (≤8 ticks for hot path)
        if case.is_hot_path() && ticks > 8 {
            tracing::warn!(
                ticks = ticks,
                case_id = %case_id,
                "Hot path exceeded Chatman constant (8 ticks)"
            );
        }

        // Generate receipt (provenance)
        let receipt = ExecutionReceipt {
            case_id,
            action_hash: hash(&action),
            ticks,
            span_id: Span::current().id(),
            timestamp: SystemTime::now(),
            cycle_id: compute_cycle_id(),
            shard_id: self.shard_id.clone(),
        };

        // Post-execution hook
        self.hook_manager.post_task(case_id, &receipt).await?;

        // Emit to lockchain
        self.lockchain_integration.record_receipt(&receipt).await?;

        Ok(receipt)
    }

    /// Execute patterns deterministically
    async fn execute_patterns(&self, case: &Case) -> Result<Action> {
        let spec = self.state_store.get_spec(case.spec_id).await?;

        // Get next enabled tasks
        let enabled_tasks = self.get_enabled_tasks(case).await?;

        // Execute each task via pattern
        for task_id in enabled_tasks {
            let task = &spec.tasks[&task_id];
            let pattern = self.pattern_registry.get_pattern(task.pattern)?;

            // Execute pattern (deterministic A = μ(O))
            let result = pattern.execute(task, case).await?;

            // Update case state
            self.state_store.update_case(case.id, result).await?;
        }

        Ok(Action::Executed)
    }
}
```

#### 3.2 Hook Manager

**File**: `rust/knhk-workflow-engine/src/hooks/manager.rs` (NEW)

Coordinate execution via hooks:

```rust
pub struct HookManager {
    memory_store: Arc<MemoryStore>,
    event_bus: Arc<EventBus>,
}

impl HookManager {
    /// Pre-task hook
    pub async fn pre_task(&self, case_id: CaseId) -> Result<()> {
        // Restore session context
        let context = self.memory_store.get(format!("case/{}", case_id)).await?;

        // Emit pre-task event
        self.event_bus.publish(Event::PreTask { case_id, context }).await?;

        Ok(())
    }

    /// Post-task hook
    pub async fn post_task(&self, case_id: CaseId, receipt: &ExecutionReceipt) -> Result<()> {
        // Store receipt in memory
        self.memory_store.set(
            format!("receipt/{}", case_id),
            serde_json::to_value(receipt)?
        ).await?;

        // Emit post-task event
        self.event_bus.publish(Event::PostTask {
            case_id,
            receipt: receipt.clone()
        }).await?;

        Ok(())
    }

    /// Session end hook
    pub async fn session_end(&self, case_id: CaseId) -> Result<SessionSummary> {
        // Collect all receipts
        let receipts = self.memory_store.get_all(format!("receipt/{}/*", case_id)).await?;

        // Generate summary
        let summary = SessionSummary::from_receipts(receipts)?;

        // Persist summary
        self.memory_store.set(
            format!("session/{}/summary", case_id),
            serde_json::to_value(&summary)?
        ).await?;

        Ok(summary)
    }
}
```

#### 3.3 Pattern Execution

**File**: `rust/knhk-workflow-engine/src/patterns/executor.rs` (enhanced)

Van der Aalst pattern execution with idempotence (μ ∘ μ = μ):

```rust
pub trait PatternExecutor: Send + Sync {
    /// Execute pattern deterministically
    async fn execute(&self, task: &Task, case: &Case) -> Result<PatternResult>;

    /// Verify idempotence
    async fn verify_idempotence(&self, task: &Task, case: &Case) -> Result<bool>;
}

/// Pattern 1: Sequence (AND-split/AND-join)
pub struct SequencePattern;

impl PatternExecutor for SequencePattern {
    async fn execute(&self, task: &Task, case: &Case) -> Result<PatternResult> {
        // Execute current task
        let result = task.execute(case).await?;

        // Enable next task (sequential)
        let next_tasks = task.output_conditions
            .iter()
            .flat_map(|cond| cond.output_tasks.clone())
            .collect();

        Ok(PatternResult {
            completed_tasks: vec![task.id.clone()],
            enabled_tasks: next_tasks,
            state_updates: result.state_updates,
        })
    }

    async fn verify_idempotence(&self, task: &Task, case: &Case) -> Result<bool> {
        // Execute twice, verify results match
        let result1 = self.execute(task, case).await?;
        let result2 = self.execute(task, case).await?;

        Ok(result1 == result2)  // μ ∘ μ = μ
    }
}
```

**Directory Structure**:
```
rust/knhk-workflow-engine/src/
├── engine.rs              # Core engine (enhanced)
├── executor/
│   ├── mod.rs
│   ├── task.rs            # Task executor
│   └── pattern.rs         # Pattern dispatcher
├── hooks/                 # Hook system (NEW)
│   ├── mod.rs
│   ├── manager.rs
│   ├── pre_task.rs
│   ├── post_task.rs
│   └── session.rs
├── patterns/              # Van der Aalst patterns
│   ├── mod.rs
│   ├── executor.rs        # Pattern trait
│   ├── basic_control.rs   # Patterns 1-5
│   ├── advanced_branch.rs # Patterns 6-11
│   ├── multiple_instance.rs  # Patterns 12-15
│   ├── state_based.rs     # Patterns 16-18
│   └── ... (all 43 patterns)
└── state/
    ├── store.rs           # State persistence
    └── snapshot.rs        # State snapshots
```

---

## Layer 4: Observation (O)

### Purpose

Collect runtime telemetry, generate receipts, and validate against ontology (O ⊨ Σ) using Weaver as source of truth.

### Components

#### 4.1 OTEL Integration

**File**: `rust/knhk-workflow-engine/src/integration/otel.rs` (existing, enhanced)

OpenTelemetry instrumentation:

```rust
pub struct OtelIntegration {
    tracer: Tracer,
    meter: Meter,
}

impl OtelIntegration {
    /// Create span for workflow execution
    pub fn span_workflow_execution(&self, case_id: CaseId) -> Span {
        self.tracer
            .span_builder("knhk.workflow.execution")
            .with_attributes(vec![
                KeyValue::new("case_id", case_id.to_string()),
                KeyValue::new("system", "knhk"),
            ])
            .start(&self.tracer)
    }

    /// Record pattern execution metrics
    pub fn record_pattern_execution(&self, pattern: &Pattern, ticks: u64) {
        let histogram = self.meter
            .u64_histogram("knhk.pattern.execution.ticks")
            .with_description("Pattern execution time in CPU ticks")
            .init();

        histogram.record(
            ticks,
            &[
                KeyValue::new("pattern.id", pattern.id as i64),
                KeyValue::new("pattern.name", pattern.name.clone()),
            ],
        );
    }
}
```

#### 4.2 Receipt Generation

**File**: `rust/knhk-workflow-engine/src/provenance/receipt.rs` (NEW)

Cryptographically signed execution receipts:

```rust
use sha2::{Sha256, Digest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionReceipt {
    /// Case ID
    pub case_id: CaseId,

    /// Action hash: hash(A) = hash(μ(O))
    pub action_hash: Hash,

    /// Execution time in CPU ticks
    pub ticks: u64,

    /// OTEL span ID
    pub span_id: SpanId,

    /// Unix timestamp
    pub timestamp: SystemTime,

    /// 8-beat cycle ID
    pub cycle_id: u64,

    /// Shard identifier
    pub shard_id: String,

    /// Merkle proof (lockchain)
    pub merkle_proof: Option<MerkleProof>,
}

impl ExecutionReceipt {
    /// Compute receipt hash
    pub fn compute_hash(&self) -> Hash {
        let mut hasher = Sha256::new();
        hasher.update(&self.case_id.to_bytes());
        hasher.update(&self.action_hash);
        hasher.update(&self.ticks.to_le_bytes());
        hasher.update(&self.span_id.to_bytes());
        hasher.update(&self.timestamp.duration_since(UNIX_EPOCH).unwrap().as_secs().to_le_bytes());
        hasher.update(&self.cycle_id.to_le_bytes());
        hasher.update(self.shard_id.as_bytes());

        Hash::from(hasher.finalize().as_slice())
    }

    /// Verify receipt against ontology (O ⊨ Σ)
    pub async fn verify_against_ontology(&self, ontology: &OntologySnapshot) -> Result<bool> {
        // Verify pattern exists in ontology
        let pattern_exists = ontology.has_pattern(self.pattern_id)?;
        if !pattern_exists {
            return Ok(false);
        }

        // Verify expected ticks match pattern
        let expected_ticks = ontology.get_pattern_ticks(self.pattern_id)?;
        if self.ticks > expected_ticks * 2 {  // Allow 2x tolerance
            return Ok(false);
        }

        // Verify invariants
        ontology.verify_invariants(self)?
    }
}
```

#### 4.3 Weaver Validation

**File**: `registry/workflow-telemetry.yaml` (NEW)

Weaver schema for workflow telemetry (source of truth):

```yaml
groups:
  - id: knhk.workflow
    type: span
    brief: "KNHK workflow execution"
    span_kind: internal
    attributes:
      - ref: knhk.case_id
      - ref: knhk.pattern_id
      - ref: knhk.execution_ticks
      - ref: knhk.action_hash

attributes:
  - id: knhk.case_id
    type: string
    brief: "Workflow case identifier"
    examples: ["550e8400-e29b-41d4-a716-446655440000"]

  - id: knhk.pattern_id
    type: int
    brief: "Van der Aalst pattern number (1-43)"
    examples: [1, 2, 3]

  - id: knhk.execution_ticks
    type: int
    brief: "Execution time in CPU ticks"
    examples: [4, 8, 12]

  - id: knhk.action_hash
    type: string
    brief: "SHA-256 hash of action"
    examples: ["abc123..."]
```

**Validation**:
```bash
# Schema validation (compile-time)
weaver registry check -r registry/

# Live telemetry validation (runtime)
weaver registry live-check --registry registry/
```

**Directory Structure**:
```
rust/knhk-workflow-engine/src/integration/
├── otel.rs                # OTEL integration (enhanced)
├── lockchain.rs           # Lockchain integration
└── weaver.rs              # Weaver validation (NEW)

rust/knhk-workflow-engine/src/provenance/
├── mod.rs
├── receipt.rs             # Receipt structure (NEW)
├── validation.rs          # Receipt validation (NEW)
└── merkle.rs              # Merkle proof generation

registry/                  # Weaver schemas
├── workflow-telemetry.yaml  # Workflow spans (NEW)
├── pattern-metrics.yaml     # Pattern metrics (NEW)
└── receipt-events.yaml      # Receipt events (NEW)
```

---

## Layer 5: MAPE-K Feedback

### Purpose

Close the autonomic loop: Monitor observations, Analyze gaps, Plan updates, Execute changes to ontology, persist Knowledge.

### Components

#### 5.1 Monitor

**File**: `rust/knhk-workflow-engine/src/mape/monitor.rs` (NEW)

Continuously monitor receipts and telemetry:

```rust
pub struct Monitor {
    receipt_stream: ReceiptStream,
    metrics_collector: MetricsCollector,
    anomaly_detector: AnomalyDetector,
}

impl Monitor {
    /// Monitor receipt stream for performance anomalies
    pub async fn monitor_receipts(&self) -> Result<Vec<Anomaly>> {
        let mut anomalies = Vec::new();

        while let Some(receipt) = self.receipt_stream.next().await {
            // Check Chatman constant violation
            if receipt.is_hot_path() && receipt.ticks > 8 {
                anomalies.push(Anomaly::ChatmanViolation {
                    case_id: receipt.case_id,
                    ticks: receipt.ticks,
                    pattern_id: receipt.pattern_id,
                });
            }

            // Check SLA violations
            if receipt.ticks > receipt.expected_ticks * 2 {
                anomalies.push(Anomaly::SLAViolation {
                    case_id: receipt.case_id,
                    actual: receipt.ticks,
                    expected: receipt.expected_ticks,
                });
            }

            // Detect statistical anomalies
            if let Some(anomaly) = self.anomaly_detector.detect(&receipt).await? {
                anomalies.push(anomaly);
            }
        }

        Ok(anomalies)
    }

    /// Collect metrics from OTEL
    pub async fn collect_metrics(&self) -> Result<MetricsSummary> {
        let metrics = self.metrics_collector.collect().await?;

        Ok(MetricsSummary {
            p50_latency: metrics.percentile(0.5),
            p95_latency: metrics.percentile(0.95),
            p99_latency: metrics.percentile(0.99),
            throughput: metrics.throughput(),
            error_rate: metrics.error_rate(),
        })
    }
}
```

#### 5.2 Analyze

**File**: `rust/knhk-workflow-engine/src/mape/analyze.rs` (NEW)

Analyze observations vs ontology:

```rust
pub struct Analyzer {
    ontology: Arc<OntologySnapshot>,
    statistical_analyzer: StatisticalAnalyzer,
}

impl Analyzer {
    /// Analyze gap between observations and ontology
    pub async fn analyze_gap(&self, observations: &[Receipt]) -> Result<AnalysisResult> {
        let mut gaps = Vec::new();

        for receipt in observations {
            // Compare actual vs expected ticks
            let expected = self.ontology.get_pattern_ticks(receipt.pattern_id)?;
            let actual = receipt.ticks;

            if actual > expected * 2 {
                gaps.push(Gap::PerformanceDegradation {
                    pattern_id: receipt.pattern_id,
                    expected,
                    actual,
                    degradation_factor: (actual as f64) / (expected as f64),
                });
            }
        }

        // Statistical analysis
        let stats = self.statistical_analyzer.analyze(observations)?;

        // Detect trends
        let trends = self.detect_trends(&stats)?;

        Ok(AnalysisResult {
            gaps,
            statistics: stats,
            trends,
            recommended_actions: self.recommend_actions(&gaps, &trends)?,
        })
    }

    /// Detect performance trends
    fn detect_trends(&self, stats: &Statistics) -> Result<Vec<Trend>> {
        let mut trends = Vec::new();

        // Detect latency increase
        if stats.latency_trend.slope > 0.1 {
            trends.push(Trend::LatencyIncrease {
                rate: stats.latency_trend.slope,
                pattern_ids: stats.latency_trend.affected_patterns.clone(),
            });
        }

        // Detect throughput decrease
        if stats.throughput_trend.slope < -0.1 {
            trends.push(Trend::ThroughputDecrease {
                rate: stats.throughput_trend.slope,
                workflows: stats.throughput_trend.affected_workflows.clone(),
            });
        }

        Ok(trends)
    }
}
```

#### 5.3 Plan

**File**: `rust/knhk-workflow-engine/src/mape/plan.rs` (NEW)

Generate ontology updates:

```rust
pub struct Planner {
    ontology: Arc<OntologySnapshot>,
    invariant_checker: InvariantChecker,
}

impl Planner {
    /// Plan ontology update to address gaps
    pub async fn plan_update(&self, analysis: &AnalysisResult) -> Result<OntologyUpdate> {
        let mut updates = Vec::new();

        for gap in &analysis.gaps {
            match gap {
                Gap::PerformanceDegradation { pattern_id, actual, .. } => {
                    // Plan: Update expected ticks for pattern
                    updates.push(Update::UpdatePatternTicks {
                        pattern_id: *pattern_id,
                        new_ticks: *actual,
                        reason: "Observed performance degradation".to_string(),
                    });
                }
            }
        }

        // Build ontology update
        let update = OntologyUpdate {
            version: self.ontology.version + 1,
            updates,
            timestamp: SystemTime::now(),
        };

        // Verify update preserves invariants (Σ ⊨ Q)
        if !self.invariant_checker.verify(&update).await? {
            return Err(Error::InvariantViolation);
        }

        Ok(update)
    }
}
```

#### 5.4 Execute

**File**: `rust/knhk-workflow-engine/src/mape/execute.rs` (NEW)

Apply ontology updates atomically:

```rust
pub struct Executor {
    ontology_manager: Arc<OntologyManager>,
    rollback_manager: RollbackManager,
}

impl Executor {
    /// Execute ontology update atomically
    pub async fn execute_update(&self, update: OntologyUpdate) -> Result<()> {
        // Save rollback point
        let rollback_snapshot = self.ontology_manager.current_snapshot();

        // Apply update
        match self.ontology_manager.apply_update(update.clone()).await {
            Ok(_) => {
                tracing::info!(
                    version = update.version,
                    "Ontology update applied successfully"
                );
                Ok(())
            }
            Err(e) => {
                // Rollback on failure
                tracing::error!(
                    error = %e,
                    "Ontology update failed, rolling back"
                );
                self.rollback_manager.rollback(rollback_snapshot).await?;
                Err(e)
            }
        }
    }
}
```

#### 5.5 Knowledge

**File**: `rust/knhk-workflow-engine/src/mape/knowledge.rs` (NEW)

Persist learned patterns:

```rust
pub struct KnowledgeBase {
    pattern_history: PatternHistory,
    learning_engine: LearningEngine,
    template_generator: Arc<TemplateGenerator>,
}

impl KnowledgeBase {
    /// Learn from execution history
    pub async fn learn_from_history(&self, receipts: &[Receipt]) -> Result<LearnedKnowledge> {
        // Analyze execution patterns
        let patterns = self.learning_engine.extract_patterns(receipts).await?;

        // Generate optimizations
        let optimizations = self.learning_engine.suggest_optimizations(&patterns).await?;

        // Update knowledge base
        self.pattern_history.record(patterns.clone()).await?;

        // Generate new templates for learned patterns
        for pattern in &patterns {
            let template = self.template_generator.generate_from_pattern(pattern).await?;
            self.persist_template(template).await?;
        }

        Ok(LearnedKnowledge {
            patterns,
            optimizations,
        })
    }
}
```

**Directory Structure**:
```
rust/knhk-workflow-engine/src/mape/
├── mod.rs
├── monitor.rs             # Monitor receipts/telemetry
├── analyze.rs             # Analyze O vs Σ
├── plan.rs                # Plan Σ updates
├── execute.rs             # Execute updates atomically
├── knowledge.rs           # Persist learned patterns
└── loop.rs                # MAPE-K loop orchestration
```

---

## Cross-Layer Integration

### Data Flow

```
1. Ontology → Projection
   Σ --[SPARQL]--> Templates --[Tera]--> Code/Workflows

2. Projection → Execution
   Generated Workflows --[Parse]--> WorkflowSpec --[Execute]--> μ

3. Execution → Observation
   μ --[Execute]--> Receipts + OTEL --[Validate]--> Weaver

4. Observation → Feedback
   Receipts --[Monitor]--> Anomalies --[Analyze]--> Gaps

5. Feedback → Ontology
   Gaps --[Plan]--> Updates --[Execute]--> Σ_{t+1}
```

### Integration Points

#### IP-1: Ontology → Projection

**Interface**: SPARQL Endpoint

```rust
pub trait OntologyProjection {
    async fn query_patterns(&self, category: &str) -> Result<Vec<Pattern>>;
    async fn query_workflows(&self, sector: &str) -> Result<Vec<Workflow>>;
    async fn query_invariants(&self) -> Result<Vec<Invariant>>;
}
```

#### IP-2: Projection → Execution

**Interface**: Workflow Parser

```rust
pub trait WorkflowProjection {
    async fn parse_workflow(&self, turtle: &str) -> Result<WorkflowSpec>;
    async fn generate_tests(&self, spec: &WorkflowSpec) -> Result<String>;
}
```

#### IP-3: Execution → Observation

**Interface**: Receipt Stream

```rust
pub trait ExecutionObservation {
    async fn emit_receipt(&self, receipt: ExecutionReceipt) -> Result<()>;
    async fn record_telemetry(&self, span: Span, metrics: Metrics) -> Result<()>;
}
```

#### IP-4: Observation → Feedback

**Interface**: Anomaly Stream

```rust
pub trait ObservationAnalysis {
    async fn stream_receipts(&self) -> ReceiptStream;
    async fn detect_anomalies(&self) -> Result<Vec<Anomaly>>;
}
```

#### IP-5: Feedback → Ontology

**Interface**: Ontology Update

```rust
pub trait FeedbackAdaptation {
    async fn plan_update(&self, analysis: AnalysisResult) -> Result<OntologyUpdate>;
    async fn apply_update(&self, update: OntologyUpdate) -> Result<()>;
}
```

---

## Snapshot System

### Ontology Versioning

```rust
pub struct OntologySnapshot {
    /// Snapshot version (monotonically increasing)
    pub version: u64,

    /// Creation timestamp
    pub timestamp: SystemTime,

    /// RDF graph store
    pub rdf_graph: Store,

    /// Invariant rules
    pub invariants: Vec<InvariantRule>,

    /// Parent snapshot hash (Merkle chain)
    pub parent_hash: Hash,

    /// Current snapshot hash
    pub snapshot_hash: Hash,
}

impl OntologySnapshot {
    /// Compute snapshot hash (URDNA2015 + SHA-256)
    pub fn compute_hash(&self) -> Result<Hash> {
        // Canonicalize RDF graph (URDNA2015)
        let canonical_rdf = self.rdf_graph.canonicalize()?;

        // Hash canonicalized RDF
        let mut hasher = Sha256::new();
        hasher.update(&canonical_rdf);
        hasher.update(&self.version.to_le_bytes());
        hasher.update(&self.parent_hash);

        Ok(Hash::from(hasher.finalize().as_slice()))
    }

    /// Validate invariants (Σ ⊨ Q)
    pub fn validate_invariants(&self) -> Result<bool> {
        for invariant in &self.invariants {
            if !invariant.check(&self.rdf_graph)? {
                tracing::error!(
                    invariant = %invariant.name,
                    "Invariant violation detected"
                );
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Create child snapshot with update
    pub fn create_child(&self, update: OntologyUpdate) -> Result<Self> {
        let mut child = Self {
            version: self.version + 1,
            timestamp: SystemTime::now(),
            rdf_graph: self.rdf_graph.clone(),
            invariants: self.invariants.clone(),
            parent_hash: self.snapshot_hash,
            snapshot_hash: Hash::default(),  // Computed below
        };

        // Apply update to child RDF graph
        child.apply_update(update)?;

        // Validate invariants
        if !child.validate_invariants()? {
            return Err(Error::InvariantViolation);
        }

        // Compute child hash
        child.snapshot_hash = child.compute_hash()?;

        Ok(child)
    }
}
```

### Atomic Updates

```rust
pub struct OntologyManager {
    current_snapshot: Arc<ArcSwap<OntologySnapshot>>,
    snapshot_history: Vec<OntologySnapshot>,
    storage: Arc<SnapshotStorage>,
}

impl OntologyManager {
    /// Apply update atomically (linearizable)
    pub async fn apply_update(&self, update: OntologyUpdate) -> Result<()> {
        // Load current snapshot
        let current = self.current_snapshot.load();

        // Create new snapshot
        let new_snapshot = current.create_child(update)?;

        // Atomically swap (linearizable point)
        self.current_snapshot.store(Arc::new(new_snapshot.clone()));

        // Persist to storage (async)
        self.storage.persist(&new_snapshot).await?;

        // Add to history
        self.snapshot_history.push(new_snapshot);

        Ok(())
    }

    /// Rollback to previous snapshot
    pub async fn rollback(&self, version: u64) -> Result<()> {
        let snapshot = self.snapshot_history
            .iter()
            .find(|s| s.version == version)
            .ok_or(Error::SnapshotNotFound)?;

        self.current_snapshot.store(Arc::new(snapshot.clone()));

        Ok(())
    }
}
```

---

## Receipt Structure

### Receipt Definition

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionReceipt {
    /// Unique receipt ID
    pub id: ReceiptId,

    /// Case ID
    pub case_id: CaseId,

    /// Workflow specification ID
    pub spec_id: WorkflowSpecId,

    /// Pattern ID (1-43)
    pub pattern_id: u32,

    /// Action hash: hash(A) where A = μ(O)
    pub action_hash: Hash,

    /// Execution time in CPU ticks
    pub ticks: u64,

    /// OTEL span ID
    pub span_id: SpanId,

    /// OTEL trace ID
    pub trace_id: TraceId,

    /// Unix timestamp (nanoseconds)
    pub timestamp: u64,

    /// 8-beat cycle ID
    pub cycle_id: u64,

    /// Shard identifier
    pub shard_id: String,

    /// Merkle proof (lockchain)
    pub merkle_proof: Option<MerkleProof>,

    /// Signature (optional)
    pub signature: Option<Signature>,
}
```

### Receipt Validation Chain

```rust
pub struct ReceiptValidator {
    ontology: Arc<OntologySnapshot>,
    weaver: Arc<WeaverValidator>,
}

impl ReceiptValidator {
    /// Validate receipt (multi-level)
    pub async fn validate(&self, receipt: &ExecutionReceipt) -> Result<ValidationResult> {
        let mut checks = Vec::new();

        // Level 1: Structural validation
        checks.push(self.validate_structure(receipt)?);

        // Level 2: Ontology validation (O ⊨ Σ)
        checks.push(self.validate_against_ontology(receipt).await?);

        // Level 3: Weaver schema validation
        checks.push(self.validate_against_weaver(receipt).await?);

        // Level 4: Cryptographic validation
        if let Some(ref proof) = receipt.merkle_proof {
            checks.push(self.validate_merkle_proof(receipt, proof)?);
        }

        // Level 5: Signature validation
        if let Some(ref sig) = receipt.signature {
            checks.push(self.validate_signature(receipt, sig)?);
        }

        Ok(ValidationResult {
            valid: checks.iter().all(|c| c.passed),
            checks,
        })
    }

    /// Validate against ontology (O ⊨ Σ)
    async fn validate_against_ontology(&self, receipt: &ExecutionReceipt) -> Result<ValidationCheck> {
        // Pattern exists in ontology
        let pattern_exists = self.ontology.has_pattern(receipt.pattern_id)?;
        if !pattern_exists {
            return Ok(ValidationCheck {
                name: "ontology.pattern_exists",
                passed: false,
                message: format!("Pattern {} not found in ontology", receipt.pattern_id),
            });
        }

        // Ticks within expected range
        let expected_ticks = self.ontology.get_pattern_ticks(receipt.pattern_id)?;
        let ticks_valid = receipt.ticks <= expected_ticks * 3;  // 3x tolerance

        Ok(ValidationCheck {
            name: "ontology.ticks_valid",
            passed: ticks_valid,
            message: format!(
                "Expected: {} ticks, Actual: {} ticks",
                expected_ticks, receipt.ticks
            ),
        })
    }

    /// Validate against Weaver schema
    async fn validate_against_weaver(&self, receipt: &ExecutionReceipt) -> Result<ValidationCheck> {
        self.weaver.validate_receipt(receipt).await
    }
}
```

---

## Directory Structure

```
/home/user/knhk/
├── ontology/                           # Layer 1: Ontology (Σ)
│   ├── knhk.owl.ttl                    # Core ontology (existing)
│   ├── yawl.ttl                        # YAWL patterns (existing)
│   ├── mape-k.ttl                      # MAPE-K policies (NEW)
│   ├── invariants.ttl                  # Invariants Q (NEW)
│   ├── sectors/                        # Domain ontologies (NEW)
│   │   ├── finance.ttl
│   │   ├── manufacturing.ttl
│   │   └── healthcare.ttl
│   └── snapshots/                      # Versioned snapshots (NEW)
│       ├── snapshot-v1.ttl
│       ├── snapshot-v2.ttl
│       └── current -> snapshot-v2.ttl
│
├── templates/                          # Layer 2: Projection (Π)
│   ├── workflow.tmpl                   # Workflow generation
│   ├── tests.tmpl                      # Test generation
│   ├── docs.tmpl                       # Documentation generation
│   ├── pattern_executor.rs.tmpl       # Code generation
│   └── README.md
│
├── rust/knhk-workflow-engine/         # Layer 3: Execution (μ)
│   ├── src/
│   │   ├── engine.rs                  # Core engine (enhanced)
│   │   ├── ggen/                      # Projection layer integration
│   │   │   ├── mod.rs
│   │   │   ├── sparql.rs              # SPARQL queries (NEW)
│   │   │   ├── templates.rs           # Template engine (enhanced)
│   │   │   ├── codegen.rs             # Code generation (NEW)
│   │   │   └── cache.rs               # Query caching (NEW)
│   │   ├── hooks/                     # Hook system (NEW)
│   │   │   ├── mod.rs
│   │   │   ├── manager.rs
│   │   │   ├── pre_task.rs
│   │   │   ├── post_task.rs
│   │   │   └── session.rs
│   │   ├── patterns/                  # Van der Aalst patterns
│   │   │   ├── mod.rs
│   │   │   ├── executor.rs            # Pattern trait
│   │   │   ├── basic_control.rs       # Patterns 1-5
│   │   │   ├── advanced_branch.rs     # Patterns 6-11
│   │   │   ├── multiple_instance.rs   # Patterns 12-15
│   │   │   ├── state_based.rs         # Patterns 16-18
│   │   │   ├── cancellation.rs        # Patterns 19-25
│   │   │   ├── advanced_control.rs    # Patterns 26-39
│   │   │   └── trigger.rs             # Patterns 40-43
│   │   ├── integration/               # Layer 4: Observation (O)
│   │   │   ├── otel.rs                # OTEL integration (enhanced)
│   │   │   ├── lockchain.rs           # Lockchain integration
│   │   │   └── weaver.rs              # Weaver validation (NEW)
│   │   ├── provenance/                # Receipt system (NEW)
│   │   │   ├── mod.rs
│   │   │   ├── receipt.rs             # Receipt structure
│   │   │   ├── validation.rs          # Receipt validation
│   │   │   └── merkle.rs              # Merkle proof
│   │   └── mape/                      # Layer 5: MAPE-K Feedback (NEW)
│   │       ├── mod.rs
│   │       ├── monitor.rs             # Monitor receipts
│   │       ├── analyze.rs             # Analyze O vs Σ
│   │       ├── plan.rs                # Plan Σ updates
│   │       ├── execute.rs             # Execute updates
│   │       ├── knowledge.rs           # Knowledge base
│   │       └── loop.rs                # MAPE-K loop
│   └── tests/
│       ├── self_executing/            # Self-executing workflow tests (NEW)
│       │   ├── mod.rs
│       │   ├── monitor_test.rs
│       │   ├── analyze_test.rs
│       │   ├── plan_test.rs
│       │   ├── execute_test.rs
│       │   └── integration_test.rs
│       └── chicago_tdd_tools_integration.rs
│
├── registry/                           # Weaver schemas (Layer 4)
│   ├── workflow-telemetry.yaml        # Workflow spans (NEW)
│   ├── pattern-metrics.yaml           # Pattern metrics (NEW)
│   └── receipt-events.yaml            # Receipt events (NEW)
│
└── docs/architecture/                  # Architecture documentation
    ├── self-executing-workflows/       # This specification (NEW)
    │   ├── architecture.md             # This document
    │   ├── data-flow.puml              # Data flow diagram
    │   ├── c4-layer-integration.puml   # C4 diagrams
    │   └── sequence-mape-k.puml        # MAPE-K sequence
    └── ... (existing architecture docs)
```

---

## Validation Strategy

### Weaver as Source of Truth

Per KNHK principles, **Weaver validation is the only trusted validation**:

```bash
# Level 1: Weaver Schema Validation (MANDATORY - Source of Truth)
weaver registry check -r registry/                    # Validate schema definition
weaver registry live-check --registry registry/       # Validate runtime telemetry

# Level 2: Compilation & Code Quality (Baseline)
cargo build --release                                 # Must compile
cargo clippy --workspace -- -D warnings               # Zero warnings

# Level 3: Traditional Tests (Supporting Evidence - Can Have False Positives)
cargo test --workspace                                # Rust unit tests
cargo test --test self_executing_integration          # Integration tests
```

**Critical**: If Weaver validation fails, the feature DOES NOT WORK, regardless of test results.

### Definition of Done for Self-Executing Workflows

Before ANY code is production-ready, ALL must be true:

#### Build & Code Quality (Baseline)
- [ ] `cargo build --workspace` succeeds with zero warnings
- [ ] `cargo clippy --workspace -- -D warnings` shows zero issues
- [ ] No `.unwrap()` or `.expect()` in production code paths
- [ ] Proper `Result<T, E>` error handling

#### Weaver Validation (MANDATORY - Source of Truth)
- [ ] **`weaver registry check -r registry/` passes** (schema is valid)
- [ ] **`weaver registry live-check --registry registry/` passes** (runtime telemetry conforms to schema)
- [ ] All workflow execution spans defined in `registry/workflow-telemetry.yaml`
- [ ] All pattern metrics defined in `registry/pattern-metrics.yaml`
- [ ] All receipt events defined in `registry/receipt-events.yaml`

#### Functional Validation (MANDATORY - Must Actually Execute)
- [ ] **MAPE-K loop executes end-to-end** (Monitor → Analyze → Plan → Execute → Knowledge)
- [ ] **Ontology updates applied atomically** (snapshot versioning works)
- [ ] **Receipts generated and validated** (O ⊨ Σ)
- [ ] **Performance constraints met** (≤8 ticks for hot path)
- [ ] **Idempotence verified** (μ ∘ μ = μ)

#### Traditional Testing (Supporting Evidence Only)
- [ ] `cargo test --workspace` passes completely
- [ ] `cargo test --test self_executing_integration` passes
- [ ] All 43 Van der Aalst patterns executable
- [ ] SPARQL queries return expected results
- [ ] Template generation produces valid code

---

## Performance Guarantees

### Chatman Constant (8 Ticks)

**Hot path operations** (pattern execution) MUST complete in ≤8 CPU ticks:

```rust
// Enforced at runtime
let start_ticks = rdtsc();
let action = self.execute_pattern(&task, &case).await?;
let end_ticks = rdtsc();
let ticks = end_ticks - start_ticks;

// Verify Chatman constant
if case.is_hot_path() && ticks > 8 {
    // Log warning, trigger MAPE-K feedback
    self.mape_loop.report_violation(ChatmanViolation {
        case_id,
        pattern_id,
        ticks
    }).await?;
}
```

### Three-Tier Path SLOs

```
Hot Path:  ≤8 ticks    (P99)  - Pattern execution
Warm Path: ≤500ms      (P99)  - Batch operations
Cold Path: ≤5s         (P95)  - Complex SPARQL queries
```

### MAPE-K Loop Latency

```
Monitor:  ≤100ms  - Receipt stream processing
Analyze:  ≤500ms  - Statistical analysis
Plan:     ≤1s     - Ontology update planning
Execute:  ≤2s     - Atomic snapshot swap
Total:    ≤4s     - End-to-end feedback latency
```

---

## Implementation Roadmap

### Phase 1: Foundation (Weeks 1-2)

**Goal**: Establish ontology snapshots and basic SPARQL projection

**Deliverables**:
1. Ontology snapshot system with versioning
2. SPARQL query interface for patterns
3. Basic template engine integration
4. Receipt structure and generation

**Success Criteria**:
- Ontology snapshots persist correctly
- SPARQL queries return expected patterns
- Templates generate valid Turtle workflows
- Receipts include all required fields

### Phase 2: Execution Integration (Weeks 3-4)

**Goal**: Connect execution layer to observation layer

**Deliverables**:
1. Hook manager implementation
2. Enhanced pattern executors with receipts
3. OTEL integration with Weaver schemas
4. Receipt validation chain

**Success Criteria**:
- Hooks execute pre/post-task correctly
- Patterns generate receipts with ≤8 ticks
- Weaver validation passes for all receipts
- Receipt validation chain works end-to-end

### Phase 3: MAPE-K Loop (Weeks 5-6)

**Goal**: Close the autonomic feedback loop

**Deliverables**:
1. Monitor implementation (receipt stream)
2. Analyzer (O vs Σ comparison)
3. Planner (ontology update generation)
4. Executor (atomic snapshot updates)
5. Knowledge base (pattern learning)

**Success Criteria**:
- MAPE-K loop executes end-to-end
- Ontology updates preserve invariants
- Knowledge base learns from history
- Feedback latency ≤4s

### Phase 4: Validation & Optimization (Weeks 7-8)

**Goal**: Ensure production readiness

**Deliverables**:
1. Comprehensive Weaver schemas
2. Chicago TDD test suite
3. Performance benchmarks
4. Documentation and runbooks

**Success Criteria**:
- All Weaver validations pass
- Performance meets SLOs (≤8 ticks, ≤500ms, ≤5s)
- Tests cover all 43 patterns
- Documentation complete and accurate

---

## Conclusion

This architecture establishes a complete self-executing workflow system where:

1. **Ontology (Σ)** defines the source of truth (YAWL patterns, MAPE-K policies, invariants)
2. **Projection (Π)** transforms ontology into executable code via SPARQL and templates
3. **Execution (μ)** runs workflows deterministically with hook-based coordination
4. **Observation (O)** collects telemetry and receipts, validated by Weaver
5. **MAPE-K Feedback** closes the loop, adapting the ontology based on runtime observations

**Key Properties**:
- **Deterministic**: A = μ(O)
- **Idempotent**: μ ∘ μ = μ
- **Consistent**: O ⊨ Σ, Σ ⊨ Q
- **Performant**: latency(μ) ≤ 8 ticks
- **Adaptive**: Σ_t → Σ_{t+1} via MAPE-K

**Validation**: Weaver as the single source of truth ensures no false positives.

**Next Steps**: Implement Phase 1 (Foundation) to establish ontology snapshots and SPARQL projection.

---

**Document Version**: 1.0
**Last Updated**: 2025-11-16
**Status**: Design Specification
**Review Status**: Pending Architecture Review
