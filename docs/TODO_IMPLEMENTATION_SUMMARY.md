# KNHK TODO Implementation Summary

**Quick Reference** | **Full Details**: See `TODO_IMPLEMENTATION_DESIGN.md`

---

## Overview

**Total TODOs**: 39 across 3 files
- **OTEL Telemetry**: 12 TODOs (telemetry.rs)
- **Code Generation**: 20 TODOs (gen.rs)
- **Workflow Execution**: 7 TODOs (runtime.rs)

---

## 1. OTEL Telemetry TODOs (12 items)

**File**: `rust/knhk-workflow-engine/src/executor/telemetry.rs`
**DOCTRINE**: Covenant 6 (Observations Drive Everything)

### Quick Implementation Guide

**Add to imports**:
```rust
use metrics::{counter, histogram, gauge, describe_counter, describe_histogram, describe_gauge};
```

**Metrics to Emit**:

| Line | Metric Name | Type | Description |
|------|-------------|------|-------------|
| 112 | `workflow_started_total` | Counter | Workflows started |
| 121 | `workflow_completed_total` | Counter | Workflows completed |
| 131 | `workflow_failed_total` | Counter | Workflows failed |
| 140 | `workflow_cancelled_total` | Counter | Workflows cancelled |
| 163 | `task_enabled_total` | Counter | Tasks enabled |
| 172 | `task_started_total` | Counter | Tasks started |
| 182 | `task_completed_total` | Counter | Tasks completed |
| 184 | `task_duration_seconds` | Histogram | Task execution time |
| 200 | `task_failed_total` | Counter | Tasks failed |
| 209 | `task_cancelled_total` | Counter | Tasks cancelled |
| 255 | `workflow_transition_duration_seconds` | Histogram | State transition time |
| 280 | `workflow_throughput_tasks_per_second` | Gauge | Current throughput |
| 293-294 | `workflow_memory_bytes`, `workflow_cpu_percent` | Gauge | Resource usage |

**Pattern for Counters**:
```rust
counter!("metric_name",
    "label1" => value1.clone(),
    "label2" => value2.clone()
).increment(1);
```

**Pattern for Histograms**:
```rust
histogram!("metric_name",
    "label1" => value1.clone()
).record(duration.as_secs_f64());
```

**Pattern for Gauges**:
```rust
gauge!("metric_name",
    "label1" => value1.clone()
).set(value as f64);
```

**Critical**: Include Chatman Constant checks (≤8 ticks) in duration metrics!

---

## 2. Code Generation TODOs (20 items)

**File**: `rust/knhk-cli/src/commands/gen.rs`
**DOCTRINE**: Covenant 1 (Turtle Is Definition and Cause)

### Quick Implementation Guide

**Core Implementation** (Line 121-127):

1. **Parse Turtle with oxigraph**:
```rust
let store = Store::new()?;
store.load_from_reader(oxigraph::io::RdfFormat::Turtle, spec_content.as_bytes())?;
```

2. **Extract via SPARQL**:
```rust
let query = format!(r#"
    PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
    SELECT ?spec ?name WHERE {{
        ?spec rdf:type yawl:Specification .
        OPTIONAL {{ ?spec rdfs:label ?name }}
    }}
"#);
let results = store.query(&query)?;
```

3. **Apply Tera template**:
```rust
let template_engine = tera::Tera::new("templates/**/*.tera")?;
let context = tera::Context::new();
context.insert("workflow", &workflow_spec);
let code = template_engine.render("workflow.rs.tera", &context)?;
```

**Weaver Validation** (Line 146-149):
```rust
let output = Command::new("weaver")
    .args(&["registry", "check", "-r", "registry/"])
    .output()?;

if !output.status.success() {
    return Err(/* validation failed */);
}
```

**New Dependencies** (add to Cargo.toml):
```toml
serde_yaml = "0.9"
dirs = "5.0"
```

---

## 3. Workflow Execution TODOs (7 items)

**File**: `rust/knhk-workflow-engine/src/executor/runtime.rs`
**DOCTRINE**: Covenant 1 + Covenant 2 (Turtle + Invariants)

### Quick Implementation Guide

**Task Input Mapping** (Line 293):
```rust
async fn get_task_input(&self, task: &TaskDefinition) -> WorkflowResult<HashMap<String, Value>> {
    let state = self.state.read().await;
    let mut task_input = HashMap::new();

    // Parse input mapping from task.metadata["input_mapping"]
    // Format: "param1:var1,param2:var2"
    if let Some(mapping) = task.metadata.get("input_mapping") {
        for entry in mapping.split(',') {
            let parts: Vec<_> = entry.split(':').collect();
            if parts.len() == 2 {
                if let Some(value) = state.data.get(parts[1]) {
                    task_input.insert(parts[0].to_string(), value.clone());
                }
            }
        }
    } else {
        // No explicit mapping - pass entire context
        task_input = state.data.clone();
    }

    Ok(task_input)
}
```

**Predicate Evaluation** (Lines 361, 368):
```rust
mod predicates {
    pub fn evaluate(predicate: &str, data: &HashMap<String, Value>) -> bool {
        // Parse: "variable operator value"
        let parts: Vec<_> = predicate.split_whitespace().collect();
        if parts.len() < 3 { return false; }

        let var_name = parts[0];
        let operator = parts[1];
        let expected = parts[2];

        let actual = data.get(var_name)?;

        match operator {
            "==" => /* compare values */,
            "!=" => /* inverse of == */,
            ">" | "<" | ">=" | "<=" => /* numeric comparison */,
            _ => false,
        }
    }
}
```

**OR Join Synchronizing Merge** (Line 413):
```rust
Some(JoinType::OR) => {
    // Track expected tokens for this join
    let token_key = format!("join:{}", task_id);
    let expected_tokens = state.tokens.get(&token_key).copied()
        .unwrap_or_else(|| {
            // Count active incoming flows
            let active = incoming.iter()
                .filter(|f| state.completed_tasks.contains(&f.from) ||
                           state.running_tasks.contains(&f.from))
                .count();
            state.tokens.insert(token_key.clone(), active);
            active
        });

    let should_enable = completed_incoming >= expected_tokens;
    should_enable
}
```

---

## Implementation Priority

### Phase 1: OTEL Telemetry ⚡ HIGH PRIORITY
**Why**: Foundation for all validation (Weaver requires telemetry)
**Time**: 2-3 hours
**Files**: `telemetry.rs`

### Phase 2: Workflow Execution 🔧 CORE
**Why**: Enables actual workflow execution
**Time**: 4-6 hours
**Files**: `runtime.rs`

### Phase 3: Code Generation 📝 PRODUCTIVITY
**Why**: Generate workflows from Turtle specs
**Time**: 6-8 hours
**Files**: `gen.rs`

### Phase 4: Marketplace 🏪 OPTIONAL
**Why**: Nice-to-have, requires backend infrastructure
**Time**: 8-12 hours (MVP), 40+ hours (full)
**Files**: `gen.rs` (templates module)

---

## Validation Checklist

After implementation, verify:

### Build
- [ ] `cargo build --release` (zero warnings)
- [ ] `cargo clippy --workspace -- -D warnings` (passes)
- [ ] `cargo fmt --all -- --check` (passes)

### Tests
- [ ] `cargo test --workspace` (all pass)
- [ ] `make test-chicago-v04` (latency ≤8 ticks)
- [ ] `make test-performance-v04` (performance bounds)
- [ ] `make test-integration-v2` (integration tests)

### **CRITICAL: Weaver Validation (Source of Truth)**
- [ ] `weaver registry check -r registry/` (schema valid)
- [ ] `weaver registry live-check --registry registry/` (runtime telemetry valid)

### DOCTRINE Compliance
- [ ] Covenant 1: Turtle is single source (no hidden logic)
- [ ] Covenant 2: All invariants validated (permutation matrix)
- [ ] Covenant 5: Hot path ≤8 ticks (Chatman constant)
- [ ] Covenant 6: All state transitions observable

---

## Dependencies

**Already in Cargo.toml**:
- ✅ `metrics` (0.23)
- ✅ `oxigraph` (workspace)
- ✅ `tera` (workspace)

**Need to Add** (to `knhk-cli/Cargo.toml`):
```toml
serde_yaml = "0.9"
dirs = "5.0"
```

---

## Key Architectural Principles

1. **PURE PASSTHROUGH**: Templates have zero logic - all behavior from Turtle
2. **WEAVER IS TRUTH**: Only Weaver validation proves telemetry works
3. **8-TICK BOUND**: Hot path operations must complete in ≤8 ticks
4. **NO ASSUMPTIONS**: Only execute what Turtle explicitly declares
5. **OBSERVABLE EVERYTHING**: Every state transition emits telemetry

---

## Common Pitfalls to Avoid

❌ **DON'T**:
- Add conditional logic to templates
- Filter or reorder SPARQL results
- Skip Weaver validation
- Use `--help` text as validation (proves nothing!)
- Assume default behavior not in Turtle
- Trust test passes without Weaver validation

✅ **DO**:
- Use SPARQL to extract EXACTLY what's in Turtle
- Validate with Weaver (static + live checks)
- Check performance bounds (≤8 ticks)
- Emit telemetry for ALL state transitions
- Document what comes from Turtle vs. code

---

## Quick Start

```bash
# 1. Implement OTEL telemetry
cd rust/knhk-workflow-engine
# Edit src/executor/telemetry.rs - replace 12 TODOs

# 2. Verify metrics
cargo test

# 3. Validate schema
weaver registry check -r ../../registry/

# 4. Implement runtime execution
# Edit src/executor/runtime.rs - replace 7 TODOs

# 5. Implement code generation
cd ../knhk-cli
# Edit src/commands/gen.rs - replace 20 TODOs

# 6. Full validation
cd ../..
make test-chicago-v04
weaver registry live-check --registry registry/
```

---

**For detailed implementation code, see**: `TODO_IMPLEMENTATION_DESIGN.md`
