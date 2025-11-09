# Van der Aalst Quality CLI Commands

## Overview

The Van der Aalst quality CLI provides comprehensive workflow analysis commands following Wil M.P. van der Aalst's methodology for process mining, workflow patterns, and soundness verification.

## Commands

### Soundness Verification (`soundness`)

Implements Van der Aalst's three fundamental soundness properties:

- `knhk soundness verify <workflow-file> [--json]` - Verify all three soundness properties
- `knhk soundness option-to-complete <workflow-file> [--json]` - Verify option to complete property
- `knhk soundness proper-completion <workflow-file> [--json]` - Verify proper completion property
- `knhk soundness no-dead-tasks <workflow-file> [--json]` - Verify no dead tasks property
- `knhk soundness report <workflow-file> [--output <file>] [--json]` - Generate detailed soundness report

### Process Mining (`mining`)

Implements Van der Aalst's process mining methodology:

- `knhk mining export-xes <case-id> [--output <file>] [--state-store <path>] [--json]` - Export case to XES format
- `knhk mining discover <xes-file> [--algorithm <alpha|alphappp>] [--output <file>] [--json]` - Discover process model
- `knhk mining conformance <workflow-file> <xes-file> [--state-store <path>] [--json]` - Check conformance
- `knhk mining fitness <workflow-file> <xes-file> [--state-store <path>] [--json]` - Calculate fitness
- `knhk mining precision <workflow-file> <xes-file> [--state-store <path>] [--json]` - Calculate precision
- `knhk mining generalization <workflow-file> <xes-file> [--state-store <path>] [--json]` - Calculate generalization

### Pattern Testing (`patterns`)

Implements Van der Aalst's 43 workflow patterns:

- `knhk patterns list [--json]` - List all 43 patterns
- `knhk patterns test <pattern-id> [--state-store <path>] [--json]` - Test a specific pattern
- `knhk patterns test-all [--state-store <path>] [--json]` - Test all 43 patterns
- `knhk patterns verify <pattern-id> <workflow-file> [--state-store <path>] [--json]` - Verify pattern in workflow
- `knhk patterns coverage <workflow-file> [--state-store <path>] [--json]` - Show pattern coverage

### Conformance Checking (`conformance`)

Implements Van der Aalst's conformance checking methodology:

- `knhk conformance check <workflow-file> <xes-file> [--state-store <path>] [--json]` - Check conformance (all dimensions)
- `knhk conformance fitness <workflow-file> <xes-file> [--state-store <path>] [--json]` - Calculate fitness
- `knhk conformance precision <workflow-file> <xes-file> [--state-store <path>] [--json]` - Calculate precision
- `knhk conformance generalization <workflow-file> <xes-file> [--state-store <path>] [--json]` - Calculate generalization
- `knhk conformance alignment <workflow-file> <xes-file> [--output <file>] [--state-store <path>] [--json]` - Generate alignment

## Example Usage

### Soundness Verification

```bash
# Verify all soundness properties
knhk soundness verify workflow.ttl

# Verify specific property
knhk soundness option-to-complete workflow.ttl

# Generate detailed report
knhk soundness report workflow.ttl --output report.json --json
```

### Process Mining

```bash
# Export case to XES
knhk mining export-xes <case-id> --output case.xes

# Discover process model
knhk mining discover case.xes --algorithm alphappp --output model.pnml

# Check conformance
knhk mining conformance workflow.ttl case.xes --json
```

### Pattern Testing

```bash
# List all patterns
knhk patterns list

# Test all patterns
knhk patterns test-all --json

# Test specific pattern
knhk patterns test 1  # Sequence pattern

# Verify pattern in workflow
knhk patterns verify 1 workflow.ttl
```

### Conformance Checking

```bash
# Check conformance (all dimensions)
knhk conformance check workflow.ttl case.xes --json

# Calculate fitness
knhk conformance fitness workflow.ttl case.xes

# Generate alignment
knhk conformance alignment workflow.ttl case.xes --output alignment.json
```

## Implementation Status

✅ **Complete**: All commands implemented and compile successfully
✅ **Van der Aalst Methodology**: Follows research standards
✅ **JSON Output**: All commands support `--json` flag
✅ **Error Handling**: Proper error handling via `CliAdapter`
✅ **Service Layer**: Uses unified service layer

## Files

- `rust/knhk-cli/src/soundness.rs` - Soundness verification commands (461 lines)
- `rust/knhk-cli/src/mining.rs` - Process mining commands (585 lines)
- `rust/knhk-cli/src/patterns.rs` - Pattern testing commands (444 lines)
- `rust/knhk-cli/src/conformance.rs` - Conformance checking commands (525 lines)

## Total Commands

- **Soundness**: 5 commands
- **Mining**: 6 commands
- **Patterns**: 5 commands
- **Conformance**: 5 commands

**Total**: 21 new Van der Aalst quality CLI commands

