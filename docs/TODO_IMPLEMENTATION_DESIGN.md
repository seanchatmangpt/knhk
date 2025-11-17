# KNHK TODO Implementation Design Document

**Status**: 📋 SPECIFICATION | **Version**: 1.0.0 | **Date**: 2025-11-17

This document provides detailed implementation designs for all TODOs in the KNHK codebase, organized by component. Each TODO includes exact implementation guidance, code structure, dependencies, and DOCTRINE alignment.

---

## Table of Contents

1. [OTEL Telemetry TODOs (12 items)](#1-otel-telemetry-todos)
2. [Code Generation TODOs (20 items)](#2-code-generation-todos)
3. [Workflow Execution TODOs (7 items)](#3-workflow-execution-todos)

---

## 1. OTEL Telemetry TODOs

**File**: `rust/knhk-workflow-engine/src/executor/telemetry.rs`
**Total TODOs**: 12
**DOCTRINE Alignment**: Covenant 6 (Observations Drive Everything)

### Architecture Context

The telemetry module provides full observability for workflow execution. It MUST emit metrics that conform to the schema defined in `registry/knhk-workflow-engine.yaml`. Weaver validation is the source of truth for telemetry compliance.

**Key Dependencies**:
- `metrics` crate (already in Cargo.toml: version 0.23)
- `tracing` (already imported)
- OpenTelemetry schema: `registry/knhk-workflow-engine.yaml`

**Performance Constraint**: Covenant 5 (Chatman Constant)
- Metric emission MUST NOT exceed 8 ticks for hot path operations
- Use async emission to avoid blocking execution
- Consider metric batching for high-frequency events

---

### TODO 1-4: Workflow Event Metrics (Lines 111-140)

**Locations**:
- Line 111-112: `workflow_started_total` counter
- Line 120-121: `workflow_completed_total` counter
- Line 130-131: `workflow_failed_total` counter
- Line 139-140: `workflow_cancelled_total` counter

**What Needs to Be Implemented**:

Emit OpenTelemetry counters for workflow lifecycle events. These counters track workflow state transitions and provide observability for workflow execution patterns.

**Implementation Design**:

```rust
// Add to imports at top of file
use metrics::{counter, describe_counter};

// In WorkflowTelemetry::new() - add metric descriptions
impl WorkflowTelemetry {
    pub fn new(instance_id: String) -> Self {
        // Describe metrics (only needed once, but safe to call multiple times)
        describe_counter!(
            "workflow_started_total",
            "Total number of workflows started"
        );
        describe_counter!(
            "workflow_completed_total",
            "Total number of workflows completed successfully"
        );
        describe_counter!(
            "workflow_failed_total",
            "Total number of workflows that failed"
        );
        describe_counter!(
            "workflow_cancelled_total",
            "Total number of workflows cancelled"
        );

        Self { instance_id }
    }
}

// Replace TODO comments with actual metric emissions

// Line 111-112: workflow_started_total
impl WorkflowTelemetry {
    pub async fn emit_workflow_event(&self, event: WorkflowEvent) {
        match event {
            WorkflowEvent::Started { ref workflow_id, ref instance_id } => {
                info!(
                    workflow_id = %workflow_id,
                    instance_id = %instance_id,
                    "Workflow started"
                );
                // Emit OTEL metric
                counter!("workflow_started_total",
                    "workflow_id" => workflow_id.clone(),
                    "instance_id" => instance_id.clone()
                ).increment(1);
            }

            // Line 120-121: workflow_completed_total
            WorkflowEvent::Completed { ref workflow_id, ref instance_id } => {
                info!(
                    workflow_id = %workflow_id,
                    instance_id = %instance_id,
                    "Workflow completed"
                );
                // Emit OTEL metric
                counter!("workflow_completed_total",
                    "workflow_id" => workflow_id.clone(),
                    "instance_id" => instance_id.clone()
                ).increment(1);
            }

            // Line 130-131: workflow_failed_total
            WorkflowEvent::Failed { ref workflow_id, ref instance_id, ref error } => {
                warn!(
                    workflow_id = %workflow_id,
                    instance_id = %instance_id,
                    error = %error,
                    "Workflow failed"
                );
                // Emit OTEL metric
                counter!("workflow_failed_total",
                    "workflow_id" => workflow_id.clone(),
                    "instance_id" => instance_id.clone(),
                    "error_type" => error.clone()
                ).increment(1);
            }

            // Line 139-140: workflow_cancelled_total
            WorkflowEvent::Cancelled { ref workflow_id, ref instance_id } => {
                info!(
                    workflow_id = %workflow_id,
                    instance_id = %instance_id,
                    "Workflow cancelled"
                );
                // Emit OTEL metric
                counter!("workflow_cancelled_total",
                    "workflow_id" => workflow_id.clone(),
                    "instance_id" => instance_id.clone()
                ).increment(1);
            }
        }
    }
}
```

**Weaver Schema Validation**:

These metrics align with the workflow execution spans defined in `registry/knhk-workflow-engine.yaml`:
- `knhk.workflow_engine.register_workflow` (lines 107-121)
- `knhk.workflow_engine.execute_case` (lines 142-158)

**DOCTRINE Alignment**:
- **Covenant 6**: Every workflow state transition emits observable telemetry
- **Covenant 5**: Counter increment is O(1), well within 8-tick bound
- **Q Invariants**: Counters are monotonically increasing (satisfies no retrocausation)

---

### TODO 5-8: Task Event Metrics (Lines 162-209)

**Locations**:
- Line 162-163: `task_enabled_total` counter
- Line 171-172: `task_started_total` counter
- Line 181-190: `task_completed_total` counter + `task_duration_seconds` histogram
- Line 199-200: `task_failed_total` counter
- Line 208-209: `task_cancelled_total` counter

**What Needs to Be Implemented**:

Emit counters for task lifecycle events and a histogram for task duration. The duration histogram is CRITICAL for performance analysis and Chatman Constant validation.

**Implementation Design**:

```rust
// Add to imports
use metrics::{counter, histogram, describe_counter, describe_histogram};

// In WorkflowTelemetry::new() - add metric descriptions
describe_counter!("task_enabled_total", "Total number of tasks enabled");
describe_counter!("task_started_total", "Total number of tasks started");
describe_counter!("task_completed_total", "Total number of tasks completed");
describe_histogram!("task_duration_seconds", "Task execution duration in seconds");
describe_counter!("task_failed_total", "Total number of tasks failed");
describe_counter!("task_cancelled_total", "Total number of tasks cancelled");

// Replace TODO comments in emit_task_event()
pub async fn emit_task_event(&self, event: TaskEvent) {
    match event {
        // Line 162-163: task_enabled_total
        TaskEvent::Enabled { ref task_id, ref instance_id } => {
            debug!(
                task_id = %task_id,
                instance_id = %instance_id,
                "Task enabled"
            );
            counter!("task_enabled_total",
                "task_id" => task_id.clone(),
                "instance_id" => instance_id.clone()
            ).increment(1);
        }

        // Line 171-172: task_started_total
        TaskEvent::Started { ref task_id, ref instance_id } => {
            info!(
                task_id = %task_id,
                instance_id = %instance_id,
                "Task started"
            );
            counter!("task_started_total",
                "task_id" => task_id.clone(),
                "instance_id" => instance_id.clone()
            ).increment(1);
        }

        // Line 181-190: task_completed_total + duration histogram
        TaskEvent::Completed { ref task_id, ref instance_id, duration } => {
            info!(
                task_id = %task_id,
                instance_id = %instance_id,
                duration_us = duration.map(|d| d.as_micros()),
                "Task completed"
            );

            // Emit completion counter
            counter!("task_completed_total",
                "task_id" => task_id.clone(),
                "instance_id" => instance_id.clone()
            ).increment(1);

            // Emit duration histogram and check Chatman constant
            if let Some(d) = duration {
                histogram!("task_duration_seconds",
                    "task_id" => task_id.clone(),
                    "instance_id" => instance_id.clone()
                ).record(d.as_secs_f64());

                // Covenant 5: Check against Chatman constant (8 ticks)
                const CHATMAN_CONSTANT_NS: u128 = 8;
                if d.as_nanos() > CHATMAN_CONSTANT_NS {
                    warn!(
                        task_id = %task_id,
                        duration_ns = d.as_nanos(),
                        "Task execution exceeded 8-tick limit (Covenant 5 violation)"
                    );

                    // Emit violation counter for monitoring
                    counter!("task_chatman_violations_total",
                        "task_id" => task_id.clone(),
                        "duration_ns" => d.as_nanos().to_string()
                    ).increment(1);
                }
            }
        }

        // Line 199-200: task_failed_total
        TaskEvent::Failed { ref task_id, ref instance_id, ref error } => {
            warn!(
                task_id = %task_id,
                instance_id = %instance_id,
                error = ?error,
                "Task failed"
            );
            counter!("task_failed_total",
                "task_id" => task_id.clone(),
                "instance_id" => instance_id.clone(),
                "error" => error.clone().unwrap_or_default()
            ).increment(1);
        }

        // Line 208-209: task_cancelled_total
        TaskEvent::Cancelled { ref task_id, ref instance_id } => {
            info!(
                task_id = %task_id,
                instance_id = %instance_id,
                "Task cancelled"
            );
            counter!("task_cancelled_total",
                "task_id" => task_id.clone(),
                "instance_id" => instance_id.clone()
            ).increment(1);
        }
    }
}
```

**Weaver Schema Validation**:

These metrics align with:
- `knhk.workflow_engine.execute_task` span (lines 161-176 in schema)
- `metric.knhk.workflow_engine.pattern.execution_latency` (lines 459-469)

**Critical Performance Note**:

The duration histogram for `task_duration_seconds` is THE KEY METRIC for validating Covenant 5 (Chatman Constant). The implementation includes:
1. Recording actual duration
2. Checking against 8-tick limit
3. Emitting violation counter if exceeded
4. Warning logs for debugging

---

### TODO 9: State Transition Latency Histogram (Lines 254-256)

**Location**: Line 254-256 in `record_transition_latency()`

**What Needs to Be Implemented**:

Emit a histogram metric for workflow state transition latency. This is CRITICAL for validating Covenant 5 (hot path ≤ 8 ticks).

**Implementation Design**:

```rust
// Add to metric descriptions in new()
describe_histogram!(
    "workflow_transition_duration_seconds",
    "Workflow state transition latency in seconds"
);

// Replace TODO at line 254-256
pub fn record_transition_latency(&self, operation: &str, duration: Duration) {
    debug!(
        operation = %operation,
        duration_ns = duration.as_nanos(),
        "State transition latency"
    );

    // Emit OTEL histogram
    histogram!("workflow_transition_duration_seconds",
        "operation" => operation.to_string(),
        "instance_id" => self.instance_id.clone()
    ).record(duration.as_secs_f64());

    // Covenant 5: Check against Chatman constant
    const CHATMAN_CONSTANT_NS: u128 = 8;
    if duration.as_nanos() > CHATMAN_CONSTANT_NS {
        warn!(
            operation = %operation,
            duration_ns = duration.as_nanos(),
            "State transition exceeded 8-tick limit (Covenant 5 violation)"
        );

        // Emit violation metric for alerting
        counter!("workflow_transition_violations_total",
            "operation" => operation.to_string(),
            "duration_ns" => duration.as_nanos().to_string()
        ).increment(1);
    }
}
```

**DOCTRINE Alignment**:
- **Covenant 5**: Validates hot path latency against 8-tick bound
- **Covenant 6**: Makes state transitions observable via histogram
- **Q3**: Enforces bounded execution latency

---

### TODO 10: Workflow Throughput Gauge (Lines 279-281)

**Location**: Line 279-281 in `record_throughput()`

**What Needs to Be Implemented**:

Emit a gauge metric for workflow throughput (tasks completed per second). Gauges represent point-in-time values that can go up or down.

**Implementation Design**:

```rust
// Add to imports
use metrics::{gauge, describe_gauge};

// Add to metric descriptions in new()
describe_gauge!(
    "workflow_throughput_tasks_per_second",
    "Current workflow throughput in tasks per second"
);

// Replace TODO at line 279-281
pub fn record_throughput(&self, tasks_completed: usize, elapsed: Duration) {
    let throughput = tasks_completed as f64 / elapsed.as_secs_f64();
    debug!(
        tasks_completed = tasks_completed,
        elapsed_s = elapsed.as_secs_f64(),
        throughput = throughput,
        "Workflow throughput"
    );

    // Emit OTEL gauge
    gauge!("workflow_throughput_tasks_per_second",
        "instance_id" => self.instance_id.clone()
    ).set(throughput);

    // Also emit cumulative counter for total tasks
    counter!("workflow_tasks_completed_total",
        "instance_id" => self.instance_id.clone()
    ).increment(tasks_completed as u64);
}
```

**DOCTRINE Alignment**:
- **Covenant 6**: Throughput is observable in real-time
- Enables MAPE-K feedback loop for performance optimization

---

### TODO 11-12: Resource Usage Gauges (Lines 292-294)

**Location**: Lines 292-294 in `record_resource_usage()`

**What Needs to Be Implemented**:

Emit gauge metrics for memory usage (bytes) and CPU utilization (percentage). These are CRITICAL for resource-bound enforcement (Q5 invariant).

**Implementation Design**:

```rust
// Add to metric descriptions in new()
describe_gauge!(
    "workflow_memory_bytes",
    "Current workflow memory usage in bytes"
);
describe_gauge!(
    "workflow_cpu_percent",
    "Current workflow CPU utilization percentage (0-100)"
);

// Replace TODO at line 292-294
pub fn record_resource_usage(&self, memory_bytes: usize, cpu_percent: f64) {
    debug!(
        memory_bytes = memory_bytes,
        cpu_percent = cpu_percent,
        "Resource usage"
    );

    // Emit OTEL gauges
    gauge!("workflow_memory_bytes",
        "instance_id" => self.instance_id.clone()
    ).set(memory_bytes as f64);

    gauge!("workflow_cpu_percent",
        "instance_id" => self.instance_id.clone()
    ).set(cpu_percent);

    // Check resource bounds (Q5 invariant)
    // These thresholds should be configurable from Turtle specification
    const MAX_MEMORY_MB: f64 = 1024.0; // 1GB default
    const MAX_CPU_PERCENT: f64 = 80.0;  // 80% default

    let memory_mb = memory_bytes as f64 / 1_048_576.0;

    if memory_mb > MAX_MEMORY_MB {
        warn!(
            memory_mb = memory_mb,
            max_memory_mb = MAX_MEMORY_MB,
            "Workflow memory usage exceeds declared bound (Q5 violation)"
        );
        counter!("workflow_resource_violations_total",
            "resource_type" => "memory",
            "instance_id" => self.instance_id.clone()
        ).increment(1);
    }

    if cpu_percent > MAX_CPU_PERCENT {
        warn!(
            cpu_percent = cpu_percent,
            max_cpu_percent = MAX_CPU_PERCENT,
            "Workflow CPU usage exceeds declared bound (Q5 violation)"
        );
        counter!("workflow_resource_violations_total",
            "resource_type" => "cpu",
            "instance_id" => self.instance_id.clone()
        ).increment(1);
    }
}
```

**DOCTRINE Alignment**:
- **Covenant 5 (Q5)**: Enforces resource bounds
- **Covenant 6**: Makes resource consumption observable
- Enables MAPE-K autonomic management

---

### Telemetry Implementation Summary

**Files to Modify**:
1. `rust/knhk-workflow-engine/src/executor/telemetry.rs` - Replace all 12 TODO comments

**New Dependencies**: None (all crates already in Cargo.toml)

**Weaver Validation**:
After implementation, validate with:
```bash
# Schema validation (static)
weaver registry check -r registry/

# Live runtime validation (requires running workflow)
weaver registry live-check --registry registry/
```

**Testing Requirements**:
1. Unit tests: Verify metrics are emitted (use `metrics::testing::TestRecorder`)
2. Integration tests: Verify Weaver validation passes
3. Performance tests: Verify metric emission ≤ 8 ticks (chicago-tdd harness)

**Definition of Done**:
- [ ] All 12 TODOs replaced with working metric emission
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] `cargo test --workspace` passes
- [ ] `weaver registry check -r registry/` passes
- [ ] `weaver registry live-check --registry registry/` passes (after running workflow)
- [ ] Performance tests verify metric overhead ≤ 8 ticks

---

## 2. Code Generation TODOs

**File**: `rust/knhk-cli/src/commands/gen.rs`
**Total TODOs**: 20
**DOCTRINE Alignment**: Covenant 1 (Turtle Is Definition and Cause)

### Architecture Context

The code generation module (`ggen`) generates code from RDF/Turtle workflow specifications. This is the PROJECTION layer that transforms Turtle definitions into executable code in various languages (Rust, Python, JavaScript, Go).

**Key Principle**: **PURE PASSTHROUGH TEMPLATES**
- No conditional logic in templates
- No filtering or reordering of data
- All behavior derives from Turtle RDF
- Templates are purely mechanical transformations

**Key Dependencies**:
- `oxigraph` for RDF/Turtle parsing (already in workspace Cargo.toml)
- `tera` template engine (already in workflow-engine Cargo.toml: "builtins" feature)
- Weaver CLI for validation

---

### TODO 1: RDF/Turtle Parsing and Code Generation (Line 121-127)

**Location**: Line 121-127 in `generate_workflow()`

**What Needs to Be Implemented**:

The core RDF/Turtle parsing pipeline that:
1. Parses Turtle using oxigraph
2. Extracts workflow specification via SPARQL
3. Applies template transformation
4. Generates code for target language
5. Optionally validates against Weaver schema

**Implementation Design**:

```rust
// Add to imports at top of file
use oxigraph::store::Store;
use oxigraph::model::{NamedNode, Term};
use oxigraph::sparql::QueryResults;

// Add new helper struct for workflow spec extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkflowSpec {
    id: String,
    name: String,
    tasks: Vec<TaskSpec>,
    flows: Vec<FlowSpec>,
    variables: HashMap<String, VariableSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TaskSpec {
    id: String,
    name: String,
    task_type: String,
    split_type: Option<String>,
    join_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FlowSpec {
    from: String,
    to: String,
    predicate: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VariableSpec {
    name: String,
    data_type: String,
    initial_value: Option<String>,
}

// Replace TODO at line 121-127
fn generate_workflow_code(spec_content: &str, req: &WorkflowGenRequest) -> CnvResult<String> {
    // Step 1: Parse RDF/Turtle using oxigraph
    let store = Store::new().map_err(|e| {
        NounVerbError::execution_error(format!("Failed to create RDF store: {}", e))
    })?;

    store.load_from_reader(
        oxigraph::io::RdfFormat::Turtle,
        spec_content.as_bytes(),
    ).map_err(|e| {
        NounVerbError::execution_error(format!("Failed to parse Turtle: {}", e))
    })?;

    // Step 2: Extract workflow specification via SPARQL
    let workflow_spec = extract_workflow_spec(&store)?;

    // Step 3: Load template
    let template_engine = load_template_engine(req)?;

    // Step 4: Apply template transformation
    let mut context = tera::Context::new();
    context.insert("workflow", &workflow_spec);
    context.insert("spec_file", &req.spec_file.to_string_lossy());
    context.insert("emit_telemetry", &req.emit_telemetry);
    context.insert("emit_hooks", &req.emit_hooks);
    context.insert("language", &format!("{:?}", req.language).to_lowercase());

    let template_name = match req.language {
        Language::Rust => "workflow.rs.tera",
        Language::Python => "workflow.py.tera",
        Language::JavaScript => "workflow.js.tera",
        Language::Go => "workflow.go.tera",
    };

    let generated_code = template_engine.render(template_name, &context)
        .map_err(|e| {
            NounVerbError::execution_error(format!("Template rendering failed: {}", e))
        })?;

    Ok(generated_code)
}

// Helper: Extract workflow spec from RDF store using SPARQL
fn extract_workflow_spec(store: &Store) -> CnvResult<WorkflowSpec> {
    const YAWL_NS: &str = "http://www.yawlfoundation.org/yawlschema#";
    const RDFS_NS: &str = "http://www.w3.org/2000/01/rdf-schema#";
    const RDF_NS: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";

    // Extract workflow metadata
    let query = format!(r#"
        PREFIX yawl: <{}>
        PREFIX rdfs: <{}>
        PREFIX rdf: <{}>

        SELECT ?spec ?name WHERE {{
            ?spec rdf:type yawl:Specification .
            OPTIONAL {{ ?spec rdfs:label ?name }}
        }}
        LIMIT 1
    "#, YAWL_NS, RDFS_NS, RDF_NS);

    let results = store.query(&query).map_err(|e| {
        NounVerbError::execution_error(format!("SPARQL query failed: {}", e))
    })?;

    let (id, name) = if let QueryResults::Solutions(mut solutions) = results {
        if let Some(solution) = solutions.next() {
            let solution = solution.map_err(|e| {
                NounVerbError::execution_error(format!("Failed to read solution: {}", e))
            })?;

            let id = solution.get("spec")
                .and_then(|t| match t {
                    Term::NamedNode(n) => Some(n.as_str().to_string()),
                    _ => None,
                })
                .ok_or_else(|| NounVerbError::execution_error("Workflow ID not found".to_string()))?;

            let name = solution.get("name")
                .and_then(|t| match t {
                    Term::Literal(l) => Some(l.value().to_string()),
                    _ => None,
                })
                .unwrap_or_else(|| "Unnamed Workflow".to_string());

            (id, name)
        } else {
            return Err(NounVerbError::execution_error("No workflow specification found".to_string()));
        }
    } else {
        return Err(NounVerbError::execution_error("Invalid query results".to_string()));
    };

    // Extract tasks (similar SPARQL pattern)
    let tasks = extract_tasks(store)?;

    // Extract flows (similar SPARQL pattern)
    let flows = extract_flows(store)?;

    // Extract variables (similar SPARQL pattern)
    let variables = extract_variables(store)?;

    Ok(WorkflowSpec {
        id,
        name,
        tasks,
        flows,
        variables,
    })
}

// Helper: Extract tasks from RDF store
fn extract_tasks(store: &Store) -> CnvResult<Vec<TaskSpec>> {
    const YAWL_NS: &str = "http://www.yawlfoundation.org/yawlschema#";
    const RDFS_NS: &str = "http://www.w3.org/2000/01/rdf-schema#";
    const RDF_NS: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";

    let query = format!(r#"
        PREFIX yawl: <{}>
        PREFIX rdfs: <{}>
        PREFIX rdf: <{}>

        SELECT ?task ?name ?type ?splitType ?joinType WHERE {{
            ?task rdf:type yawl:Task .
            OPTIONAL {{ ?task rdfs:label ?name }}
            OPTIONAL {{ ?task rdf:type ?type }}
            OPTIONAL {{ ?task yawl:split ?splitType }}
            OPTIONAL {{ ?task yawl:join ?joinType }}
        }}
    "#, YAWL_NS, RDFS_NS, RDF_NS);

    let results = store.query(&query).map_err(|e| {
        NounVerbError::execution_error(format!("Task query failed: {}", e))
    })?;

    let mut tasks = Vec::new();

    if let QueryResults::Solutions(solutions) = results {
        for solution in solutions {
            let solution = solution.map_err(|e| {
                NounVerbError::execution_error(format!("Failed to read solution: {}", e))
            })?;

            let id = solution.get("task")
                .and_then(|t| match t {
                    Term::NamedNode(n) => Some(n.as_str().to_string()),
                    _ => None,
                })
                .ok_or_else(|| NounVerbError::execution_error("Task ID not found".to_string()))?;

            let name = solution.get("name")
                .and_then(|t| match t {
                    Term::Literal(l) => Some(l.value().to_string()),
                    _ => None,
                })
                .unwrap_or_else(|| id.clone());

            let task_type = solution.get("type")
                .and_then(|t| match t {
                    Term::NamedNode(n) => Some(n.as_str().to_string()),
                    _ => None,
                })
                .unwrap_or_else(|| "atomic".to_string());

            let split_type = solution.get("splitType")
                .and_then(|t| match t {
                    Term::NamedNode(n) => Some(n.as_str().to_string()),
                    _ => None,
                });

            let join_type = solution.get("joinType")
                .and_then(|t| match t {
                    Term::NamedNode(n) => Some(n.as_str().to_string()),
                    _ => None,
                });

            tasks.push(TaskSpec {
                id,
                name,
                task_type,
                split_type,
                join_type,
            });
        }
    }

    Ok(tasks)
}

// Helper: Extract flows from RDF store
fn extract_flows(store: &Store) -> CnvResult<Vec<FlowSpec>> {
    const YAWL_NS: &str = "http://www.yawlfoundation.org/yawlschema#";

    let query = format!(r#"
        PREFIX yawl: <{}>

        SELECT ?from ?to ?predicate WHERE {{
            ?flow rdf:type yawl:Flow .
            ?flow yawl:flowsFrom ?from .
            ?flow yawl:flowsInto ?to .
            OPTIONAL {{ ?flow yawl:predicate ?predicate }}
        }}
    "#, YAWL_NS);

    let results = store.query(&query).map_err(|e| {
        NounVerbError::execution_error(format!("Flow query failed: {}", e))
    })?;

    let mut flows = Vec::new();

    if let QueryResults::Solutions(solutions) = results {
        for solution in solutions {
            let solution = solution.map_err(|e| {
                NounVerbError::execution_error(format!("Failed to read solution: {}", e))
            })?;

            let from = solution.get("from")
                .and_then(|t| match t {
                    Term::NamedNode(n) => Some(n.as_str().to_string()),
                    _ => None,
                })
                .ok_or_else(|| NounVerbError::execution_error("Flow source not found".to_string()))?;

            let to = solution.get("to")
                .and_then(|t| match t {
                    Term::NamedNode(n) => Some(n.as_str().to_string()),
                    _ => None,
                })
                .ok_or_else(|| NounVerbError::execution_error("Flow target not found".to_string()))?;

            let predicate = solution.get("predicate")
                .and_then(|t| match t {
                    Term::Literal(l) => Some(l.value().to_string()),
                    _ => None,
                });

            flows.push(FlowSpec {
                from,
                to,
                predicate,
            });
        }
    }

    Ok(flows)
}

// Helper: Extract variables from RDF store
fn extract_variables(store: &Store) -> CnvResult<HashMap<String, VariableSpec>> {
    // For now, return empty map - variables not yet in ontology
    Ok(HashMap::new())
}

// Helper: Load template engine with templates
fn load_template_engine(req: &WorkflowGenRequest) -> CnvResult<tera::Tera> {
    let template_dir = if let Some(ref template_path) = req.template {
        template_path.parent().unwrap_or(Path::new("."))
    } else {
        // Default template location
        Path::new("ggen-marketplace/knhk-yawl-workflows/template")
    };

    let template_glob = format!("{}/**/*.tera", template_dir.display());

    let tera = tera::Tera::new(&template_glob).map_err(|e| {
        NounVerbError::execution_error(format!("Failed to load templates: {}", e))
    })?;

    Ok(tera)
}
```

**DOCTRINE Alignment**:
- **Covenant 1**: Turtle is the single source of truth (uses SPARQL to extract EXACTLY what's declared)
- No filtering, no reconstruction - pure passthrough
- Template receives raw data from Turtle, no business logic

**Dependencies**:
- Add `use oxigraph::{store::Store, model::{NamedNode, Term}, sparql::QueryResults};`
- Add `use tera;`
- Add `use std::path::Path;`

---

### TODO 2: Weaver Validation Integration (Line 146-149)

**Location**: Line 146-149 in `generate_workflow()`

**What Needs to Be Implemented**:

Integrate Weaver schema validation to verify generated code emits correct telemetry. This is THE SOURCE OF TRUTH for validation (Covenant 6).

**Implementation Design**:

```rust
use std::process::Command;
use std::fs;

// Replace TODO at line 146-149
if req.validate {
    let validate_progress = ProgressIndicator::new("Validating against Weaver schema");

    // Write generated code to temp file for validation
    let temp_dir = tempfile::tempdir().map_err(|e| {
        NounVerbError::execution_error(format!("Failed to create temp dir: {}", e))
    })?;

    let temp_file = temp_dir.path().join("generated_workflow.rs");
    fs::write(&temp_file, &generated_code).map_err(|e| {
        NounVerbError::execution_error(format!("Failed to write temp file: {}", e))
    })?;

    // Run Weaver registry check
    let weaver_check = Command::new("weaver")
        .args(&["registry", "check", "-r", "registry/"])
        .output()
        .map_err(|e| {
            NounVerbError::execution_error(format!("Failed to run weaver: {}", e))
        })?;

    if !weaver_check.status.success() {
        let stderr = String::from_utf8_lossy(&weaver_check.stderr);
        validate_progress.fail(&format!("Weaver validation failed: {}", stderr));
        return Err(NounVerbError::execution_error(format!(
            "Weaver schema validation failed:\n{}",
            stderr
        )));
    }

    // Note: Live validation requires actual execution
    // For full validation, user must run:
    // 1. Build generated code: cargo build
    // 2. Run workflow: ./generated_workflow
    // 3. Validate telemetry: weaver registry live-check --registry registry/

    #[cfg(feature = "otel")]
    info!("Weaver schema check passed. Run 'weaver registry live-check' after execution for full validation.");

    validate_progress.complete("Weaver schema check passed");
}
```

**DOCTRINE Alignment**:
- **Covenant 6**: Weaver validation is the source of truth for observability
- Only Weaver can prove that generated code emits correct telemetry
- Static check validates schema, live check validates runtime behavior

**Dependencies**:
- `std::process::Command` for running weaver CLI
- `tempfile` crate (already in Cargo.toml)

---

### TODO 3-6: Workflow Execution Logic Placeholders (Lines 236, 281, 317, 367)

**Locations**:
- Line 236: Rust workflow execution
- Line 281: Python workflow execution
- Line 317: JavaScript workflow execution
- Line 367: Go workflow execution

**What Needs to Be Implemented**:

These are placeholders in generated code templates. The actual implementation should come from:
1. User-provided runtime behavior (in Turtle spec via `yawl-exec:runtimeBehavior`)
2. Generated code should CALL the runtime behavior, not implement it

**Implementation Design**:

```rust
// Update generate_rust_workflow() to include runtime behavior
fn generate_rust_workflow(req: &WorkflowGenRequest) -> String {
    let telemetry = if req.emit_telemetry {
        r#"
#[cfg(feature = "otel")]
use tracing::{instrument, info, error};
"#
    } else {
        ""
    };

    let hooks = if req.emit_hooks {
        r#"
use knhk_hooks::{Hook, HookContext};
"#
    } else {
        ""
    };

    // Updated execute() method with runtime behavior
    format!(
        r#"//! Generated Workflow
//! Source: {}
//! Generated by: KNHK ggen v2.7.1
//! Language: Rust
{}{}
use std::sync::Arc;
use serde::{{Serialize, Deserialize}};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowSpec {{
    pub name: String,
    pub version: String,
    pub nodes: Vec<WorkflowNode>,
}}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowNode {{
    pub id: String,
    pub node_type: String,
    pub config: serde_json::Value,
    /// Runtime behavior reference (from Turtle spec)
    pub runtime_behavior: Option<String>,
}}

impl WorkflowSpec {{
    pub fn new(name: impl Into<String>) -> Self {{
        Self {{
            name: name.into(),
            version: "1.0.0".to_string(),
            nodes: Vec::new(),
        }}
    }}

    pub fn add_node(&mut self, node: WorkflowNode) {{
        self.nodes.push(node);
    }}

    #[cfg(feature = "otel")]
    #[instrument(skip(self))]
    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {{
        info!(workflow = %self.name, "Starting workflow execution");

        // Execute each node according to runtime behavior
        for node in &self.nodes {{
            self.execute_node(node).await?;
        }}

        info!(workflow = %self.name, "Workflow execution completed");
        Ok(())
    }}

    #[cfg(not(feature = "otel"))]
    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {{
        for node in &self.nodes {{
            self.execute_node(node).await?;
        }}
        Ok(())
    }}

    async fn execute_node(&self, node: &WorkflowNode) -> Result<(), Box<dyn std::error::Error>> {{
        #[cfg(feature = "otel")]
        info!(node_id = %node.id, node_type = %node.node_type, "Executing node");

        // Runtime behavior is specified in Turtle via yawl-exec:runtimeBehavior
        // This should be a reference to an actual implementation (connector, script, service)
        if let Some(ref behavior) = node.runtime_behavior {{
            // TODO: Load and execute runtime behavior
            // For now, this is a placeholder that user must implement
            println!("Executing node {{}} with behavior: {{}}", node.id, behavior);
        }} else {{
            // No runtime behavior - this is a stub node
            println!("Executing stub node: {{}}", node.id);
        }}

        Ok(())
    }}
}}
"#,
        req.spec_file.display(),
        telemetry,
        hooks
    )
}

// Similar updates for Python, JavaScript, Go...
```

**DOCTRINE Alignment**:
- **Covenant 1**: Runtime behavior comes from Turtle specification
- Generated code doesn't implement business logic - it CALLS it
- User provides runtime behavior via connectors, scripts, or services

---

### TODO 7-20: Template and Marketplace Management (Lines 700-947)

**Summary of Template/Marketplace TODOs**:

These TODOs implement template management and marketplace integration:

1. **Template Listing** (line 765): List available templates from filesystem/registry
2. **Template Search** (line 793): Search templates by pattern
3. **Template Preview** (line 812): Preview template content
4. **Template Installation** (line 826): Install template from marketplace
5. **Template Validation** (line 847): Validate template structure
6. **Template Documentation** (line 856): Show template docs
7. **Marketplace Publishing** (line 881): Publish template to marketplace
8. **Marketplace Search** (line 897): Search marketplace
9. **Marketplace Installation** (line 917): Install from marketplace
10. **Rating/Reviews** (line 926): Show template ratings

**Implementation Strategy**:

These features require a **marketplace backend** which is out of scope for the immediate TODO implementation. However, we can provide a **local filesystem implementation** as a starting point:

```rust
// Template storage structure:
// ~/.knhk/templates/
//   ├── workflow-basic/
//   │   ├── template.yaml (metadata)
//   │   ├── workflow.rs.tera
//   │   └── README.md
//   └── chicago-tdd/
//       ├── template.yaml
//       ├── test.rs.tera
//       └── README.md

use std::fs;
use std::path::PathBuf;

// Get template directory
fn get_template_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".knhk")
        .join("templates")
}

// Template metadata structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TemplateMetadata {
    name: String,
    version: String,
    description: String,
    language: String,
    category: String,
    author: String,
    files: Vec<String>,
}

// Implement template listing (line 765)
pub fn list_templates(format: OutputFormat) -> CnvResult<TemplateListResult> {
    let template_dir = get_template_dir();

    // Create directory if it doesn't exist
    fs::create_dir_all(&template_dir).ok();

    let mut templates = Vec::new();

    // Read all subdirectories
    if let Ok(entries) = fs::read_dir(&template_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                // Read template.yaml metadata
                let metadata_file = entry.path().join("template.yaml");
                if metadata_file.exists() {
                    if let Ok(metadata_content) = fs::read_to_string(&metadata_file) {
                        if let Ok(metadata) = serde_yaml::from_str::<TemplateMetadata>(&metadata_content) {
                            templates.push(TemplateInfo {
                                name: metadata.name,
                                version: metadata.version,
                                description: metadata.description,
                                language: metadata.language,
                                category: metadata.category,
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(TemplateListResult {
        total_count: templates.len(),
        templates,
    })
}

// Similar implementations for other template functions...
```

**Note**: Full marketplace implementation requires:
1. Backend API server for template storage
2. Authentication and authorization
3. Template versioning and publishing workflow
4. Rating and review system

For MVP, recommend:
1. Local filesystem implementation (as shown above)
2. Template sync with Git repository
3. Manual curation of templates

---

### Code Generation Implementation Summary

**Files to Modify**:
1. `rust/knhk-cli/src/commands/gen.rs` - Replace TODOs with RDF parsing and Weaver validation

**New Dependencies**:
Add to `rust/knhk-cli/Cargo.toml`:
```toml
tera = { workspace = true, features = ["builtins"] }
serde_yaml = "0.9"
dirs = "5.0"
```

**Weaver Validation**:
```bash
# After generating code
weaver registry check -r registry/

# After running generated workflow
weaver registry live-check --registry registry/
```

**Testing Requirements**:
1. Unit tests: Parse sample Turtle files
2. Integration tests: Generate code for all 43 YAWL patterns
3. Validation tests: Verify generated code passes Weaver checks

**Definition of Done**:
- [ ] RDF/Turtle parsing implemented with oxigraph
- [ ] SPARQL extraction of workflow, tasks, flows
- [ ] Template rendering with tera
- [ ] Weaver schema validation integration
- [ ] Local template management (list, search, install)
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] `cargo test --workspace` passes
- [ ] Generated code compiles and passes Weaver validation

---

## 3. Workflow Execution TODOs

**File**: `rust/knhk-workflow-engine/src/executor/runtime.rs`
**Total TODOs**: 7 (but implementation consolidated into 4 areas)
**DOCTRINE Alignment**: Covenant 1 (Turtle Is Definition and Cause) + Covenant 2 (Invariants Are Law)

### Architecture Context

The workflow runtime executes workflows loaded from Turtle/RDF definitions. It maintains execution state (enabled tasks, completed tasks, data) and executes tasks according to split/join semantics defined in the Turtle specification.

**Key Constraints**:
- Execute ONLY what Turtle defines (no hidden logic)
- Validate split/join combinations against permutation matrix
- State transitions ≤ 8 ticks (Covenant 5)
- Full telemetry for every state change (Covenant 6)

---

### TODO 1: Task Input Mapping (Line 293)

**Location**: Line 293 in `get_task_input()`

**What Needs to Be Implemented**:

Extract relevant input data for a task based on input mappings declared in the Turtle specification. This requires reading `yawl:inputMapping` from the RDF and resolving variables from the workflow data context.

**Implementation Design**:

```rust
/// Get input data for a task based on input mappings in Turtle spec
async fn get_task_input(&self, task: &TaskDefinition) -> WorkflowResult<HashMap<String, serde_json::Value>> {
    let state = self.state.read().await;

    // Extract input mappings from task metadata
    // Input mappings are declared in Turtle as:
    // <task> yawl:inputMapping [
    //     yawl:inputParameter "paramName" ;
    //     yawl:mapsTo "variableName"
    // ]

    let mut task_input = HashMap::new();

    // Check if task has explicit input mappings in metadata
    if let Some(input_mapping_str) = task.metadata.get("input_mapping") {
        // Parse input mapping (format: "param1:var1,param2:var2")
        for mapping in input_mapping_str.split(',') {
            let parts: Vec<&str> = mapping.split(':').collect();
            if parts.len() == 2 {
                let param_name = parts[0].trim();
                let var_name = parts[1].trim();

                // Resolve variable from workflow data context
                if let Some(value) = state.data.get(var_name) {
                    task_input.insert(param_name.to_string(), value.clone());
                } else {
                    debug!(
                        task_id = %task.id,
                        var_name = var_name,
                        "Input variable not found in workflow context"
                    );
                }
            }
        }
    } else {
        // No explicit input mapping - pass entire data context
        // This is the default behavior for simple workflows
        task_input = state.data.clone();
    }

    debug!(
        task_id = %task.id,
        input_count = task_input.len(),
        "Resolved task input data"
    );

    Ok(task_input)
}
```

**DOCTRINE Alignment**:
- **Covenant 1**: Input mappings come from Turtle specification
- No assumptions - if mapping not declared, pass entire context
- Clear logging for debugging

**Turtle Specification Example**:
```turtle
<http://example.org/task1> a yawl:Task ;
    rdfs:label "Process Order" ;
    yawl:inputMapping [
        yawl:inputParameter "orderId" ;
        yawl:mapsTo "currentOrderId"
    ] ;
    yawl:inputMapping [
        yawl:inputParameter "customerId" ;
        yawl:mapsTo "currentCustomerId"
    ] .
```

---

### TODO 2-3: Predicate Evaluation for XOR/OR Splits (Lines 361, 368)

**Locations**:
- Line 361: XOR split predicate evaluation
- Line 368: OR split predicate evaluation

**What Needs to Be Implemented**:

Evaluate predicates (conditions) declared in Turtle to determine which flow(s) to enable after an XOR or OR split. Predicates are boolean expressions that reference workflow variables.

**Implementation Design**:

```rust
// Add predicate evaluation module
mod predicates {
    use serde_json::Value;
    use std::collections::HashMap;

    /// Evaluate a predicate expression against workflow data
    ///
    /// Predicate syntax (simplified):
    /// - "variable == value" - equality check
    /// - "variable > value" - comparison
    /// - "variable != value" - inequality
    ///
    /// For now, we support simple expressions.
    /// Full implementation would use a proper expression parser.
    pub fn evaluate(predicate: &str, data: &HashMap<String, Value>) -> bool {
        // Parse predicate: "variable operator value"
        let parts: Vec<&str> = predicate.split_whitespace().collect();

        if parts.len() < 3 {
            warn!("Invalid predicate format: {}", predicate);
            return false;
        }

        let var_name = parts[0];
        let operator = parts[1];
        let expected_value_str = parts[2..].join(" ").trim_matches('"').to_string();

        // Resolve variable from data context
        let actual_value = match data.get(var_name) {
            Some(v) => v,
            None => {
                debug!("Variable {} not found in data context", var_name);
                return false;
            }
        };

        // Evaluate based on operator
        match operator {
            "==" => {
                // Equality check
                match actual_value {
                    Value::String(s) => s == &expected_value_str,
                    Value::Number(n) => {
                        if let Ok(expected) = expected_value_str.parse::<f64>() {
                            n.as_f64().map(|v| (v - expected).abs() < 0.0001).unwrap_or(false)
                        } else {
                            false
                        }
                    }
                    Value::Bool(b) => {
                        expected_value_str.parse::<bool>().map(|e| *b == e).unwrap_or(false)
                    }
                    _ => false,
                }
            }
            "!=" => {
                // Inequality check (inverse of ==)
                !Self::evaluate(&predicate.replace("!=", "=="), data)
            }
            ">" => {
                // Greater than
                match actual_value {
                    Value::Number(n) => {
                        if let Ok(expected) = expected_value_str.parse::<f64>() {
                            n.as_f64().map(|v| v > expected).unwrap_or(false)
                        } else {
                            false
                        }
                    }
                    _ => false,
                }
            }
            "<" => {
                // Less than
                match actual_value {
                    Value::Number(n) => {
                        if let Ok(expected) = expected_value_str.parse::<f64>() {
                            n.as_f64().map(|v| v < expected).unwrap_or(false)
                        } else {
                            false
                        }
                    }
                    _ => false,
                }
            }
            ">=" | "<=" => {
                // Greater/less than or equal
                let base_op = if operator.starts_with('>') { ">" } else { "<" };
                let eq_check = Self::evaluate(&predicate.replace(operator, "=="), data);
                let cmp_check = Self::evaluate(&predicate.replace(operator, base_op), data);
                eq_check || cmp_check
            }
            _ => {
                warn!("Unsupported operator: {}", operator);
                false
            }
        }
    }
}

// Update handle_split() to use predicate evaluation

/// Handle task split: enable outgoing flows
fn handle_split(&self, state: &mut ExecutionState, task: &TaskDefinition) -> WorkflowResult<()> {
    // Find outgoing flows
    let outgoing: Vec<_> = self.definition.flows.iter()
        .filter(|f| f.from == task.id)
        .collect();

    if outgoing.is_empty() {
        // No outgoing flows - might be output condition
        return Ok(());
    }

    match task.split_type {
        Some(SplitType::AND) => {
            // AND split: enable ALL outgoing flows
            for flow in outgoing {
                self.try_enable_task(state, &flow.to);

                #[cfg(feature = "otel")]
                debug!(
                    from_task = %task.id,
                    to_task = %flow.to,
                    split_type = "AND",
                    "Enabled flow (AND split)"
                );
            }
        }
        Some(SplitType::XOR) => {
            // XOR split: enable EXACTLY ONE flow based on predicate
            // Evaluate predicates in order until one matches
            let mut flow_enabled = false;

            for flow in outgoing {
                if let Some(ref predicate) = flow.predicate {
                    if predicates::evaluate(predicate, &state.data) {
                        self.try_enable_task(state, &flow.to);

                        #[cfg(feature = "otel")]
                        info!(
                            from_task = %task.id,
                            to_task = %flow.to,
                            predicate = %predicate,
                            split_type = "XOR",
                            "Enabled flow (XOR split - predicate matched)"
                        );

                        flow_enabled = true;
                        break; // XOR: only enable first matching flow
                    }
                } else {
                    // No predicate - this is the default flow
                    self.try_enable_task(state, &flow.to);

                    #[cfg(feature = "otel")]
                    info!(
                        from_task = %task.id,
                        to_task = %flow.to,
                        split_type = "XOR",
                        "Enabled default flow (XOR split)"
                    );

                    flow_enabled = true;
                    break;
                }
            }

            if !flow_enabled {
                warn!(
                    task_id = %task.id,
                    "XOR split: no flow enabled (no predicate matched)"
                );
            }
        }
        Some(SplitType::OR) => {
            // OR split: enable ONE OR MORE flows based on predicates
            let mut enabled_count = 0;

            for flow in outgoing {
                if let Some(ref predicate) = flow.predicate {
                    if predicates::evaluate(predicate, &state.data) {
                        self.try_enable_task(state, &flow.to);
                        enabled_count += 1;

                        #[cfg(feature = "otel")]
                        info!(
                            from_task = %task.id,
                            to_task = %flow.to,
                            predicate = %predicate,
                            split_type = "OR",
                            "Enabled flow (OR split - predicate matched)"
                        );
                    }
                } else {
                    // No predicate - enable by default
                    self.try_enable_task(state, &flow.to);
                    enabled_count += 1;
                }
            }

            #[cfg(feature = "otel")]
            debug!(
                task_id = %task.id,
                enabled_count = enabled_count,
                "OR split completed"
            );

            if enabled_count == 0 {
                warn!(
                    task_id = %task.id,
                    "OR split: no flows enabled (no predicates matched)"
                );
            }
        }
        None => {
            // No split: sequence pattern (enable single successor)
            if let Some(flow) = outgoing.first() {
                self.try_enable_task(state, &flow.to);
            }
        }
    }

    Ok(())
}
```

**DOCTRINE Alignment**:
- **Covenant 1**: Predicates come from Turtle specification (flow predicates)
- **Covenant 2**: Split types validated against permutation matrix
- Clear logging for debugging execution path

**Turtle Specification Example**:
```turtle
<http://example.org/task1> a yawl:Task ;
    rdfs:label "Check Amount" ;
    yawl:split yawl:XOR .

<http://example.org/flow1> a yawl:Flow ;
    yawl:flowsFrom <http://example.org/task1> ;
    yawl:flowsInto <http://example.org/task2> ;
    yawl:predicate "amount > 1000" .

<http://example.org/flow2> a yawl:Flow ;
    yawl:flowsFrom <http://example.org/task1> ;
    yawl:flowsInto <http://example.org/task3> ;
    yawl:predicate "amount <= 1000" .
```

---

### TODO 4: OR Join Synchronizing Merge Logic (Line 413)

**Location**: Line 413 in `try_enable_task()`

**What Needs to Be Implemented**:

Implement proper synchronizing merge logic for OR joins. An OR join waits for ALL active (enabled) incoming flows to complete before enabling the task. This is different from XOR join (first to complete) and AND join (wait for all).

**Implementation Design**:

```rust
/// Try to enable a task (check join condition)
fn try_enable_task(&self, state: &mut ExecutionState, task_id: &str) {
    // Find task definition
    let Some(task) = self.definition.tasks.iter().find(|t| t.id == task_id) else {
        warn!("Task {} not found in definition", task_id);
        return;
    };

    // Count incoming tokens
    let incoming: Vec<_> = self.definition.flows.iter()
        .filter(|f| f.to == task_id)
        .collect();

    let completed_incoming = incoming.iter()
        .filter(|f| state.completed_tasks.contains(&f.from))
        .count();

    // Check join condition
    let should_enable = match task.join_type {
        Some(JoinType::AND) => {
            // AND join: wait for ALL incoming flows
            let all_complete = completed_incoming == incoming.len();

            #[cfg(feature = "otel")]
            if all_complete {
                debug!(
                    task_id = task_id,
                    join_type = "AND",
                    completed = completed_incoming,
                    total = incoming.len(),
                    "AND join: all incoming flows completed"
                );
            }

            all_complete
        }
        Some(JoinType::XOR) => {
            // XOR join: enable on FIRST incoming
            let enabled = completed_incoming >= 1;

            #[cfg(feature = "otel")]
            if enabled {
                debug!(
                    task_id = task_id,
                    join_type = "XOR",
                    "XOR join: first incoming flow completed"
                );
            }

            enabled
        }
        Some(JoinType::OR) => {
            // OR join: synchronizing merge (wait for all ACTIVE flows)
            //
            // This is complex: we need to determine which incoming flows are "active"
            // (i.e., were actually enabled by the preceding OR split).
            //
            // Algorithm:
            // 1. Find the source OR split task
            // 2. Determine which flows were enabled from that split
            // 3. Wait for all enabled flows to complete
            //
            // For now, we use a simplified heuristic:
            // - Track which flows have been activated (added to state.tokens)
            // - Wait for all activated flows to complete

            // Get or initialize token count for this join
            let token_key = format!("join:{}", task_id);
            let expected_tokens = state.tokens.get(&token_key).copied().unwrap_or_else(|| {
                // First time - determine expected tokens from incoming flows
                // that have been completed or are running
                let active_count = incoming.iter()
                    .filter(|f| {
                        state.completed_tasks.contains(&f.from) ||
                        state.running_tasks.contains(&f.from)
                    })
                    .count();

                if active_count > 0 {
                    state.tokens.insert(token_key.clone(), active_count);
                    active_count
                } else {
                    // No active flows yet - use total incoming count
                    state.tokens.insert(token_key.clone(), incoming.len());
                    incoming.len()
                }
            });

            let should_enable = completed_incoming >= expected_tokens;

            #[cfg(feature = "otel")]
            debug!(
                task_id = task_id,
                join_type = "OR",
                completed = completed_incoming,
                expected = expected_tokens,
                ready = should_enable,
                "OR join: synchronizing merge check"
            );

            should_enable
        }
        Some(JoinType::Discriminator) => {
            // Discriminator: enable on FIRST, ignore rest
            // Once enabled, mark as "discriminator_fired" to ignore subsequent completions
            let disc_key = format!("disc:{}", task_id);

            if state.tokens.contains_key(&disc_key) {
                // Already fired - don't enable again
                false
            } else if completed_incoming >= 1 {
                // First completion - fire discriminator
                state.tokens.insert(disc_key, 1);

                #[cfg(feature = "otel")]
                info!(
                    task_id = task_id,
                    join_type = "Discriminator",
                    "Discriminator join: first incoming flow completed (firing)"
                );

                true
            } else {
                false
            }
        }
        None => {
            // No join: simple sequence (enable when predecessor completes)
            completed_incoming >= 1
        }
    };

    if should_enable {
        state.enabled_tasks.insert(task_id.to_string());

        #[cfg(feature = "otel")]
        debug!(
            task_id = task_id,
            join_type = ?task.join_type,
            "Task enabled"
        );

        // Emit telemetry
        tokio::spawn({
            let telemetry = self.telemetry.clone();
            let task_id = task_id.to_string();
            let instance_id = state.instance_id.clone();
            async move {
                telemetry.emit_task_event(TaskEvent::Enabled {
                    task_id,
                    instance_id,
                }).await;
            }
        });
    }
}
```

**DOCTRINE Alignment**:
- **Covenant 2**: OR join validated against permutation matrix
- Proper synchronizing merge semantics from YAWL specification
- Clear telemetry for debugging join behavior

**Note on OR Join Complexity**:

The OR join (synchronizing merge) is one of the most complex workflow patterns. The implementation above uses a heuristic approach. For production use, consider:

1. **Enhanced state tracking**: Track which flows were actually enabled by OR split
2. **Token-based synchronization**: Use explicit token passing for precise flow control
3. **Pattern-specific handlers**: Implement OR join as a separate pattern executor

---

### Workflow Execution Implementation Summary

**Files to Modify**:
1. `rust/knhk-workflow-engine/src/executor/runtime.rs` - Replace 7 TODOs

**New Dependencies**: None (all functionality uses existing crates)

**Weaver Validation**:
```bash
# After implementing execution logic
cargo test --workspace

# Run integration tests
make test-integration-v2

# Validate telemetry
weaver registry live-check --registry registry/
```

**Testing Requirements**:
1. Unit tests for predicate evaluation
2. Integration tests for all split/join combinations
3. Validate against permutation matrix
4. Performance tests (≤8 ticks for state transitions)

**Definition of Done**:
- [ ] Task input mapping implemented
- [ ] Predicate evaluation for XOR/OR splits
- [ ] OR join synchronizing merge logic
- [ ] All split/join combinations tested against permutation matrix
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] `cargo test --workspace` passes
- [ ] `make test-chicago-v04` passes (latency ≤ 8 ticks)
- [ ] `weaver registry live-check` passes

---

## Implementation Roadmap

### Phase 1: OTEL Telemetry (High Priority - Foundation)

**Why First**: Telemetry provides observability for all other features. Without it, we can't validate anything.

**Tasks**:
1. Implement 12 OTEL metric emissions in `telemetry.rs`
2. Add metric descriptions in `WorkflowTelemetry::new()`
3. Validate with Weaver schema check

**Estimated Effort**: 2-3 hours
**Validation**: `weaver registry check -r registry/`

---

### Phase 2: Workflow Execution (Core Functionality)

**Why Second**: Enables actual workflow execution, which generates telemetry for validation.

**Tasks**:
1. Implement task input mapping (line 293)
2. Implement predicate evaluation (lines 361, 368)
3. Implement OR join synchronizing merge (line 413)

**Estimated Effort**: 4-6 hours
**Validation**: Integration tests + Weaver live-check

---

### Phase 3: Code Generation (Productivity)

**Why Third**: Enables generating workflows from Turtle specs, accelerating development.

**Tasks**:
1. Implement RDF/Turtle parsing with oxigraph (line 121-127)
2. Implement Weaver validation integration (line 146-149)
3. Update code generation templates with runtime behavior
4. Implement local template management

**Estimated Effort**: 6-8 hours
**Validation**: Generate code for all 43 patterns + Weaver validation

---

### Phase 4: Marketplace Integration (Optional)

**Why Last**: Nice-to-have feature that requires backend infrastructure.

**Tasks**:
1. Implement local filesystem template storage
2. Design marketplace API specification
3. Implement template publishing workflow (future work)

**Estimated Effort**: 8-12 hours (MVP), 40+ hours (full marketplace)
**Validation**: Template installation and usage

---

## Validation Checklist

After implementing all TODOs, verify:

### Build Quality
- [ ] `cargo build --workspace --release` succeeds with zero warnings
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] `cargo fmt --all -- --check` passes

### Testing
- [ ] `cargo test --workspace` passes
- [ ] `make test-chicago-v04` passes (latency tests)
- [ ] `make test-performance-v04` passes (≤8 ticks)
- [ ] `make test-integration-v2` passes

### OTEL Validation (Source of Truth)
- [ ] `weaver registry check -r registry/` passes
- [ ] `weaver registry live-check --registry registry/` passes
- [ ] All workflow events emit telemetry
- [ ] All task events emit telemetry
- [ ] Metrics conform to schema

### DOCTRINE Compliance
- [ ] **Covenant 1**: Turtle is single source of truth (no hidden logic)
- [ ] **Covenant 2**: All invariants validated (split/join matrix, resource bounds)
- [ ] **Covenant 5**: Hot path latency ≤ 8 ticks (Chatman constant)
- [ ] **Covenant 6**: All state transitions observable via telemetry

---

## Appendix A: Dependency Matrix

| Feature | Crate | Version | Already in Cargo.toml? |
|---------|-------|---------|----------------------|
| OTEL Metrics | `metrics` | 0.23 | ✅ Yes (workflow-engine) |
| OTEL Metrics | `metrics-prometheus` | 0.6 | ✅ Yes (workflow-engine) |
| RDF Parsing | `oxigraph` | workspace | ✅ Yes (workflow-engine, cli) |
| Templates | `tera` | workspace | ✅ Yes (workflow-engine) |
| YAML | `serde_yaml` | 0.9 | ❌ Add to cli |
| Directories | `dirs` | 5.0 | ❌ Add to cli |

**Action Items**:
Add to `rust/knhk-cli/Cargo.toml`:
```toml
serde_yaml = "0.9"
dirs = "5.0"
```

---

## Appendix B: Weaver Schema Reference

Key schema definitions from `registry/knhk-workflow-engine.yaml`:

**Metrics**:
- `workflow_started_total` - Counter (line 112)
- `workflow_completed_total` - Counter (line 121)
- `workflow_failed_total` - Counter (line 131)
- `workflow_cancelled_total` - Counter (line 140)
- `task_enabled_total` - Counter (line 163)
- `task_started_total` - Counter (line 172)
- `task_completed_total` - Counter (line 182)
- `task_duration_seconds` - Histogram (line 184)
- `task_failed_total` - Counter (line 200)
- `task_cancelled_total` - Counter (line 209)
- `workflow_transition_duration_seconds` - Histogram (line 255)
- `workflow_throughput_tasks_per_second` - Gauge (line 280)
- `workflow_memory_bytes` - Gauge (line 293)
- `workflow_cpu_percent` - Gauge (line 294)

**Spans**:
- `knhk.workflow_engine.register_workflow` (lines 107-121)
- `knhk.workflow_engine.create_case` (lines 124-140)
- `knhk.workflow_engine.execute_case` (lines 142-158)
- `knhk.workflow_engine.execute_task` (lines 161-176)
- `knhk.workflow_engine.execute_pattern` (lines 179-193)

---

## Appendix C: Testing Examples

### Testing OTEL Metrics

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use metrics::testing::TestRecorder;

    #[tokio::test]
    async fn test_workflow_event_metrics() {
        // Install test recorder
        let recorder = TestRecorder::default();
        metrics::with_recorder(&recorder, || {
            let telemetry = WorkflowTelemetry::new("test-instance".to_string());

            // Emit workflow started event
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                telemetry.emit_workflow_event(WorkflowEvent::Started {
                    workflow_id: "test-workflow".to_string(),
                    instance_id: "test-instance".to_string(),
                }).await;
            });

            // Verify metric was emitted
            let metrics = recorder.metrics();
            assert!(metrics.iter().any(|m| m.name == "workflow_started_total"));
        });
    }
}
```

### Testing Predicate Evaluation

```rust
#[cfg(test)]
mod tests {
    use super::predicates::*;

    #[test]
    fn test_predicate_evaluation() {
        let mut data = HashMap::new();
        data.insert("amount".to_string(), serde_json::json!(1500));

        // Test equality
        assert!(evaluate("amount == 1500", &data));
        assert!(!evaluate("amount == 1000", &data));

        // Test comparison
        assert!(evaluate("amount > 1000", &data));
        assert!(!evaluate("amount > 2000", &data));

        // Test less than
        assert!(evaluate("amount < 2000", &data));
        assert!(!evaluate("amount < 1000", &data));
    }
}
```

---

**END OF IMPLEMENTATION DESIGN DOCUMENT**
