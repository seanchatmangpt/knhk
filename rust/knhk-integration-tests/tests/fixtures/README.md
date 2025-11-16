# Test Fixtures for KNHK Integration Tests

This directory contains test workflow definitions used by the integration test suite.

## Valid Workflows

### simple_valid_workflow.ttl
- **Purpose**: Basic sequential workflow for testing fundamental patterns
- **Pattern**: Sequence (XOR-XOR)
- **Tasks**: 3 simple tasks in sequence
- **Use**: Baseline validation, performance benchmarks

### valid_parallel_workflow.ttl
- **Purpose**: Parallel execution with synchronization
- **Pattern**: Parallel Split + Synchronization (AND-AND)
- **Tasks**: 1 split → 3 parallel branches → 1 sync
- **Use**: Testing concurrent execution patterns

## Invalid Workflows (Anti-Patterns)

### invalid_workflow_unbounded_loop.ttl
- **Violation**: Covenant 5 (Chatman Constant)
- **Problem**: Backward flow without MaxIterations constraint
- **Expected**: Validation should REJECT (unbounded recursion)
- **Use**: Testing Q3 enforcement

### invalid_workflow_bad_pattern.ttl
- **Violation**: Covenant 4 (Pattern Matrix Completeness)
- **Problem**: XOR split with AND join (invalid combination)
- **Expected**: Validation should REJECT (not in permutation matrix)
- **Use**: Testing pattern validation

## Production Workflows

The integration tests also use production workflows from `/ontology/workflows/`:

- **autonomous-work-definition.ttl** - Complete multi-pattern workflow (12 tasks)
- **swift_payment.ttl** - Real-world financial workflow
- **autonomic-self-healing-workflow.ttl** - MAPE-K integration example

## Usage in Tests

```rust
use std::path::PathBuf;

fn fixture_path(filename: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(filename)
}

let workflow = fs::read_to_string(fixture_path("simple_valid_workflow.ttl"))?;
```

## Validation Criteria

All valid workflows MUST:
1. Parse as valid Turtle RDF
2. Use only combinations from pattern permutation matrix
3. Have bounded iteration (MaxIterations ≤ 8)
4. Declare execution modes (sync/async)
5. Have typed data variables
6. Have complete flow graph (start → tasks → end)

All invalid workflows MUST:
1. Parse as valid Turtle (syntax is valid)
2. Fail semantic validation (violate a covenant)
3. Have clear violation documented in comments
