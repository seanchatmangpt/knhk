# Van der Aalst End-to-End Validation Framework

## Overview

The Van der Aalst End-to-End Validation Framework provides comprehensive validation for the workflow engine based on Wil M.P. van der Aalst's process mining methodology. The framework validates workflows across five dimensions:

1. **Fitness** - Can the process actually be executed?
2. **Precision** - Does the process match the specification?
3. **Generalization** - Does the process work beyond the examples?
4. **Process Mining** - XES event log analysis and conformance checking
5. **Formal Verification** - State transitions, deadlock freedom, termination

## Architecture

### Core Components

- **ValidationFramework** (`rust/knhk-workflow-engine/src/validation/framework.rs`) - Orchestrates all validation phases
- **FitnessValidator** (`rust/knhk-workflow-engine/src/validation/fitness.rs`) - Validates workflow execution
- **PrecisionValidator** (`rust/knhk-workflow-engine/src/validation/precision.rs`) - Validates specification match
- **GeneralizationValidator** (`rust/knhk-workflow-engine/src/validation/generalization.rs`) - Validates beyond examples
- **ProcessMiningAnalyzer** (`rust/knhk-workflow-engine/src/validation/process_mining.rs`) - Analyzes XES event logs
- **FormalVerifier** (`rust/knhk-workflow-engine/src/validation/formal.rs`) - Verifies formal properties
- **ValidationReport** (`rust/knhk-workflow-engine/src/validation/report.rs`) - Generates comprehensive reports

## Usage

### CLI Command

Run complete validation framework:

```bash
knhk workflow validate --spec-id <spec_id> [--output-dir <dir>] [--format <format>]
```

Run specific validation phase:

```bash
knhk workflow validate --spec-id <spec_id> --phase <phase> [--output-dir <dir>] [--format <format>]
```

**Phases**:
- `fitness` - Fitness validation
- `precision` - Precision validation
- `generalization` - Generalization validation
- `process_mining` - Process mining analysis
- `formal` - Formal verification

**Formats**:
- `markdown` (default) - Markdown report
- `json` - JSON report
- `html` - HTML report

**Examples**:

```bash
# Run complete validation
knhk workflow validate --spec-id abc-123-def --output-dir ./validation_results

# Run fitness phase only
knhk workflow validate --spec-id abc-123-def --phase fitness

# Generate JSON report
knhk workflow validate --spec-id abc-123-def --format json
```

### Programmatic Usage

```rust
use knhk_workflow_engine::{
    validation::ValidationFramework,
    WorkflowEngine,
    StateStore,
};
use std::sync::Arc;

// Create engine
let state_store = Arc::new(StateStore::new("./workflow_db")?);
let engine = Arc::new(WorkflowEngine::new(state_store));

// Create framework
let framework = ValidationFramework::new(engine.clone());

// Run complete validation
let report = framework.run_complete_validation(spec_id).await?;

// Generate report
let markdown = report.to_markdown();
let json = report.to_json()?;
let html = report.to_html();
```

## Validation Phases

### 1. Fitness Validation

**Tests**:
- Simple workflow execution
- Event log collection
- Pattern execution (all 43 patterns)

**Metrics**:
- Execution success rate
- Event log completeness
- Pattern coverage

### 2. Precision Validation

**Tests**:
- Specification comparison
- Pattern semantics verification
- State transition verification

**Metrics**:
- Specification match rate
- Semantic correctness
- State transition validity

### 3. Generalization Validation

**Tests**:
- Varied input testing
- Edge case handling
- Load testing (delegated)

**Metrics**:
- Input variety coverage
- Edge case handling rate
- Performance under load

### 4. Process Mining Analysis

**Tests**:
- XES import and validation
- Process discovery (Alpha+++)
- Conformance checking

**Metrics**:
- Fitness (0.0 - 1.0)
- Precision (0.0 - 1.0)
- Petri net structure

### 5. Formal Verification

**Tests**:
- State transition verification
- Deadlock detection
- Termination verification

**Metrics**:
- Valid state transitions
- Deadlock freedom
- Termination rate

## Report Format

### Markdown Report

```markdown
# Van der Aalst Validation Report

**Workflow Spec ID**: abc-123-def
**Timestamp**: 2025-01-XX

## Summary

- **Overall Status**: Pass
- **Phases**: 5 passed / 0 failed / 5 total
- **Tests**: 25 passed / 0 failed / 25 total
- **Warnings**: 0

## Phase Results

### fitness
**Status**: Pass
- Passed: 3
- Failed: 0
- Warnings: 0
- Skipped: 0

...
```

### JSON Report

```json
{
  "spec_id": "abc-123-def",
  "timestamp": "2025-01-XX",
  "phases": {
    "fitness": {
      "phase": "fitness",
      "status": "Pass",
      "passed": 3,
      "failed": 0,
      "warnings": 0,
      "skipped": 0,
      "details": [...],
      "metrics": {}
    },
    ...
  },
  "summary": {
    "total_phases": 5,
    "passed_phases": 5,
    "failed_phases": 0,
    "total_tests": 25,
    "passed_tests": 25,
    "failed_tests": 0,
    "warnings": 0,
    "overall_status": "Pass"
  }
}
```

## Integration

### CI/CD Integration

Add to CI/CD pipeline:

```yaml
- name: Van der Aalst Validation
  run: |
    knhk workflow validate --spec-id $WORKFLOW_SPEC_ID --output-dir ./validation_results --format json
    # Check validation status
    if [ "$(jq -r '.summary.overall_status' validation_results/validation_report.json)" != "Pass" ]; then
      echo "Validation failed"
      exit 1
    fi
```

### Test Integration

Use in tests:

```rust
#[tokio::test]
async fn test_workflow_validation() {
    let engine = create_test_engine();
    let spec = create_test_workflow();
    engine.register_workflow(spec.clone()).await.unwrap();

    let framework = ValidationFramework::new(engine);
    let report = framework.run_complete_validation(spec.id).await.unwrap();

    assert_eq!(report.summary.overall_status, ValidationStatus::Pass);
}
```

## Status

**Status**: ✅ COMPLETE - Framework implemented and tested

**Test Results**: ✅ All tests pass

**Next Steps**: 
- Expand pattern testing coverage
- Add more detailed metrics
- Integrate with CI/CD pipelines

---

**Last Updated**: 2025-01-XX  
**Status**: ✅ COMPLETE - Van der Aalst End-to-End Validation Framework

