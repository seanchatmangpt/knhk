# Chicago TDD Test Suite for Self-Executing Workflows

Comprehensive Chicago-style Test-Driven Development test suite for KNHK self-executing workflows.

## Overview

This test suite validates all aspects of KNHK self-executing workflows following Chicago TDD methodology:

- **State-based testing** with real collaborators (no mocks)
- **Behavior verification** testing what the code does, not how
- **AAA pattern** (Arrange, Act, Assert) for clarity
- **Production paths** using actual implementations

## Test Structure

```
tests/
├── chicago/                    # Chicago TDD unit/component tests
│   ├── mod.rs                 # Test module organization
│   ├── pattern_sequence_test.rs         # Pattern 1: Sequence
│   ├── pattern_parallel_split_test.rs   # Pattern 2: Parallel Split
│   ├── hook_engine_test.rs              # Hook lifecycle tests
│   ├── guard_enforcement_test.rs        # Guard constraint tests
│   ├── receipt_generation_test.rs       # Lockchain receipt tests
│   ├── mape_k_monitor_test.rs           # MAPE-K Monitor phase
│   ├── mape_k_analyze_test.rs           # MAPE-K Analyze phase
│   └── snapshot_system_test.rs          # State persistence tests
├── integration/                # End-to-end integration tests
│   ├── mod.rs                 # Integration test organization
│   ├── workflow_end_to_end_test.rs      # Complete workflow pipeline
│   └── mape_k_feedback_loop_test.rs     # MAPE-K autonomic cycles
└── fixtures/                   # Test data and fixtures
    └── example_workflows.ttl   # Example workflow ontologies
```

## Test Categories

### 1. Pattern Tests

Tests for YAWL workflow patterns:

- **Sequence (Pattern 1)**: Linear task execution
- **Parallel Split (Pattern 2)**: Concurrent task execution
- Additional patterns: XOR, OR, loops, etc. (expandable)

```bash
cargo test --test pattern_sequence_test
cargo test --test pattern_parallel_split_test
```

### 2. Hook Engine Tests

Tests for workflow lifecycle hooks:

- Before/after task execution
- Before/after case creation/completion
- Hook priority ordering
- Hook data modification
- Hook execution blocking

```bash
cargo test --test hook_engine_test
```

### 3. Guard Enforcement Tests

Tests for constraint validation and guards:

- Tick budget enforcement (Chatman Constant ≤8 ticks)
- Cardinality constraints
- Schema validation
- Authorization guards
- Type validation
- Uniqueness constraints

```bash
cargo test --test guard_enforcement_test
```

### 4. Receipt Generation Tests

Tests for execution receipt generation (lockchain integration):

- Receipt generation and persistence
- Timestamp tracking
- Task execution details
- XES export validation
- Process mining compatibility
- Receipt immutability
- Parallel execution merging

```bash
cargo test --test receipt_generation_test
```

### 5. MAPE-K Tests

Tests for autonomic Monitor-Analyze-Plan-Execute-Knowledge loop:

**Monitor Phase**:
- Execution tracking
- Performance metrics collection
- State transition detection
- Parallel execution monitoring
- Telemetry export

```bash
cargo test --test mape_k_monitor_test
```

**Analyze Phase**:
- Performance bottleneck identification
- Pattern detection
- Success metrics calculation
- Optimization opportunity identification
- SLO compliance evaluation

```bash
cargo test --test mape_k_analyze_test
```

### 6. Snapshot System Tests

Tests for workflow state persistence:

- Case state persistence
- Recovery from snapshots
- Data consistency
- Point-in-time recovery
- Concurrent update safety
- Execution history preservation
- Audit trail generation

```bash
cargo test --test snapshot_system_test
```

### 7. Integration Tests

End-to-end workflow execution tests:

**End-to-End Tests**:
- Simple workflow execution
- Parallel workflow execution
- Multi-case batch execution
- Hook integration
- Process mining export
- Error handling
- Performance validation

```bash
cargo test --test workflow_end_to_end_test
```

**MAPE-K Feedback Loop Tests**:
- Complete feedback loop execution
- Monitor phase integration
- Analyze phase integration
- Plan and execute phases
- Performance improvement
- Anomaly handling
- SLO compliance
- Continuous improvement

```bash
cargo test --test mape_k_feedback_loop_test
```

## Running Tests

### Run All Chicago TDD Tests

```bash
# From workflow engine directory
cd rust/knhk-workflow-engine
cargo test

# Or from project root
cargo test --package knhk-workflow-engine
```

### Run Specific Test Modules

```bash
# Pattern tests
cargo test pattern_sequence
cargo test pattern_parallel_split

# Component tests
cargo test hook_engine
cargo test guard_enforcement
cargo test receipt_generation
cargo test snapshot_system

# MAPE-K tests
cargo test mape_k_monitor
cargo test mape_k_analyze

# Integration tests
cargo test workflow_end_to_end
cargo test mape_k_feedback_loop
```

### Run Tests with Output

```bash
cargo test -- --nocapture --test-threads=1
```

### Run Tests in Release Mode

```bash
cargo test --release
```

## Test Fixtures

Test fixtures are located in `tests/fixtures/`:

- `example_workflows.ttl` - Example workflow definitions in Turtle/RDF format
  - Simple sequential workflow (order processing)
  - Parallel split workflow (loan approval)
  - XOR split workflow (customer support)
  - Multiple instance workflow (document review)
  - Workflow with guards (secure transaction)

## Coverage Goals

| Category | Target Coverage |
|----------|----------------|
| Hot Path (≤8 ticks) | 100% |
| Pattern Execution | 100% |
| Hook System | 90%+ |
| Guard Validation | 90%+ |
| Receipt Generation | 95%+ |
| MAPE-K Monitor | 90%+ |
| MAPE-K Analyze | 85%+ |
| Snapshot System | 90%+ |
| Integration E2E | 85%+ |

## Key Principles

### Chicago TDD Methodology

1. **State-Based Testing**: Verify state changes, not implementation
2. **Real Collaborators**: Use actual implementations, not mocks
3. **Behavior Focus**: Test what the code does, not how it does it
4. **Production Paths**: Exercise actual production code paths
5. **Error Handling**: Cover error paths and guard violations

### AAA Pattern

All tests follow Arrange-Act-Assert pattern:

```rust
#[tokio::test]
async fn test_example() -> WorkflowResult<()> {
    // Arrange: Set up test state
    let mut fixture = WorkflowTestFixture::new()?;
    let workflow = create_simple_sequential_workflow(...);
    let spec_id = fixture.register_workflow(workflow).await?;

    // Act: Execute the operation
    let case_id = fixture.create_case(spec_id, data).await?;
    let case = fixture.execute_case(case_id).await?;

    // Assert: Verify the outcome
    fixture.assert_case_completed(&case);
    assert_eq!(case.data["field"], expected_value);

    fixture.cleanup()?;
    Ok(())
}
```

### No Mocking of Domain Logic

Chicago TDD uses real implementations for collaborators:

- ✅ Real workflow engine
- ✅ Real state store (with test isolation)
- ✅ Real pattern registry
- ✅ Real hook system
- ❌ No mocked repositories
- ❌ No stubbed services

## Validation Hierarchy

Tests follow KNHK's validation hierarchy:

**Level 1: Weaver Schema Validation** (External to this suite)
- OpenTelemetry schema validation
- Runtime telemetry conformance
- Source of truth for feature validation

**Level 2: Compilation & Code Quality**
- `cargo build --release`
- `cargo clippy -- -D warnings`
- Zero warnings policy

**Level 3: Chicago TDD Tests** (This suite)
- Comprehensive behavior validation
- State-based verification
- Supporting evidence for features

## Adding New Tests

### 1. Create Test File

```rust
//! Chicago TDD Tests: [Feature Name]
//!
//! Tests [description of what is being tested].
//! AAA Pattern: Arrange, Act, Assert

use knhk_workflow_engine::case::CaseState;
use knhk_workflow_engine::error::WorkflowResult;
use knhk_workflow_engine::testing::chicago_tdd::*;

#[tokio::test]
async fn test_feature_behavior() -> WorkflowResult<()> {
    // Arrange
    let mut fixture = WorkflowTestFixture::new()?;

    // Act
    let result = fixture.execute_something().await?;

    // Assert
    assert!(result.is_valid());

    fixture.cleanup()?;
    Ok(())
}
```

### 2. Add to Module

Update `tests/chicago/mod.rs` or `tests/integration/mod.rs`:

```rust
mod new_feature_test;
```

### 3. Run Tests

```bash
cargo test new_feature_test
```

## Continuous Integration

These tests run automatically in CI/CD:

```bash
# CI test command
cargo test --workspace --all-features
cargo clippy --workspace -- -D warnings
```

## Related Documentation

- [CLAUDE.md](../../CLAUDE.md) - KNHK testing strategy
- [Testing README](../README.md) - Overall test suite documentation
- [Chicago TDD Framework](../../rust/knhk-workflow-engine/src/testing/chicago_tdd.rs) - Test framework source

## License

MIT License - See LICENSE file in project root
