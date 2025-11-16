# KNHK YAWL Generator - Complete Examples

Comprehensive examples demonstrating all major YAWL patterns.

## Table of Contents

1. [Simple Sequence](#simple-sequence)
2. [Parallel Split](#parallel-split)
3. [Exclusive Choice (XOR)](#exclusive-choice-xor)
4. [Multi-Choice (OR)](#multi-choice-or)
5. [Synchronizing Merge](#synchronizing-merge)

## Simple Sequence

**Pattern**: Sequential task execution (Task1 → Task2 → Task3)

**File**: `examples/simple-sequence.ttl`

**Description**: The most basic control flow pattern where tasks execute one after another in a fixed sequence.

**Usage**:

```bash
ggen template generate-rdf \
  --ontology examples/simple-sequence.ttl \
  --template io.knhk.yawl-workflows \
  --output sequence.yawl
```

**Structure**:

```
┌──────────┐     ┌──────────┐     ┌──────────┐
│ Task 1   │ --> │ Task 2   │ --> │ Task 3   │
└──────────┘     └──────────┘     └──────────┘
```

**RDF Key Points**:
- All tasks use `yawl:AND` split and join types
- Connections via `yawl:hasOutgoingFlow` and `yawl:hasIncomingFlow`
- Start condition has `yawl:initialMarking 1`
- Intermediate conditions have `yawl:initialMarking 0`

**Generated YAWL** (excerpt):

```xml
<Specification>
  <Net id="workflow_net">
    <Place id="start">
      <Name>Start</Name>
      <Token id="Default" value="True" />
    </Place>

    <Transition id="task1">
      <Name>Task 1</Name>
    </Transition>

    <Arc source="start" target="task1" />
    <Arc source="task1" target="c1" />
    <!-- ... -->
  </Net>
</Specification>
```

## Parallel Split

**Pattern**: Multiple concurrent task execution

**File**: `examples/parallel-split.ttl`

**Description**: After a split task, multiple tasks execute in parallel. A join task waits for all parallel tasks to complete before proceeding.

**Usage**:

```bash
ggen template generate-rdf \
  --ontology examples/parallel-split.ttl \
  --template io.knhk.yawl-workflows \
  --output parallel.yawl
```

**Structure**:

```
         ┌──────────┐
         │ Task A   │
         └──────────┘
            /    \
    ┌──────┐      ┌──────┐
    │Split │      │  Sync│
    │(OR)  │      │(AND) │
    └──────┘      └──────┘
            \    /
         ┌──────────┐
         │ Task B   │
         └──────────┘
         ┌──────────┐
         │ Task C   │
         └──────────┘
```

**Key RDF Concepts**:

1. **OR Split**: Task with `yawl:hasSplitType yawl:OR`
   - Creates multiple parallel paths
   - All outgoing flows are activated

2. **AND Join**: Task with `yawl:hasJoinType yawl:AND`
   - Waits for ALL inputs before proceeding
   - Synchronizes parallel paths

3. **Multiple Inputs**: Condition with multiple incoming flows
   ```turtle
   <http://example.org/cond/sync-input>
       yawl:hasIncomingFlow <http://example.org/task/task-a>,
                            <http://example.org/task/task-b>,
                            <http://example.org/task/task-c> .
   ```

**Generated YAWL** (excerpt):

```xml
<Transition id="split_task">
  <Name>Split Task</Name>
  <!-- OR split creates multiple paths -->
  <PreSet>
    <Arc source="cond1" target="split_task" />
  </PreSet>
  <PostSet>
    <Arc source="split_task" target="path_a" />
    <Arc source="split_task" target="path_b" />
    <Arc source="split_task" target="path_c" />
  </PostSet>
</Transition>

<Transition id="join_task">
  <Name>Join Task</Name>
  <!-- AND join synchronizes parallel paths -->
  <PreSet>
    <Arc source="path_a" target="join_task" />
    <Arc source="path_b" target="join_task" />
    <Arc source="path_c" target="join_task" />
  </PreSet>
</Transition>
```

## Exclusive Choice (XOR)

**Pattern**: Conditional branching (one-of-many decision)

**File**: `examples/exclusive-choice.ttl`

**Description**: A decision point where exactly ONE of multiple paths is taken based on conditions.

**Usage**:

```bash
ggen template generate-rdf \
  --ontology examples/exclusive-choice.ttl \
  --template io.knhk.yawl-workflows \
  --output xor.yawl
```

**Structure**:

```
              ┌──────────┐
              │ Decision │
              │(XOR Split)
              └──────────┘
               /        \
    ┌────────────┐    ┌──────────┐
    │ Approved   │    │ Rejected │
    │(if true)   │    │(if false)│
    └────────────┘    └──────────┘
               \        /
              ┌──────────┐
              │ Complete │
              └──────────┘
```

**Key RDF Concepts**:

1. **XOR Split**: Task with `yawl:hasSplitType yawl:XOR`
   - Exactly ONE of multiple paths activates
   - Based on flow predicates

2. **Flow Predicates**: Conditions on arcs
   ```turtle
   <http://example.org/task/approved> yawl:flowCondition "decision == true" .
   <http://example.org/task/rejected> yawl:flowCondition "decision == false" .
   ```

3. **Simple Merge**: Join without synchronization
   - Multiple inputs can activate
   - No waiting for all inputs
   - Each input produces one output

**Generated YAWL** (excerpt):

```xml
<Transition id="decision">
  <Name>Decision Point</Name>
  <PreSet>
    <Arc source="decision_input" target="decision" />
  </PreSet>
  <PostSet>
    <Arc source="decision" target="path_approved">
      <Predicate>decision == true</Predicate>
    </Arc>
    <Arc source="decision" target="path_rejected">
      <Predicate>decision == false</Predicate>
    </Arc>
  </PostSet>
</Transition>
```

## Multi-Choice (OR)

**Pattern**: One-or-more path selection

**Description**: Similar to exclusive choice, but allows multiple paths to be taken. Use when:
- Multiple paths CAN be executed in parallel
- Not all possible combinations are valid
- Conditions determine which combinations

**Key Difference from Parallel**:
- Parallel (AND): ALL paths MUST execute
- Multi-Choice (OR): ONE OR MORE paths EXECUTE
- Exclusive Choice (XOR): EXACTLY ONE path executes

**RDF Pattern**:

```turtle
<http://example.org/task/multi-choice> a yawl:Task ;
    yawl:hasSplitType yawl:OR ;    # One or more paths
    yawl:hasOutgoingFlow <http://example.org/cond/path1>,
                         <http://example.org/cond/path2>,
                         <http://example.org/cond/path3> .
```

## Synchronizing Merge

**Pattern**: Convergence of multiple paths with synchronization

**Description**: Multiple paths converge, and when any path completes, the join task activates. Used for:
- Deferred choice patterns
- Interleaved parallel routing
- Complex synchronization patterns

**Key RDF Concepts**:

1. **Discriminator Join**: AND join that activates on FIRST input
   ```turtle
   <http://example.org/task/discriminator> a yawl:Task ;
       yawl:hasJoinType yawl:Discriminator ;
       yawl:hasIncomingFlow <http://example.org/cond/path1>,
                            <http://example.org/cond/path2>,
                            <http://example.org/cond/path3> .
   ```

2. **Multiple Merge**: Activates on ANY input (not synchronizing)
   ```turtle
   <http://example.org/task/multiple-merge> a yawl:Task ;
       yawl:hasJoinType yawl:MultipleMerge ;
       yawl:hasIncomingFlow <http://example.org/cond/path1>,
                            <http://example.org/cond/path2> .
   ```

## Running Examples

### Generate All Examples

```bash
cd ggen-marketplace/knhk-yawl-workflows

# Simple sequence
ggen template generate-rdf \
  --ontology examples/simple-sequence.ttl \
  --template io.knhk.yawl-workflows \
  --output examples/output/sequence.yawl

# Parallel split
ggen template generate-rdf \
  --ontology examples/parallel-split.ttl \
  --template io.knhk.yawl-workflows \
  --output examples/output/parallel.yawl

# Exclusive choice
ggen template generate-rdf \
  --ontology examples/exclusive-choice.ttl \
  --template io.knhk.yawl-workflows \
  --output examples/output/exclusive.yawl
```

### Validate Generated YAWL

```bash
# Check YAWL syntax
ggen template lint examples/output/sequence.yawl

# Query generated structure
ggen graph load examples/output/sequence.yawl  # If YAWL is RDF-serializable
```

### Execute with KNHK

```rust
use knhk_workflow_engine::{WorkflowEngine, WorkflowParser};

// Load generated YAWL
let mut parser = WorkflowParser::new()?;
let spec = parser.parse_file("examples/output/sequence.yawl")?;

// Execute
let engine = WorkflowEngine::new(state_store);
engine.register_workflow(spec).await?;
engine.start_case(workflow_id, context).await?;
```

## Pattern Mapping: RDF → YAWL

| YAWL Pattern | Split Type | Join Type | Conditions |
|--------------|-----------|-----------|-----------|
| **Sequence** | AND | AND | Single flow |
| **Parallel Split** | OR | AND | Multiple flows, AND join |
| **Exclusive Choice** | XOR | Simple | Multiple flows with predicates |
| **Multi-Choice** | OR | Simple | Multiple flows (no sync) |
| **Synchronizing Merge** | - | AND | Multiple inputs, AND join |
| **Discriminator** | - | Discriminator | First-one-wins |
| **Multiple Merge** | - | Multiple | Any input activates |

## Advanced Patterns

### With Conditions

```turtle
<http://example.org/flow> yawl:flowCondition "amount > 1000" .
```

### With Timeout

```turtle
<http://example.org/task/approval> yawl:timeoutDuration "PT24H"^^xsd:duration .
```

### With Multi-Instance

```turtle
<http://example.org/task/notify-users>
    yawl:isMultiInstance true ;
    yawl:multiInstanceCollection "users" ;
    yawl:multiInstanceIterator "user" .
```

## Testing Your Examples

```bash
# Validate RDF
ggen graph load your-workflow.ttl

# Check SPARQL extraction
ggen graph query --ontology your-workflow.ttl --sparql \
  'PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>
   SELECT ?task WHERE { ?task a yawl:Task }'

# Generate YAWL
ggen template generate-rdf \
  --ontology your-workflow.ttl \
  --template io.knhk.yawl-workflows

# Verify output
ls -la *.yawl
cat *.yawl  # Check generated YAWL XML
```

## Next Steps

1. Start with **Simple Sequence** to understand basics
2. Explore **Parallel Split** for concurrent execution
3. Try **Exclusive Choice** for conditional logic
4. Build your own workflow by combining patterns
5. See [YAWL Integration Guide](../../docs/YAWL_INTEGRATION.md) for 43-pattern reference

---

For detailed usage instructions, see [USAGE.md](USAGE.md).
