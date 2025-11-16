# ggen Templates - Projection Layer (Π)

**Transform ontologies into executable code with deterministic code generation**

## Overview

This directory contains Handlebars templates that implement the projection layer (Π) for KNHK. These templates transform RDF/OWL ontologies (Σ) into production-ready Rust code, YAML configurations, and OpenTelemetry schemas.

### The Projection Function

```
Π: Σ → Code

Where:
  Σ = Ontology (RDF/Turtle, YAWL workflows)
  Π = Template engine (Handlebars + SPARQL)
  Code = {Rust modules, YAML configs, OTEL schemas}
```

This implements **A = μ(O)** - Actions as deterministic projections of observations.

## Directory Structure

```
templates/
├── rust-knhk/              # Rust code generation templates
│   ├── task_enum.rs.hbs    - Task enumerations from YAWL
│   ├── state_machine.rs.hbs- State machine implementation
│   ├── hooks.rs.hbs        - Knowledge hook functions
│   └── otel_spans.rs.hbs   - OpenTelemetry spans/metrics
│
├── config/                 # Configuration generation
│   └── workflow.yaml.hbs   - Complete workflow config
│
├── weaver/                 # OTEL Weaver registry
│   └── registry.yaml.hbs   - Telemetry schema
│
├── sparql/                 # SPARQL integration examples
│   ├── query_bindings.hbs  - Variable binding examples
│   └── integration_example.rs.hbs - Full integration
│
└── README.md               - This file
```

## Quick Start

### 1. Create Workflow Ontology

```turtle
@prefix yawl: <http://yawl.org/> .
@prefix knhk: <http://knhk.io/> .

:MyWorkflow a yawl:Workflow ;
    rdfs:label "My Workflow" .

:Task1 a yawl:Task ;
    rdfs:label "Start" ;
    yawl:pattern "Basic" .
```

### 2. Generate Code

```bash
./scripts/ggen/generate-workflows.sh my_workflow.ttl
```

### 3. Output

```
target/generated/
├── src/
│   ├── task_enum.rs       (Generated Rust enum)
│   ├── state_machine.rs   (Generated state machine)
│   ├── hooks.rs           (Generated hooks)
│   └── otel_spans.rs      (Generated OTEL code)
├── config/
│   └── workflow.yaml      (Generated config)
└── weaver/
    └── registry.yaml      (Generated OTEL schema)
```

## Template Categories

### Rust Code Generation (rust-knhk/)

#### task_enum.rs.hbs
Generates task enumerations from YAWL workflow tasks.

**Input**: SPARQL query results for tasks
**Output**: Rust enum with metadata, helper methods, tests

**Features**:
- Task name → PascalCase enum variant
- YAWL pattern metadata
- Guard function references
- Comprehensive metadata
- Auto-generated tests

#### state_machine.rs.hbs
Generates state machine from workflow graph.

**Input**: States and transitions from ontology
**Output**: State enum, transition logic, guard checks

**Features**:
- State definitions
- Transition validation
- Guard enforcement
- History tracking
- Error handling

#### hooks.rs.hbs
Generates hook functions for knowledge-driven workflows.

**Input**: Hook definitions from ontology
**Output**: Async hook functions with guards

**Features**:
- Pre/post-condition validation
- Guard integration
- SPARQL query execution
- OTEL instrumentation
- Error handling

#### otel_spans.rs.hbs
Generates OpenTelemetry observability code.

**Input**: Tasks, hooks, events
**Output**: Spans, metrics, events

**Features**:
- Span creation helpers
- Metrics registration
- Event recording
- Proper attributes
- Weaver compliance

### Configuration (config/)

#### workflow.yaml.hbs
Generates complete workflow configuration.

**Includes**:
- Task definitions
- State machine config
- Hook settings
- Guard constraints
- MAPE-K configuration
- Performance settings
- Observability config

### Weaver Registry (weaver/)

#### registry.yaml.hbs
Generates OpenTelemetry Weaver schema.

**Must pass**: `weaver registry check`

**Includes**:
- Span definitions
- Metric schemas
- Event definitions
- Attribute requirements
- Validation rules

### SPARQL Integration (sparql/)

Examples demonstrating how SPARQL query results bind to templates.

## Usage

### Basic Generation

```bash
# Generate from single ontology
./scripts/ggen/generate-workflows.sh workflow.ttl

# Generate to specific directory
./scripts/ggen/generate-workflows.sh workflow.ttl target/my-output

# Generate multiple workflows
./scripts/ggen/generate-workflows.sh ontology/workflows/*.ttl
```

### Validation

```bash
# Validate templates
./scripts/ggen/validate-templates.sh

# Validate generated code
cargo fmt --check target/generated/src/*.rs
cargo clippy --manifest-path target/generated/Cargo.toml -- -D warnings

# Validate Weaver registry (SOURCE OF TRUTH!)
weaver registry check -r target/generated/weaver/

# Validate YAML
yamllint target/generated/config/*.yaml
```

### Integration

```bash
# Copy to project
cp target/generated/src/* rust/my-crate/src/

# Build and test
cargo build --package my-crate
cargo test --package my-crate
```

## Template Variables

All templates have access to:

### Common Variables
- `{{workflow_name}}` - Workflow name
- `{{workflow_id}}` - Workflow identifier
- `{{workflow_version}}` - Version string
- `{{generation_timestamp}}` - ISO timestamp
- `{{ontology_path}}` - Source ontology path

### Collection Variables
- `{{#each tasks}}` - Array of tasks
- `{{#each states}}` - Array of states
- `{{#each transitions}}` - Array of transitions
- `{{#each hooks}}` - Array of hooks
- `{{#each guards}}` - Array of guard constraints

### Task Variables
- `{{this.id}}` - Task ID
- `{{this.name}}` - Task name
- `{{this.task_type}}` - Task type
- `{{this.yawl_pattern}}` - YAWL pattern
- `{{this.guards}}` - Guard conditions

See [ggen-templates.md](../../docs/codegen/ggen-templates.md) for complete variable reference.

## SPARQL Integration

Templates consume SPARQL query results automatically.

### Example Query

```sparql
SELECT ?name ?type ?pattern WHERE {
  ?task a yawl:Task ;
        rdfs:label ?name ;
        yawl:taskType ?type ;
        yawl:pattern ?pattern .
}
```

### Template Usage

```handlebars
{{#each sparql_results}}
pub const TASK_{{upper (snake_case this.name)}}: &str = "{{this.name}}";
{{/each}}
```

### Generated Code

```rust
pub const TASK_START_WORKFLOW: &str = "Start Workflow";
pub const TASK_PROCESS_DATA: &str = "Process Data";
```

## Handlebars Helpers

### Built-in
- `{{#each}}` - Iterate arrays
- `{{#if}}` - Conditionals
- `{{this}}` - Current item
- `{{@index}}` - Loop index

### Custom (Project-Specific)
- `{{pascal_case str}}` - PascalCase
- `{{snake_case str}}` - snake_case
- `{{upper str}}` - UPPERCASE
- `{{first_key obj}}` - First key
- `{{first_value obj}}` - First value

## Best Practices

### DO
✅ Include generation timestamp and source
✅ Add "DO NOT EDIT" warnings
✅ Generate comprehensive tests
✅ Use type-safe constructs
✅ Validate with Weaver (SOURCE OF TRUTH!)

### DON'T
❌ Generate placeholder code
❌ Omit error handling
❌ Create overly complex templates
❌ Skip validation steps
❌ Trust text without testing

## Validation Requirements

### Compilation & Code Quality
- [ ] `cargo build` succeeds
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `rustfmt --check` passes
- [ ] All tests pass

### Weaver Validation (MANDATORY)
- [ ] `weaver registry check` passes ✅ SOURCE OF TRUTH
- [ ] `weaver registry live-check` passes (runtime)
- [ ] All telemetry matches schema
- [ ] No undefined spans/metrics/events

### Functional Validation
- [ ] Commands execute with real arguments
- [ ] Expected output produced
- [ ] Performance constraints met (≤8 ticks hot path)

## Performance

### Generation Speed
- Small ontology (<100 triples): <100ms
- Medium ontology (100-1000 triples): 100-500ms
- Large ontology (>1000 triples): 500ms-2s

### Determinism
**100% deterministic**: Same ontology → Same output

No randomness, timestamps are metadata only.

## Documentation

- **Full Guide**: [docs/codegen/ggen-templates.md](../../docs/codegen/ggen-templates.md)
- **Quick Start**: [docs/codegen/QUICKSTART.md](../../docs/codegen/QUICKSTART.md)
- **Examples**: See generated code in `target/generated/`
- **KNHK Docs**: [docs/](../../docs/)

## Examples

See [docs/codegen/ggen-templates.md](../../docs/codegen/ggen-templates.md) for:
- Simple sequential workflow
- Approval workflow with guards
- Parallel split pattern
- SPARQL integration examples

## Troubleshooting

### Template rendering fails
```bash
./scripts/ggen/validate-templates.sh
```

### Generated Rust doesn't compile
```bash
rustfmt target/generated/src/*.rs
cargo check --manifest-path target/generated/Cargo.toml
```

### Weaver validation fails
```bash
weaver registry check -r target/generated/weaver/ --verbose
```

### SPARQL query returns no results
```bash
npx oxigraph validate ontology.ttl
npx oxigraph query --file ontology.ttl --query 'SELECT * WHERE { ?s ?p ?o } LIMIT 10'
```

## References

- [YAWL Patterns](http://yawlsystem.com/patterns)
- [Handlebars](https://handlebarsjs.com/)
- [SPARQL 1.1](https://www.w3.org/TR/sparql11-query/)
- [OpenTelemetry](https://opentelemetry.io/)
- [Weaver](https://github.com/open-telemetry/weaver)

## Version

**Version**: 1.0.0
**Status**: Production Ready ✅
**Last Updated**: 2024-11-16

---

**Never trust the text, only trust test results and Weaver validation** 🔒
