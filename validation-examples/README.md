# SHACL Validation Examples for YAWL Turtle Workflows

This directory contains example workflows for testing SHACL validation against Q invariants (Covenant 2: Invariants Are Law).

## Directory Structure

```
validation-examples/
├── valid/                    # Workflows that pass all validations
│   ├── simple-workflow.ttl   # Basic sequential workflow
│   └── parallel-workflow.ttl # Parallel split-join with resource bounds
├── invalid/                  # Workflows that violate Q invariants
│   ├── unbounded-recursion.ttl        # Violates Q3 (Chatman constant)
│   ├── type-mismatch.ttl              # Violates Q2 (type soundness)
│   └── missing-resource-bounds.ttl    # Violates Q5 (resource bounds)
└── README.md                 # This file
```

## Q Invariants (Covenant 2)

The SHACL validation enforces these hard quality constraints:

### Q1: No Retrocausation (Immutability)
- Observation snapshots form immutable DAG
- Workflow versions are immutable
- State transitions are monotonic (no time reversal)
- **Violation Example**: Workflow without version number

### Q2: Type Soundness (O ⊨ Σ)
- All split-join combinations must be in permutation matrix
- Data variables must have declared types
- Execution modes must be valid
- Resources must match capability requirements
- **Violation Example**: `type-mismatch.ttl` - integer output → string input

### Q3: Bounded Recursion (max_run_length ≤ 8)
- Chatman constant enforced: no operation > 8 ticks on critical path
- All cycles must have iteration limits ≤ 8
- Recursion depth must be bounded
- **Violation Example**: `unbounded-recursion.ttl` - cycle without MaxIterations

### Q4: Latency SLOs
- Hot path tasks: ≤ 8 ticks (2 nanoseconds)
- Warm path tasks: ≤ 100ms
- All tasks must declare expected duration
- Async tasks must have timeout policies
- **Violation Example**: Task without expected duration

### Q5: Resource Bounds
- Parallel tasks must declare concurrency limits
- Discriminator joins must declare quorum thresholds
- Critical sections must declare semaphore capacity
- Resource metrics must be monitored
- **Violation Example**: `missing-resource-bounds.ttl` - parallel task without MaxConcurrency

## Usage

### Validate a Single Workflow

```bash
# Run all validations
./ggen-marketplace/knhk-yawl-workflows/scripts/validate-shapes.sh \
    validation-examples/valid/simple-workflow.ttl

# Verbose output
./ggen-marketplace/knhk-yawl-workflows/scripts/validate-shapes.sh \
    validation-examples/valid/simple-workflow.ttl --verbose

# Skip Q invariants (only soundness)
./ggen-marketplace/knhk-yawl-workflows/scripts/validate-shapes.sh \
    validation-examples/valid/simple-workflow.ttl --no-q-invariants
```

### Validate All Examples

```bash
# Valid workflows (should all pass)
for file in validation-examples/valid/*.ttl; do
    echo "Validating: $file"
    ./ggen-marketplace/knhk-yawl-workflows/scripts/validate-shapes.sh "$file"
done

# Invalid workflows (should all fail with violations)
for file in validation-examples/invalid/*.ttl; do
    echo "Validating: $file"
    ./ggen-marketplace/knhk-yawl-workflows/scripts/validate-shapes.sh "$file" || true
done
```

## Expected Results

### Valid Workflows

All workflows in `valid/` should pass with exit code 0:

```
═══════════════════════════════════════════════════════════════════
  SHACL VALIDATION: simple-workflow.ttl
═══════════════════════════════════════════════════════════════════

INFO: Validating Turtle syntax: simple-workflow.ttl
SUCCESS: Turtle syntax valid

INFO: Running SHACL validation: q-invariants
SUCCESS: q-invariants: No violations

INFO: Running SHACL validation: workflow-soundness
SUCCESS: workflow-soundness: No violations

═══════════════════════════════════════════════════════════════════
  VALIDATION SUMMARY
═══════════════════════════════════════════════════════════════════

SUCCESS: All validations passed! ✓

Workflow satisfies all Q invariants and soundness constraints.
Ready for deployment.
```

### Invalid Workflows

All workflows in `invalid/` should fail with violations:

#### unbounded-recursion.ttl (Violates Q3)
```
ERROR: q-invariants: 2 violation(s) detected

Q3-VIOLATION: Task has cycle but no iteration limit
Q3-VIOLATION: No cycle detection mode declared

═══════════════════════════════════════════════════════════════════
  VALIDATION SUMMARY
═══════════════════════════════════════════════════════════════════

ERROR: Violations: 2 (HARD INVARIANTS BROKEN)

Covenant 2: Invariants Are Law
These violations BLOCK deployment. Fix all violations before proceeding.
```

#### type-mismatch.ttl (Violates Q2)
```
ERROR: q-invariants: 1 violation(s) detected
ERROR: workflow-soundness: 1 violation(s) detected

Q2-VIOLATION: Invalid AND-Discriminator combination not in permutation matrix
WS-D4: Variable type mismatch (integer → string)

═══════════════════════════════════════════════════════════════════
  VALIDATION SUMMARY
═══════════════════════════════════════════════════════════════════

ERROR: Violations: 2 (HARD INVARIANTS BROKEN)
```

#### missing-resource-bounds.ttl (Violates Q5)
```
ERROR: q-invariants: 2 violation(s) detected

Q5-VIOLATION: Parallel task missing MaxConcurrency
Q5-VIOLATION: Discriminator join missing quorum threshold

═══════════════════════════════════════════════════════════════════
  VALIDATION SUMMARY
═══════════════════════════════════════════════════════════════════

ERROR: Violations: 2 (HARD INVARIANTS BROKEN)
```

## Covenant 2: Enforcement Rules

**CRITICAL**: All violations (sh:Violation) BLOCK deployment.

### Violation Levels

1. **sh:Violation** (EXIT CODE 1)
   - Hard invariant broken
   - Workflow CANNOT deploy
   - Must be fixed before proceeding
   - Example: Unbounded recursion, type mismatch, missing resource bounds

2. **sh:Warning** (EXIT CODE 2)
   - Quality issue detected
   - Workflow CAN deploy but has problems
   - Should be fixed for production
   - Example: Degenerate split (split with only one branch)

3. **sh:Info** (EXIT CODE 0)
   - Advisory message
   - No blocking issue
   - Optimization opportunity
   - Example: Unused output variable

## Creating New Examples

### Valid Workflow Template

```turtle
@prefix yawl: <http://bitflow.ai/ontology/yawl/v2#> .
@prefix yawl-exec: <http://bitflow.ai/ontology/yawl/execution/v1#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

:MyWorkflow a yawl:WorkflowSpecification ;
    yawl:id "my-workflow-001" ;
    yawl:versionNumber "1.0.0" ;                    # Q1: Immutable version
    yawl:hasInputCondition :start ;                 # WS-C1: Exactly one input
    yawl:hasOutputCondition :end ;                  # WS-C2: Exactly one output
    yawl:hasTask :myTask .

:myTask a yawl:Task ;
    yawl-exec:executionMode yawl-exec:Synchronous ; # Q2: Valid mode
    yawl-exec:runtimeBehavior <http://...> ;        # Q: Must have implementation
    yawl:expectedDuration "PT0.000000002S"^^xsd:duration ; # Q4: Duration ≤ 8 ticks
    yawl:hasSplitType yawl:XOR ;                    # Q2: Valid combination
    yawl:hasJoinType yawl:XOR .                     # Q2: (XOR-XOR is valid)
```

### Invalid Workflow Template (for testing)

```turtle
# Intentionally violate specific Q invariant
:BadWorkflow a yawl:WorkflowSpecification ;
    yawl:id "bad-workflow-001" ;
    # ❌ Missing versionNumber (Q1 violation)
    yawl:hasInputCondition :start ;
    yawl:hasOutputCondition :end ;
    yawl:hasTask :badTask .

:badTask a yawl:Task ;
    # ❌ Missing executionMode (Q2 violation)
    # ❌ Missing runtimeBehavior (Q violation)
    # ❌ Missing expectedDuration (Q4 violation)
    yawl:hasSplitType yawl:AND ;
    yawl:hasJoinType yawl:Discriminator . # ❌ Invalid combination (Q2 violation)
```

## Integration with Development Workflow

### Pre-Commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

for file in $(git diff --cached --name-only | grep '\.ttl$'); do
    if [[ $file == ontology/workflows/* ]]; then
        ./ggen-marketplace/knhk-yawl-workflows/scripts/validate-shapes.sh "$file"
        if [ $? -eq 1 ]; then
            echo "ERROR: Workflow $file has Q invariant violations"
            echo "Fix violations before committing (Covenant 2: Invariants Are Law)"
            exit 1
        fi
    fi
done
```

### CI/CD Integration

```yaml
# .github/workflows/validate-workflows.yml
name: Validate YAWL Workflows

on: [push, pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2

      - name: Install dependencies
        run: |
          sudo apt-get install -y raptor2-utils
          pip install pyshacl

      - name: Validate all workflows
        run: |
          for file in ontology/workflows/**/*.ttl; do
            ./ggen-marketplace/knhk-yawl-workflows/scripts/validate-shapes.sh "$file"
          done
```

## Canonical References

- **DOCTRINE_2027.md** - Foundational narrative (Q principles)
- **DOCTRINE_COVENANT.md** - Covenant 2: Invariants Are Law
- **ontology/shacl/q-invariants.ttl** - Q constraint SHACL shapes
- **ontology/shacl/workflow-soundness.ttl** - Structural soundness shapes
- **ontology/yawl-pattern-permutations.ttl** - Valid pattern matrix

## Support

If validation fails unexpectedly:

1. Check Turtle syntax with `rapper -i turtle -o ntriples <file.ttl>`
2. Run validation with `--verbose` flag for detailed output
3. Compare against valid examples in `valid/`
4. Refer to Q invariant definitions in `DOCTRINE_COVENANT.md`
5. Check that all required ontologies are present in `ontology/`

**Remember**: Violations are not warnings. They are law (Covenant 2). Fix all violations before deployment.
