# ggen Quick Start Guide

Get started with ggen template-based code generation in 5 minutes.

## Prerequisites

```bash
# Install Node.js dependencies
npm install -g handlebars-cli

# Install Rust tools (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install oxigraph CLI for SPARQL queries
cargo install oxigraph_cli

# Install Weaver for OTEL validation
pip install weaver-cli
```

## Step 1: Create a Workflow Ontology

Create `my_first_workflow.ttl`:

```turtle
@prefix : <http://example.org/workflows/> .
@prefix yawl: <http://yawl.org/> .
@prefix knhk: <http://knhk.io/> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

:MyFirstWorkflow a yawl:Workflow ;
    rdfs:label "My First Workflow" ;
    yawl:workflowId "my-first-workflow-v1" ;
    rdfs:comment "A simple workflow to get started" .

# Tasks
:StartTask a yawl:Task ;
    rdfs:label "Start Task" ;
    yawl:taskId "start-task" ;
    yawl:taskType "Atomic" ;
    yawl:pattern "Basic" ;
    rdfs:comment "Initial task that starts the workflow" .

:ProcessTask a yawl:Task ;
    rdfs:label "Process Task" ;
    yawl:taskId "process-task" ;
    yawl:taskType "Atomic" ;
    yawl:pattern "Sequence" ;
    rdfs:comment "Main processing task" .

:EndTask a yawl:Task ;
    rdfs:label "End Task" ;
    yawl:taskId "end-task" ;
    yawl:taskType "Atomic" ;
    yawl:pattern "Basic" ;
    rdfs:comment "Final task that completes the workflow" .

# States
:InitialState a yawl:State ;
    rdfs:label "Initial" ;
    yawl:stateType "start" .

:ProcessingState a yawl:State ;
    rdfs:label "Processing" ;
    yawl:stateType "active" .

:CompletedState a yawl:State ;
    rdfs:label "Completed" ;
    yawl:stateType "end" ;
    yawl:isFinal true .

# Transitions
:StartTransition a yawl:Transition ;
    yawl:fromState :InitialState ;
    yawl:toState :ProcessingState ;
    yawl:event "start" .

:CompleteTransition a yawl:Transition ;
    yawl:fromState :ProcessingState ;
    yawl:toState :CompletedState ;
    yawl:event "complete" .
```

## Step 2: Generate Code

```bash
# Generate all code from your workflow ontology
./scripts/ggen/generate-workflows.sh my_first_workflow.ttl

# This creates:
# target/generated/
#   ├── src/
#   │   ├── task_enum.rs       (Task enumeration)
#   │   ├── state_machine.rs   (State machine)
#   │   ├── hooks.rs           (Hook functions)
#   │   └── otel_spans.rs      (OTEL observability)
#   ├── config/
#   │   └── workflow.yaml      (Workflow configuration)
#   └── weaver/
#       └── registry.yaml      (OTEL Weaver schema)
```

## Step 3: Review Generated Code

### Task Enum (`target/generated/src/task_enum.rs`)

```rust
pub enum MyFirstWorkflowTask {
    /// Initial task that starts the workflow
    StartTask,
    /// Main processing task
    ProcessTask,
    /// Final task that completes the workflow
    EndTask,
}

impl MyFirstWorkflowTask {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::StartTask => "Start Task",
            Self::ProcessTask => "Process Task",
            Self::EndTask => "End Task",
        }
    }
}
```

### State Machine (`target/generated/src/state_machine.rs`)

```rust
pub enum MyFirstWorkflowState {
    Initial,
    Processing,
    Completed,
}

pub struct MyFirstWorkflowStateMachine {
    current_state: MyFirstWorkflowState,
    // ...
}

impl MyFirstWorkflowStateMachine {
    pub fn transition(&mut self, event: &str) -> WorkflowResult<()> {
        // Generated transition logic
    }
}
```

## Step 4: Validate Generated Code

```bash
# Validate all generated artifacts
cd target/generated

# 1. Validate Rust syntax
cargo fmt --check src/*.rs

# 2. Validate YAML configuration
yamllint config/*.yaml

# 3. Validate Weaver registry (SOURCE OF TRUTH!)
weaver registry check -r weaver/

# All validation must pass!
```

## Step 5: Integrate with Your Project

```bash
# Option A: Copy to existing crate
cp target/generated/src/* rust/my-workflow-crate/src/

# Option B: Create new crate
cd rust
cargo new my-first-workflow --lib
cp ../target/generated/src/* my-first-workflow/src/

# Add dependencies to Cargo.toml
cd my-first-workflow
cat >> Cargo.toml <<EOF

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
opentelemetry = "0.21"
tracing = "0.1"
EOF

# Build and test
cargo build
cargo test
```

## Step 6: Use Generated Code

```rust
// main.rs
use my_first_workflow::{
    MyFirstWorkflowTask,
    MyFirstWorkflowStateMachine,
    MyFirstWorkflowMetrics,
};

fn main() {
    // Create state machine
    let mut sm = MyFirstWorkflowStateMachine::new();
    println!("Initial state: {:?}", sm.current_state());

    // Execute transitions
    sm.transition("start").expect("Start transition");
    println!("After start: {:?}", sm.current_state());

    sm.transition("complete").expect("Complete transition");
    println!("After complete: {:?}", sm.current_state());

    // Use task enum
    let tasks = MyFirstWorkflowTask::all();
    for task in tasks {
        println!("Task: {}", task.as_str());
    }
}
```

## Next Steps

### Add Guards

Update your ontology to include guard checks:

```turtle
# Add a guard to a task
:ProcessTask knhk:hasGuard :DataValidGuard .

:DataValidGuard a knhk:Guard ;
    rdfs:label "data_valid" ;
    knhk:guardType "ASK_SP" ;
    knhk:expression "data is not null and data.length > 0" ;
    knhk:maxRunLen 8 .
```

Regenerate:
```bash
./scripts/ggen/generate-workflows.sh my_first_workflow.ttl
```

### Add Hooks

Add knowledge hooks for reactive behavior:

```turtle
:OnStartHook a knhk:Hook ;
    rdfs:label "on_start" ;
    knhk:trigger "workflow_started" ;
    knhk:hookType "PreCondition" ;
    rdfs:comment "Executed when workflow starts" .
```

### Add SPARQL Queries

Include SPARQL queries in your hooks:

```turtle
:ValidateDataHook a knhk:Hook ;
    rdfs:label "validate_data" ;
    knhk:sparqlQuery """
        ASK {
            ?data a :ValidData ;
                  :hasValue ?value .
            FILTER(?value > 0)
        }
    """ .
```

### Enable OTEL Observability

The generated code includes full OTEL support:

```rust
use opentelemetry::global;

// Initialize OTEL
let tracer = global::tracer("my-workflow");
let meter = global::meter("my-workflow");

// Use generated metrics
let metrics = MyFirstWorkflowMetrics::new(&meter);
metrics.record_task_execution("start-task", duration, true);
```

### Validate with Weaver

```bash
# Start your application with OTEL export
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317 cargo run

# Validate live telemetry against schema
weaver registry live-check --registry target/generated/weaver/

# This is the ONLY source of truth!
```

## Common Commands

```bash
# Generate code
./scripts/ggen/generate-workflows.sh ontology.ttl

# Validate templates
./scripts/ggen/validate-templates.sh

# Regenerate on changes
make ggen-generate

# Full workflow (validate → generate → test)
make ggen-workflow

# Clean generated code
make ggen-clean
```

## Troubleshooting

**Q: Template rendering fails**
```bash
# Validate template syntax
./scripts/ggen/validate-templates.sh

# Check SPARQL results
npx oxigraph query --file ontology.ttl \
  --query 'SELECT * WHERE { ?s ?p ?o } LIMIT 10' \
  --results-format json
```

**Q: Generated Rust doesn't compile**
```bash
# Format the code
rustfmt target/generated/src/*.rs

# Check for errors
cargo check --manifest-path target/generated/Cargo.toml
```

**Q: Weaver validation fails**
```bash
# Check registry syntax
weaver registry check -r target/generated/weaver/

# Validate specific issues
weaver registry check -r target/generated/weaver/ --verbose
```

## Resources

- **Full Documentation**: [ggen-templates.md](./ggen-templates.md)
- **Template Reference**: [templates/](../../templates/)
- **Examples**: [examples/](../../examples/)
- **KNHK Docs**: [docs/](../../docs/)

## Success Checklist

- [ ] Created workflow ontology (Turtle/RDF)
- [ ] Generated code successfully
- [ ] All validation passes (Rust, YAML, Weaver)
- [ ] Integrated with project
- [ ] Tests pass
- [ ] Weaver live-check passes (SOURCE OF TRUTH!)

**You're now ready to use ggen for production code generation!** 🚀
