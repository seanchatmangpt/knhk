# Automated XES Validation Full Loop

## van der Aalst Process Mining Validation

Automated validation loop that executes the complete van der Aalst validation process:
1. Execute workflow
2. Export to XES
3. Validate XES format
4. Compare with specification
5. Check conformance

## Automation Methods

### CLI Command: `knhk workflow validate-xes` (Recommended)

Automates the full validation loop using clap-noun-verb v3.4.0:
- Executes workflow
- Exports to XES
- Validates XES format
- Compares with specification
- Checks conformance

**Usage**:
```bash
knhk workflow validate-xes --spec-id <spec_id> [--output-dir <dir>]
```

**Output**:
- `<output-dir>/workflow_execution.xes` - XES event log
- `<output-dir>/validation_report.md` - Validation report

**Example**:
```bash
knhk workflow validate-xes --spec-id abc-123-def --output-dir ./validation_results
```

### Shell Script: `scripts/automate_xes_validation_loop.sh` (Legacy)

Legacy shell script for automated validation:
- Executes workflow tests
- Extracts XES files
- Validates XES format
- Generates validation report

**Usage**:
```bash
./scripts/automate_xes_validation_loop.sh
```

**Output**:
- `tmp/xes_validation/test_output.log` - Test output
- `tmp/xes_validation/*.xes` - XES files
- `tmp/xes_validation/validation_report.md` - Validation report

### Rust Test: `tests/automated_xes_validation_loop.rs`

Automated test that performs the full validation loop:
- Executes workflow
- Exports to XES
- Validates XES format
- Compares with specification
- Checks conformance

**Usage**:
```bash
cd rust/knhk-workflow-engine
cargo test --test automated_xes_validation_loop
```

## Validation Phases

### Phase 1: Execute Workflow
- Register workflow specification
- Create case
- Start case execution
- Wait for execution to complete

### Phase 2: Export to XES
- Export case execution to XES format
- Write XES file
- Verify XES export succeeded

### Phase 3: Validate XES Format
- Verify XES is valid XML
- Verify XES 2.0 compliance
- Verify required attributes (concept:name, time:timestamp, lifecycle:transition)
- Verify optional attributes (org:resource, pattern:id)

### Phase 4: Compare with Specification
- Import XES using process_mining library
- Extract activity names from XES
- Compare with workflow specification
- Verify all tasks appear in XES

### Phase 5: Check Conformance
- Use process discovery (Alpha++ algorithm)
- Discover Petri net from XES
- Compare discovered process with specification
- Verify conformance

## Validation Results

### Automated Test Results

**Test**: `test_automated_xes_validation_full_loop`
- ✅ Phase 1: Workflow execution - PASS
- ✅ Phase 2: XES export - PASS
- ✅ Phase 3: XES format validation - PASS
- ✅ Phase 4: Specification comparison - PASS
- ✅ Phase 5: Conformance checking - PASS

### Validation Checklist

- [x] Workflow executes successfully
- [x] XES export contains all events
- [x] XES format is valid (XES 2.0 compliant)
- [x] XES event log matches specification
- [x] Process discovery produces valid Petri net
- [x] Discovered process matches specification
- [x] Conformance validated

## Output Files

### Test Output
- `tmp/xes_validation/test_output.log` - Complete test output

### XES Files
- `tmp/xes_validation/*.xes` - Exported XES event logs

### Validation Report
- `tmp/xes_validation/validation_report.md` - Validation summary

## Next Steps

1. **Review Test Output**: Check `tmp/xes_validation/test_output.log` for detailed results
2. **Analyze XES Files**: Review exported XES files for event completeness
3. **Compare Specifications**: Compare XES event logs with workflow specifications
4. **Document Deviations**: Document any deviations from specification
5. **Continuous Validation**: Integrate into CI/CD pipeline

## Integration

### CI/CD Integration

Add to CI/CD pipeline:
```yaml
- name: XES Validation Full Loop
  run: |
    knhk workflow validate-xes --spec-id $WORKFLOW_SPEC_ID --output-dir ./validation_results
    cargo test --test automated_xes_validation_loop
```

### Scheduled Validation

Run periodically:
```bash
# Daily validation
0 0 * * * cd /path/to/knhk && ./scripts/automate_xes_validation_loop.sh
```

## Status

**Status**: ✅ COMPLETE - Full loop automated and tested

**Test Results**: ✅ All phases pass

**Next Steps**: Integrate into CI/CD pipeline

---

**Last Updated**: $(date)  
**Status**: ✅ COMPLETE - Full loop automated

