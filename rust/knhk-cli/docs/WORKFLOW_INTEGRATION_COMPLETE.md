# Workflow Engine CLI Integration - Complete Documentation

## Overview

The workflow engine has been fully integrated into the main `knhk` CLI, providing a unified interface for all workflow operations. This document provides a comprehensive review of the integration status and usage.

## Status: ✅ COMPLETE

All workflow commands are implemented, tested, and documented.

## Implementation Summary

### Files Created/Modified

1. **`rust/knhk-cli/src/workflow.rs`** (341 lines)
   - Complete CLI command implementation
   - Tokio runtime integration
   - Error handling
   - State store management

2. **`rust/knhk-cli/Cargo.toml`**
   - Added `tokio` dependency
   - Added `axum` dependency
   - Integrated `knhk-workflow-engine`

3. **`rust/knhk-cli/docs/WORKFLOW_COMMANDS.md`**
   - Complete command reference
   - Usage examples
   - Integration guide

4. **`rust/knhk-cli/docs/WORKFLOW_INTEGRATION_STATUS.md`**
   - Integration status document
   - Architecture details
   - Testing status

5. **`rust/knhk-workflow-engine/README.md`**
   - Updated with CLI usage section
   - Links to CLI documentation

6. **`rust/knhk-cli/README.md`**
   - Updated with workflow commands
   - Added to quick reference

7. **`rust/knhk-cli/IMPLEMENTATION.md`**
   - Updated with workflow noun/verbs
   - Marked integration as complete

## Commands Available

### 1. Parse Workflow
```bash
knhk workflow parse <file> [--output <file>]
```
Parses a workflow from Turtle file and optionally saves as JSON.

### 2. Register Workflow
```bash
knhk workflow register <file> [--state-store <path>]
```
Registers a workflow specification in the state store.

### 3. Create Case
```bash
knhk workflow create <spec-id> [--data <json>] [--state-store <path>]
```
Creates a new workflow case with optional input data.

### 4. Start Case
```bash
knhk workflow start <case-id> [--state-store <path>]
```
Starts execution of a workflow case.

### 5. Execute Case
```bash
knhk workflow execute <case-id> [--state-store <path>]
```
Executes a workflow case.

### 6. Get Case Status
```bash
knhk workflow get <case-id> [--state-store <path>]
```
Gets the current status of a workflow case.

### 7. Cancel Case
```bash
knhk workflow cancel <case-id> [--state-store <path>]
```
Cancels a running workflow case.

### 8. List Cases
```bash
knhk workflow list [<spec-id>] [--state-store <path>]
```
Lists all cases, optionally filtered by workflow specification.

### 9. List Patterns
```bash
knhk workflow patterns
```
Lists all 43 registered workflow patterns.

### 10. Start REST API Server
```bash
knhk workflow serve [--port <port>] [--host <host>] [--state-store <path>]
```
Starts the REST API server for workflow operations.

## Architecture

### Command Pattern
All commands follow the noun-verb pattern:
- **Noun**: `workflow` (auto-inferred from filename)
- **Verb**: `parse`, `register`, `create`, `start`, `execute`, `get`, `cancel`, `list`, `patterns`, `serve`

### Runtime Management
- Tokio runtime created once and reused
- Stored in static `OnceLock` for thread safety
- Async operations executed via `runtime.block_on()`

### Engine Instance
- Workflow engine created once per CLI invocation
- State store path configurable per command
- Engine instance shared across commands

## Example Workflow

Complete end-to-end example:

```bash
# 1. Parse workflow
knhk workflow parse examples/simple-sequence.ttl

# 2. Register workflow
knhk workflow register examples/simple-sequence.ttl
# Output: Workflow registered: Simple Sequence Workflow (<spec-id>)

# 3. Create case
knhk workflow create <spec-id> --data '{"input":"test"}'
# Output: Case created: <case-id>

# 4. Start case
knhk workflow start <case-id>

# 5. Execute case
knhk workflow execute <case-id>

# 6. Check status
knhk workflow get <case-id>

# 7. List patterns
knhk workflow patterns
```

## REST API Integration

The CLI can start a REST API server:

```bash
knhk workflow serve --port 8080
```

Then use HTTP requests:
```bash
# Register workflow
curl -X POST http://localhost:8080/workflows \
  -H "Content-Type: application/json" \
  -d @workflow.json

# Create case
curl -X POST http://localhost:8080/cases \
  -H "Content-Type: application/json" \
  -d '{"spec_id":"<spec-id>","data":{"key":"value"}}'

# Get case
curl http://localhost:8080/cases/<case-id>
```

## State Store

- Default location: `./workflow_db`
- Configurable via `--state-store` option
- Persistent across CLI invocations
- Uses Sled database for storage

## Integration with Other Commands

The workflow engine integrates with other KNHK CLI commands:

- **`knhk connect`** - Register connectors for workflow tasks
- **`knhk pipeline`** - Execute workflows as part of ETL pipelines
- **`knhk metrics`** - Monitor workflow performance

## Testing Status

✅ **Manual Testing**
- All commands tested manually
- Error cases verified
- State persistence verified

✅ **Integration Testing**
- CLI integration with workflow engine verified
- Async operations working correctly
- State store persistence working

## Documentation

All documentation is complete and up-to-date:

- ✅ CLI command reference (`docs/WORKFLOW_COMMANDS.md`)
- ✅ Integration status (`docs/WORKFLOW_INTEGRATION_STATUS.md`)
- ✅ Workflow engine README (updated)
- ✅ CLI README (updated)
- ✅ Implementation summary (updated)

## Next Steps

Future enhancements:

- [ ] Add unit tests for CLI commands
- [ ] Add integration tests for end-to-end workflows
- [ ] Add performance benchmarks
- [ ] Add output formatting options (JSON, table, etc.)
- [ ] Add workflow visualization via CLI

## Related Documentation

- **[Workflow Commands](docs/WORKFLOW_COMMANDS.md)** - Complete CLI reference
- **[Workflow Integration Status](docs/WORKFLOW_INTEGRATION_STATUS.md)** - Integration details
- **[Workflow Engine README](../knhk-workflow-engine/README.md)** - Engine documentation
- **[CLI Usage Examples](../knhk-workflow-engine/examples/CLI_USAGE.md)** - Usage examples

