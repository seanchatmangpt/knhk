# ggen Templates: Projection Layer (Π) Documentation

**Version**: 1.0.0
**Status**: Production Ready
**Last Updated**: 2024-11-16

## Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Template Categories](#template-categories)
- [Quick Start](#quick-start)
- [Template Reference](#template-reference)
- [SPARQL Integration](#sparql-integration)
- [Build Integration](#build-integration)
- [Validation](#validation)
- [Examples](#examples)

---

## Overview

The ggen projection layer (Π) transforms ontologies (Σ) into executable code and configurations. This enables:

- **Schema-driven development**: Code generated from RDF/OWL ontologies
- **Type safety**: Generated code preserves ontological type constraints
- **Deterministic output**: Same ontology → same generated code
- **Full observability**: Auto-generated OTEL spans, metrics, events
- **Validation by design**: Weaver registry schemas ensure correctness

### The Projection Function

```
Π: Σ → Code

Where:
  Σ = Ontology (RDF/Turtle/OWL)
  Π = ggen template engine
  Code = {Rust, YAML, Weaver Registry}
```

### Key Principle

**A = μ(O)** - Actions are deterministic projections of observations

The ggen templates implement μ (the measurement/projection function) that transforms ontological observations (O) into executable actions (A).

---

## Architecture

### Three-Layer Code Generation

```
┌─────────────────────────────────────────┐
│  Ontology Layer (Σ)                     │
│  - RDF/Turtle workflow specs            │
│  - YAWL pattern definitions             │
│  - SHACL constraints                    │
└──────────────┬──────────────────────────┘
               │
               │ SPARQL Queries
               ├──> Extract tasks
               ├──> Extract states
               ├──> Extract transitions
               ├──> Extract hooks
               │
┌──────────────▼──────────────────────────┐
│  Template Layer (Π)                     │
│  - Handlebars templates (.hbs)          │
│  - Variable binding from SPARQL         │
│  - Helper functions                     │
└──────────────┬──────────────────────────┘
               │
               │ Code Generation
               ├──> Rust code
               ├──> YAML configs
               └──> Weaver registry
               │
┌──────────────▼──────────────────────────┐
│  Output Layer (Code)                    │
│  - Type-safe Rust modules               │
│  - Workflow configurations              │
│  - OTEL observability schemas           │
└─────────────────────────────────────────┘
```

### Directory Structure

```
templates/
├── rust-knhk/           # Rust code generation
│   ├── task_enum.rs.hbs
│   ├── state_machine.rs.hbs
│   ├── hooks.rs.hbs
│   └── otel_spans.rs.hbs
├── config/              # Configuration generation
│   └── workflow.yaml.hbs
├── weaver/              # OTEL Weaver registry
│   └── registry.yaml.hbs
└── sparql/              # SPARQL integration
    ├── query_bindings.hbs
    └── integration_example.rs.hbs
```

---

## Template Categories

### 1. Rust Code Generation Templates

#### task_enum.rs.hbs

Generates task enumerations from YAWL workflow ontology.

**Input**: Workflow ontology with tasks
**Output**: Rust enum with all tasks, metadata, and helper methods

**Features**:
- Task name → Rust enum variant
- YAWL pattern metadata
- Guard function references
- Comprehensive task metadata
- Test generation

**Example**:
```rust
// Generated from ontology
#[derive(Debug, Clone)]
pub enum ApprovalWorkflowTask {
    /// Submit request for approval
    SubmitRequest,
    /// Manager reviews request
    ManagerReview,
    /// Director approves request
    DirectorApproval,
}
```

#### state_machine.rs.hbs

Generates state machine implementation from workflow graph.

**Input**: States and transitions from ontology
**Output**: State enum, transition logic, guard checks

**Features**:
- State definitions with metadata
- Transition validation
- Guard checking logic
- History tracking
- Error handling

**Example**:
```rust
// Generated state machine
#[derive(Debug, Clone)]
pub enum ApprovalState {
    Pending,
    ManagerReview,
    DirectorReview,
    Approved,
    Rejected,
}

impl StateMachine {
    pub fn transition(&mut self, event: &str) -> Result<()> {
        // Generated transition logic with guard checks
    }
}
```

#### hooks.rs.hbs

Generates hook functions for knowledge-driven workflows.

**Input**: Hook definitions from ontology
**Output**: Async hook functions with guards and validation

**Features**:
- Pre/post-condition validation
- Guard check integration
- SPARQL query execution
- OTEL span creation
- Error handling

**Example**:
```rust
// Generated hook
pub async fn submit_request_hook(
    context: &mut HookContext,
    input: Value,
) -> Result<Value> {
    // Generated guard checks
    // Generated SPARQL queries
    // Generated action logic
}
```

#### otel_spans.rs.hbs

Generates OpenTelemetry observability code.

**Input**: Tasks, hooks, events from ontology
**Output**: Span creation, metrics, events

**Features**:
- Auto-generated span definitions
- Metrics for all operations
- Event recording helpers
- Proper attribute assignment
- Weaver schema compliance

**Example**:
```rust
// Generated OTEL metrics
pub struct WorkflowMetrics {
    pub task_executions: Counter<u64>,
    pub task_duration: Histogram<f64>,
    pub guard_checks: Counter<u64>,
}
```

### 2. Configuration Templates

#### workflow.yaml.hbs

Generates complete workflow configuration.

**Input**: Full workflow ontology
**Output**: YAML configuration with all settings

**Features**:
- Task definitions
- State machine config
- Hook configuration
- Guard constraints
- MAPE-K settings
- Performance constraints
- Observability config

### 3. Weaver Registry Templates

#### registry.yaml.hbs

Generates OpenTelemetry Weaver registry schema.

**Input**: Workflow tasks, metrics, events
**Output**: Weaver-compliant OTEL schema

**Features**:
- Span definitions
- Metric definitions
- Event schemas
- Attribute requirements
- Validation rules

**Validation**: Must pass `weaver registry check`

---

## Quick Start

### 1. Prerequisites

```bash
# Install dependencies
npm install -g handlebars-cli
cargo install oxigraph_cli
pip install weaver-cli
```

### 2. Create Workflow Ontology

Create `my_workflow.ttl`:

```turtle
@prefix yawl: <http://yawl.org/> .
@prefix knhk: <http://knhk.io/> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

:MyWorkflow a yawl:Workflow ;
    rdfs:label "My Workflow" ;
    yawl:workflowId "my-workflow-v1" .

:Task1 a yawl:Task ;
    rdfs:label "Start Task" ;
    yawl:taskId "task-1" ;
    yawl:taskType "Atomic" ;
    yawl:pattern "Basic" .
```

### 3. Generate Code

```bash
# Generate all code from ontology
./scripts/ggen/generate-workflows.sh my_workflow.ttl

# Output:
#   target/generated/src/       (Rust code)
#   target/generated/config/    (YAML config)
#   target/generated/weaver/    (OTEL schema)
```

### 4. Validate Generated Code

```bash
# Validate templates
./scripts/ggen/validate-templates.sh

# Validate generated Rust
cargo fmt --check target/generated/src/*.rs

# Validate Weaver registry
weaver registry check -r target/generated/weaver/

# Validate YAML
yamllint target/generated/config/*.yaml
```

### 5. Integrate with Project

```bash
# Copy generated code to your crate
cp target/generated/src/* rust/my-crate/src/

# Build and test
cd rust/my-crate
cargo build
cargo test
```

---

## Template Reference

### Handlebars Helpers

#### Built-in Helpers

- `{{#each items}}...{{/each}}` - Iterate over arrays
- `{{#if condition}}...{{/if}}` - Conditional rendering
- `{{#unless condition}}...{{/unless}}` - Negative conditional
- `{{this}}` - Current context item
- `{{@index}}` - Loop index
- `{{@key}}` - Object key
- `{{@first}}` / `{{@last}}` - Loop position

#### Custom Helpers (Project-Specific)

- `{{pascal_case string}}` - Convert to PascalCase
- `{{snake_case string}}` - Convert to snake_case
- `{{upper string}}` - Convert to UPPERCASE
- `{{first_key object}}` - Get first object key
- `{{first_value object}}` - Get first object value

### Template Variables

#### Common Variables

All templates have access to:

```handlebars
{{workflow_name}}           - Workflow name
{{workflow_id}}             - Workflow identifier
{{workflow_version}}        - Version string
{{generation_timestamp}}    - ISO timestamp
{{ontology_path}}          - Source ontology path
```

#### Task Variables

```handlebars
{{#each tasks}}
  {{this.id}}              - Task ID
  {{this.name}}            - Task name
  {{this.task_type}}       - Task type
  {{this.yawl_pattern}}    - YAWL pattern
  {{this.description}}     - Description
  {{this.guards}}          - Array of guards
  {{this.inputs}}          - Input parameters
  {{this.outputs}}         - Output parameters
{{/each}}
```

#### State Variables

```handlebars
{{#each states}}
  {{this.name}}            - State name
  {{this.type}}            - State type
  {{this.is_final}}        - Is final state?
  {{this.is_error}}        - Is error state?
  {{this.timeout_ms}}      - Timeout in ms
{{/each}}
```

#### Transition Variables

```handlebars
{{#each transitions}}
  {{this.from}}            - Source state
  {{this.to}}              - Target state
  {{this.event}}           - Trigger event
  {{this.guards}}          - Guard conditions
  {{this.condition}}       - Transition condition
{{/each}}
```

#### Hook Variables

```handlebars
{{#each hooks}}
  {{this.name}}            - Hook name
  {{this.trigger}}         - Trigger event
  {{this.type}}            - Hook type
  {{this.yawl_pattern}}    - YAWL pattern
  {{this.guards}}          - Guard conditions
  {{this.sparql_query}}    - SPARQL query
  {{this.action}}          - Action to execute
{{/each}}
```

---

## SPARQL Integration

### How SPARQL Results Bind to Templates

ggen templates consume SPARQL query results automatically:

#### 1. Execute SPARQL Query

```sparql
SELECT ?taskName ?taskType ?pattern WHERE {
  ?task a yawl:Task ;
        rdfs:label ?taskName ;
        yawl:taskType ?taskType ;
        yawl:pattern ?pattern .
}
```

#### 2. Results Bind to Template Context

```json
{
  "sparql_results": [
    {
      "taskName": "Submit Request",
      "taskType": "Atomic",
      "pattern": "Basic"
    },
    {
      "taskName": "Review Request",
      "taskType": "Composite",
      "pattern": "Sequential"
    }
  ]
}
```

#### 3. Access in Template

```handlebars
{{#each sparql_results}}
pub const TASK_{{upper (snake_case this.taskName)}}: &str = "{{this.taskName}}";
{{/each}}
```

#### 4. Generated Code

```rust
pub const TASK_SUBMIT_REQUEST: &str = "Submit Request";
pub const TASK_REVIEW_REQUEST: &str = "Review Request";
```

### SPARQL Query Patterns

#### Extract All Tasks

```sparql
SELECT ?id ?name ?type ?pattern ?description WHERE {
  ?task a yawl:Task ;
        yawl:taskId ?id ;
        rdfs:label ?name ;
        yawl:taskType ?type ;
        yawl:pattern ?pattern .
  OPTIONAL { ?task rdfs:comment ?description }
}
```

#### Extract State Transitions

```sparql
SELECT ?fromState ?toState ?event ?guard WHERE {
  ?trans a yawl:Transition ;
         yawl:fromState ?from ;
         yawl:toState ?to ;
         yawl:event ?event .
  ?from rdfs:label ?fromState .
  ?to rdfs:label ?toState .
  OPTIONAL { ?trans yawl:guard ?guard }
}
```

#### Extract Guards

```sparql
SELECT ?guardName ?guardType ?expression ?maxRunLen WHERE {
  ?guard a knhk:Guard ;
         rdfs:label ?guardName ;
         knhk:guardType ?guardType ;
         knhk:expression ?expression ;
         knhk:maxRunLen ?maxRunLen .
}
```

#### ASK Queries

```sparql
ASK { ?workflow yawl:hasParallelism true }
```

Use in template:
```handlebars
{{#if sparql_ask_result}}
// Parallelism enabled
{{else}}
// Sequential execution only
{{/if}}
```

---

## Build Integration

### Cargo Integration

Add to `build.rs`:

```rust
use std::process::Command;

fn main() {
    // Trigger rebuild if ontology changes
    println!("cargo:rerun-if-changed=ontology/workflow.ttl");

    // Generate code during build
    let status = Command::new("./scripts/ggen/generate-workflows.sh")
        .arg("ontology/workflow.ttl")
        .arg("target/generated")
        .status()
        .expect("Failed to execute ggen");

    if !status.success() {
        panic!("ggen code generation failed");
    }
}
```

### Makefile Integration

```makefile
# Generate workflows from ontology
.PHONY: ggen-generate
ggen-generate:
	@echo "Generating workflows from ontology..."
	./scripts/ggen/generate-workflows.sh ontology/workflows/*.ttl

# Validate templates
.PHONY: ggen-validate
ggen-validate:
	@echo "Validating ggen templates..."
	./scripts/ggen/validate-templates.sh

# Clean generated code
.PHONY: ggen-clean
ggen-clean:
	@echo "Cleaning generated code..."
	rm -rf target/generated/*

# Full workflow: validate → generate → test
.PHONY: ggen-workflow
ggen-workflow: ggen-validate ggen-generate
	@echo "Running tests on generated code..."
	cargo test --package generated-workflows
```

### CI/CD Integration

```yaml
# .github/workflows/ggen.yml
name: ggen Code Generation

on:
  push:
    paths:
      - 'ontology/**/*.ttl'
      - 'templates/**/*.hbs'

jobs:
  generate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install dependencies
        run: |
          npm install -g handlebars-cli
          cargo install oxigraph_cli
          pip install weaver-cli

      - name: Validate templates
        run: ./scripts/ggen/validate-templates.sh

      - name: Generate code
        run: ./scripts/ggen/generate-workflows.sh ontology/workflow.ttl

      - name: Validate generated code
        run: |
          cargo fmt --check target/generated/src/*.rs
          weaver registry check -r target/generated/weaver/

      - name: Test generated code
        run: cargo test --package generated-workflows
```

---

## Validation

### Template Validation

```bash
# Validate template syntax and structure
./scripts/ggen/validate-templates.sh
```

**Checks**:
- Directory structure exists
- Required templates present
- Handlebars syntax valid
- No unclosed blocks
- No malformed expressions

### Generated Code Validation

#### Rust Validation

```bash
# Format check
cargo fmt --check target/generated/src/*.rs

# Lint check
cargo clippy --manifest-path target/generated/Cargo.toml -- -D warnings

# Build check
cargo build --manifest-path target/generated/Cargo.toml

# Test check
cargo test --manifest-path target/generated/Cargo.toml
```

#### YAML Validation

```bash
# Syntax validation
yamllint target/generated/config/*.yaml

# Schema validation (if schema exists)
python -m yaml_validator --schema schema.yaml target/generated/config/workflow.yaml
```

#### Weaver Registry Validation

```bash
# Validate registry schema
weaver registry check -r target/generated/weaver/

# Validate against live telemetry (if running)
weaver registry live-check --registry target/generated/weaver/
```

**This is the ONLY source of truth for validation!**

---

## Examples

### Example 1: Simple Sequential Workflow

**Ontology** (`simple_workflow.ttl`):
```turtle
@prefix : <http://example.org/> .
@prefix yawl: <http://yawl.org/> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

:SimpleWorkflow a yawl:Workflow ;
    rdfs:label "Simple Sequential Workflow" ;
    yawl:workflowId "simple-v1" .

:Task1 a yawl:Task ;
    rdfs:label "Start" ;
    yawl:taskId "start" ;
    yawl:taskType "Atomic" ;
    yawl:pattern "Sequence" .

:Task2 a yawl:Task ;
    rdfs:label "Process" ;
    yawl:taskId "process" ;
    yawl:taskType "Atomic" ;
    yawl:pattern "Sequence" .

:Task3 a yawl:Task ;
    rdfs:label "Complete" ;
    yawl:taskId "complete" ;
    yawl:taskType "Atomic" ;
    yawl:pattern "Sequence" .
```

**Generate**:
```bash
./scripts/ggen/generate-workflows.sh simple_workflow.ttl
```

**Generated Task Enum** (`task_enum.rs`):
```rust
pub enum SimpleSequentialWorkflowTask {
    Start,
    Process,
    Complete,
}
```

### Example 2: Approval Workflow with Guards

**Ontology** (`approval_workflow.ttl`):
```turtle
:ApprovalWorkflow a yawl:Workflow ;
    rdfs:label "Approval Workflow" .

:SubmitTask a yawl:Task ;
    rdfs:label "Submit Request" ;
    knhk:hasGuard :AmountGuard .

:AmountGuard a knhk:Guard ;
    rdfs:label "amount_check" ;
    knhk:guardType "COMPARE_O_LE" ;
    knhk:expression "amount <= 10000" ;
    knhk:maxRunLen 8 .
```

**Generated Hook** (`hooks.rs`):
```rust
pub async fn submit_request_hook(
    context: &mut HookContext,
    input: Value,
) -> Result<Value> {
    // Generated guard check
    if !check_guard_amount_check(context)? {
        return Err(WorkflowError::GuardCheckFailed {
            guard: "amount_check".to_string(),
            transition: "submit_request".to_string(),
        });
    }

    // Execute hook logic
    Ok(output)
}

fn check_guard_amount_check(context: &HookContext) -> Result<bool> {
    let amount = context.data.get("amount")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);

    Ok(amount <= 10000.0)
}
```

### Example 3: Parallel Split Pattern

**Ontology**:
```turtle
:ParallelWorkflow a yawl:Workflow ;
    rdfs:label "Parallel Processing" .

:SplitTask a yawl:Task ;
    rdfs:label "Split Work" ;
    yawl:pattern "ParallelSplit" .

:Task1 a yawl:Task ;
    rdfs:label "Process A" ;
    yawl:dependsOn :SplitTask .

:Task2 a yawl:Task ;
    rdfs:label "Process B" ;
    yawl:dependsOn :SplitTask .

:JoinTask a yawl:Task ;
    rdfs:label "Join Results" ;
    yawl:pattern "Synchronization" ;
    yawl:dependsOn :Task1, :Task2 .
```

**Generated State Machine**:
```rust
pub enum ParallelProcessingState {
    Initial,
    Split,
    ProcessingA,
    ProcessingB,
    Joined,
    Complete,
}

impl StateMachine {
    fn transition(&mut self, event: &str) -> Result<()> {
        match (&self.current_state, event) {
            (State::Split, "start_parallel") => {
                // Fork execution
                Ok(())
            }
            (State::ProcessingA | State::ProcessingB, "complete") => {
                // Check if both branches complete
                if self.all_parallel_complete() {
                    self.current_state = State::Joined;
                }
                Ok(())
            }
            _ => Err(InvalidTransition)
        }
    }
}
```

---

## Best Practices

### 1. Ontology Design

**DO**:
- Use descriptive labels (`rdfs:label`)
- Include comments (`rdfs:comment`)
- Define all YAWL patterns explicitly
- Specify guard constraints clearly
- Use consistent namespaces

**DON'T**:
- Hardcode values that should be configurable
- Omit type information
- Mix concerns (keep workflow, guards, hooks separate)
- Use non-standard prefixes without documentation

### 2. Template Design

**DO**:
- Include generation timestamp and source
- Add "DO NOT EDIT" warnings
- Generate comprehensive tests
- Include metadata and documentation
- Use type-safe constructs

**DON'T**:
- Generate placeholder code
- Omit error handling
- Create overly complex templates
- Mix generation logic with business logic

### 3. Validation

**DO**:
- Always validate with Weaver (`weaver registry check`)
- Run rustfmt and clippy on generated code
- Test generated code comprehensively
- Validate YAML syntax
- Check for determinism (same input → same output)

**DON'T**:
- Skip validation steps
- Trust text without testing
- Deploy generated code without Weaver validation
- Ignore clippy warnings

### 4. Build Integration

**DO**:
- Regenerate on ontology changes
- Version generated code separately
- Include validation in CI/CD
- Track generation metadata

**DON'T**:
- Commit generated code to version control
- Skip validation in CI
- Manually edit generated code
- Mix generated and hand-written code

---

## Troubleshooting

### Common Issues

#### Issue: Template rendering fails

**Symptoms**: `handlebars` error during generation

**Solutions**:
1. Validate template syntax: `./scripts/ggen/validate-templates.sh`
2. Check SPARQL query results: `npx oxigraph query --file ontology.ttl --query "SELECT * WHERE { ?s ?p ?o }"`
3. Verify context JSON: `cat target/generated/context.json | jq`

#### Issue: Generated Rust code doesn't compile

**Symptoms**: `cargo build` fails

**Solutions**:
1. Run rustfmt: `rustfmt target/generated/src/*.rs`
2. Check for missing imports
3. Verify template generates valid Rust syntax
4. Test template with minimal ontology

#### Issue: Weaver validation fails

**Symptoms**: `weaver registry check` fails

**Solutions**:
1. Check for missing required attributes
2. Verify metric/span naming conventions
3. Ensure all instrument types are valid
4. Check for duplicate definitions

#### Issue: SPARQL query returns no results

**Symptoms**: Empty `sparql_results` in context

**Solutions**:
1. Verify ontology is valid Turtle: `npx oxigraph validate ontology.ttl`
2. Check query syntax: Test query in SPARQL playground
3. Verify namespace prefixes match
4. Check for typos in property names

---

## Performance Characteristics

### Template Rendering

- **Small ontology** (<100 triples): <100ms
- **Medium ontology** (100-1000 triples): 100-500ms
- **Large ontology** (>1000 triples): 500ms-2s

### Code Generation Pipeline

```
Parse ontology:        10-50ms
Execute SPARQL:        50-200ms
Render templates:      100-500ms
Validate output:       100-300ms
Format code:           50-100ms
-----------------------------------
Total:                 310-1150ms
```

### Determinism

Generated code is **100% deterministic**:
- Same ontology → Same output
- Timestamps are separate metadata
- No random values
- Reproducible builds

---

## References

### Specifications

- [YAWL Patterns](http://yawlsystem.com/patterns)
- [RDF 1.1 Turtle](https://www.w3.org/TR/turtle/)
- [SPARQL 1.1](https://www.w3.org/TR/sparql11-query/)
- [OpenTelemetry Semantic Conventions](https://opentelemetry.io/docs/specs/semconv/)
- [Weaver Schema](https://github.com/open-telemetry/weaver)

### Tools

- [Handlebars](https://handlebarsjs.com/)
- [oxigraph](https://github.com/oxigraph/oxigraph)
- [Weaver](https://github.com/open-telemetry/weaver)
- [KNHK](https://github.com/seanchatmangpt/knhk)

### Related Documentation

- [KNHK Architecture](../ARCHITECTURE.md)
- [Workflow Engine Guide](../WORKFLOW_ENGINE.md)
- [OTEL Integration](../OTEL_INTEGRATION.md)
- [Testing Strategy](../TESTING.md)

---

**Version**: 1.0.0
**Last Updated**: 2024-11-16
**Status**: Production Ready ✅
