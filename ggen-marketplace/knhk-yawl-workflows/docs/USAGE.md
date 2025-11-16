# KNHK YAWL Workflow Generator - Usage Guide

Generate YAWL XML specifications from RDF/Turtle workflow ontologies using ggen.

## Table of Contents

- [Quick Start](#quick-start)
- [Installation](#installation)
- [Basic Usage](#basic-usage)
- [Advanced Usage](#advanced-usage)
- [Examples](#examples)
- [Troubleshooting](#troubleshooting)

## Quick Start

Generate a YAWL workflow in 3 steps:

```bash
# 1. Install template
ggen marketplace install io.knhk.yawl-workflows

# 2. Create or prepare your workflow RDF file
cat > workflow.ttl << 'EOF'
@prefix yawl: <http://bitflow.ai/ontology/yawl/v2#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

<http://example.org/workflow/my-process> a yawl:WorkflowSpecification ;
    rdfs:label "My Workflow" ;
    yawl:hasTask <http://example.org/task/task1>, <http://example.org/task/task2> .

<http://example.org/task/task1> a yawl:Task ;
    rdfs:label "First Task" ;
    yawl:hasSplitType yawl:AND ;
    yawl:hasJoinType yawl:AND .

<http://example.org/task/task2> a yawl:Task ;
    rdfs:label "Second Task" .
EOF

# 3. Generate YAWL workflow
ggen template generate-rdf \
  --ontology workflow.ttl \
  --template io.knhk.yawl-workflows \
  --output workflow.yawl
```

Output: `workflow.yawl` - YAWL 2.2 XML specification

## Installation

### Option 1: Via ggen marketplace (Recommended)

```bash
ggen marketplace install io.knhk.yawl-workflows
```

### Option 2: Manual installation

1. Clone the knhk repository:
```bash
git clone https://github.com/seanchatmangpt/knhk.git
cd knhk/ggen-marketplace/knhk-yawl-workflows
```

2. Install locally:
```bash
ggen marketplace install --path .
```

## Basic Usage

### Generate from RDF/Turtle

```bash
ggen template generate-rdf \
  --ontology my-workflow.ttl \
  --template io.knhk.yawl-workflows
```

**Output**: `output.yawl` (YAWL 2.2 XML format)

### Generate JSON format

```bash
ggen template generate-rdf \
  --ontology my-workflow.ttl \
  --template io.knhk.yawl-workflows \
  --output workflow.json
```

### Custom output filename

```bash
ggen template generate-rdf \
  --ontology my-workflow.ttl \
  --template io.knhk.yawl-workflows \
  --output my-process-spec.yawl
```

## Advanced Usage

### Batch generation (multiple workflows)

```bash
# Generate YAWL from all RDF files in a directory
for file in workflows/*.ttl; do
  ggen template generate-rdf \
    --ontology "$file" \
    --template io.knhk.yawl-workflows \
    --output "generated/${file%.ttl}.yawl"
done
```

### Validate generated YAWL

```bash
# Check YAWL specification with Weaver validation
weaver registry check -r registry/

# Live check against running workflow engine
weaver registry live-check --registry registry/
```

### Watch mode (auto-regenerate)

```bash
# Automatically regenerate on file changes
ggen project watch --template io.knhk.yawl-workflows
```

### Merge ontologies before generation

```bash
# Combine multiple RDF files
cat base-ontology.ttl workflow-ontology.ttl > merged.ttl

# Generate from merged ontology
ggen template generate-rdf \
  --ontology merged.ttl \
  --template io.knhk.yawl-workflows
```

## Examples

### Example 1: Simple Sequence

**Input** (`simple-sequence.ttl`):
```turtle
@prefix yawl: <http://bitflow.ai/ontology/yawl/v2#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

<http://example.org/workflow/sequence> a yawl:WorkflowSpecification ;
    rdfs:label "Simple Sequence" ;
    yawl:hasStartCondition <http://example.org/cond/start> ;
    yawl:hasEndCondition <http://example.org/cond/end> .

<http://example.org/task/t1> a yawl:Task ;
    rdfs:label "Task 1" ;
    yawl:hasOutgoingFlow <http://example.org/cond/middle> .

<http://example.org/task/t2> a yawl:Task ;
    rdfs:label "Task 2" ;
    yawl:hasIncomingFlow <http://example.org/cond/middle> .
```

**Generation**:
```bash
ggen template generate-rdf \
  --ontology simple-sequence.ttl \
  --template io.knhk.yawl-workflows \
  --output sequence.yawl
```

### Example 2: Parallel Split

```turtle
@prefix yawl: <http://bitflow.ai/ontology/yawl/v2#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

<http://example.org/workflow/parallel> a yawl:WorkflowSpecification ;
    rdfs:label "Parallel Split Pattern" .

<http://example.org/task/split> a yawl:Task ;
    rdfs:label "Split Task" ;
    yawl:hasSplitType yawl:OR ;
    yawl:hasOutgoingFlow <http://example.org/cond/path1>, <http://example.org/cond/path2> .

<http://example.org/task/task-a> a yawl:Task ;
    rdfs:label "Task A" ;
    yawl:hasIncomingFlow <http://example.org/cond/path1> ;
    yawl:hasOutgoingFlow <http://example.org/cond/sync> .

<http://example.org/task/task-b> a yawl:Task ;
    rdfs:label "Task B" ;
    yawl:hasIncomingFlow <http://example.org/cond/path2> ;
    yawl:hasOutgoingFlow <http://example.org/cond/sync> .

<http://example.org/task/join> a yawl:Task ;
    rdfs:label "Join Task" ;
    yawl:hasJoinType yawl:AND ;
    yawl:hasIncomingFlow <http://example.org/cond/sync> .
```

### Example 3: Exclusive Choice (XOR Gateway)

```turtle
<http://example.org/task/decision> a yawl:Task ;
    rdfs:label "Decision Point" ;
    yawl:hasSplitType yawl:XOR ;
    yawl:hasOutgoingFlow <http://example.org/cond/path-yes>, <http://example.org/cond/path-no> .

<http://example.org/task/approved> a yawl:Task ;
    rdfs:label "Approved Path" ;
    yawl:hasIncomingFlow <http://example.org/cond/path-yes> ;
    yawl:flowCondition "approved == true" .

<http://example.org/task/rejected> a yawl:Task ;
    rdfs:label "Rejected Path" ;
    yawl:hasIncomingFlow <http://example.org/cond/path-no> ;
    yawl:flowCondition "approved == false" .
```

## Supported YAWL Patterns

This template supports all 43 YAWL control flow patterns:

### Basic Patterns
- Sequence
- Parallel Split (AND)
- Synchronization (AND Join)
- Exclusive Choice (XOR)
- Simple Merge

### Advanced Patterns
- Multi-Choice (OR)
- Synchronizing Merge
- Multiple Merge
- Discriminator

### Structural Patterns
- Arbitrary Cycles (loops)
- Implicit Termination
- Deferred Choice
- Interleaved Parallel Routing

### Additional Patterns
- Cancellation (cancel task, cancel case, cancel region)
- Iteration and Recursion
- Milestone patterns
- Critical sections

## Output Formats

### YAWL XML (Default)

Generated YAWL 2.2 XML specification, ready for:
- YAWL Worklist Handler
- knhk workflow engine
- Process modeling tools supporting YAWL

### YAWL JSON

Alternative JSON format containing:
- Workflow metadata
- Task definitions with routing
- Conditions/places
- Flow connections
- Pattern information

## Integration

### With knhk Workflow Engine

```rust
use knhk_workflow_engine::{WorkflowEngine, WorkflowParser};

// Load generated YAWL
let mut parser = WorkflowParser::new()?;
let spec = parser.parse_file("workflow.yawl")?;

// Execute in KNHK engine
let engine = WorkflowEngine::new(state_store);
engine.register_workflow(spec).await?;
```

### With KNHK CLI

```bash
# Boot with generated YAWL
knhk boot init workflow.yawl invariants.sparql

# Run workflow pipeline
knhk pipeline run --connectors kafka-prod
```

## Troubleshooting

### "Template not found"

```bash
# List installed templates
ggen template list | grep yawl

# Reinstall
ggen marketplace install io.knhk.yawl-workflows --force
```

### "Invalid RDF/Turtle"

Validate your Turtle syntax:

```bash
# Check Turtle validity
ggen graph load workflow.ttl

# Query to verify structure
ggen graph query --ontology workflow.ttl --sparql \
  'SELECT ?s WHERE { ?s a ?type } LIMIT 5'
```

### "Missing workflow namespace"

Ensure your Turtle file includes required namespaces:

```turtle
@prefix yawl: <http://bitflow.ai/ontology/yawl/v2#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
```

### "SPARQL query failed"

Debug with:

```bash
# Load RDF first
ggen graph load workflow.ttl

# Test individual SPARQL queries
ggen graph query --ontology workflow.ttl --sparql \
  "PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>
   SELECT ?workflow WHERE { ?workflow a yawl:WorkflowSpecification }"
```

## Support

- **Documentation**: https://github.com/seanchatmangpt/knhk/tree/main/docs
- **Issues**: https://github.com/seanchatmangpt/knhk/issues
- **YAWL Spec**: http://www.yawlfoundation.org/
- **RDF/Turtle**: https://www.w3.org/TR/turtle/

## Next Steps

1. [Ontology Guide](../docs/ONTOLOGY.md) - Learn to create workflow ontologies
2. [KNHK Workflow Engine](../../rust/knhk-workflow-engine/README.md) - Execute generated workflows
3. [YAWL Integration](../../docs/YAWL_INTEGRATION.md) - Deep dive into YAWL support
