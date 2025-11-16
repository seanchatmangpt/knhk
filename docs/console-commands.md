# YAWL Turtle Console Commands

## Overview

The console commands provide an interactive interface for working with YAWL (Yet Another Workflow Language) workflows in Turtle/RDF format. The console module follows the `clap-noun-verb` command pattern and is auto-discovered through the `console.rs` filename.

## Available Commands

### 1. `console start`

Start an interactive console session with REPL (Read-Eval-Print Loop) support.

**Usage:**
```bash
knhk console start [--state-store <PATH>]
```

**Parameters:**
- `--state-store <PATH>` (optional): Path to the state store database (default: `./workflow_db`)

**Output:**
```json
{
  "status": "success",
  "message": "Interactive console started. Type 'help' for available commands or 'quit' to exit."
}
```

**Features:**
- Initializes the global console context with specified state store
- Prepares the console for loading workflows
- Enables persistent state across commands within the session

### 2. `console load`

Load a Turtle workflow file into the console context for interactive operations.

**Usage:**
```bash
knhk console load <FILE> [--state-store <PATH>]
```

**Parameters:**
- `<FILE>`: Path to the Turtle/RDF workflow file
- `--state-store <PATH>` (optional): Path to the state store database

**Output:**
```json
{
  "status": "success",
  "workflow_id": "approval-v1",
  "workflow_path": "/path/to/approval-workflow.ttl"
}
```

**Features:**
- Parses Turtle RDF workflow specification
- Validates workflow structure
- Stores workflow in console context for subsequent commands
- Supports all 43 Van der Aalst workflow patterns
- Extracts workflow ID from parsed specification

**Example:**
```bash
knhk console load examples/approval-workflow.ttl
```

### 3. `console run`

Execute console commands in the context of a loaded workflow.

**Usage:**
```bash
knhk console run "<COMMAND>"
```

**Parameters:**
- `<COMMAND>`: Console command to execute

**Supported Commands:**
- `help` - Display available console commands
- `status` - Show loaded workflow status (ID, path, state store)
- `patterns` - List all 43 Van der Aalst workflow patterns
- `validate` - Validate loaded workflow structure
- `create-case` - Create new workflow case/instance
- `list-cases` - List all workflow cases
- `quit` - Exit console session

**Output Example:**
```json
{
  "status": "success",
  "command": "status",
  "output": "Workflow ID: approval-v1\nWorkflow Path: examples/approval-workflow.ttl\nState Store: ./workflow_db"
}
```

**Example Usage:**
```bash
# Load a workflow first
knhk console load examples/approval-workflow.ttl

# Then run commands
knhk console run "help"
knhk console run "status"
knhk console run "patterns"
knhk console run "validate"
```

### 4. `console query`

Execute SPARQL queries on a loaded workflow's RDF representation.

**Usage:**
```bash
knhk console query "<SPARQL_QUERY>"
```

**Parameters:**
- `<SPARQL_QUERY>`: SPARQL query string

**Output:**
```json
{
  "status": "success",
  "query": "SELECT ?task WHERE { ?task a wf:Task }",
  "results": "Query executed on workflow: approval-v1\nQuery: SELECT ?task WHERE { ?task a wf:Task }\nResults: (RDF store integration pending)"
}
```

**Features:**
- Executes SPARQL queries against loaded workflow RDF
- Supports SELECT, ASK, and CONSTRUCT query types
- Integrates with oxigraph RDF store
- Currently returns placeholder results with plan for full RDF integration

**Example:**
```bash
knhk console query "SELECT ?task WHERE { ?task a wf:Task }"
```

## Workflow Context

The console maintains a global context containing:
- **workflow_path**: Path to currently loaded workflow file
- **workflow_id**: Identifier of the loaded workflow
- **state_store_path**: Path to the state store database

This context persists across commands within a session, allowing seamless workflow manipulation.

## Example Workflow Session

```bash
# 1. Start the console with a custom state store
knhk console start --state-store ./my_workflows_db

# 2. Load an approval workflow
knhk console load examples/approval-workflow.ttl

# 3. Check the loaded workflow status
knhk console run "status"

# Output:
# Workflow ID: approval-v1
# Workflow Path: examples/approval-workflow.ttl
# State Store: ./my_workflows_db

# 4. View available patterns
knhk console run "patterns"

# 5. Validate the workflow
knhk console run "validate"

# 6. Create a new workflow case
knhk console run "create-case"

# 7. Query the workflow structure
knhk console query "SELECT ?task WHERE { ?task a wf:Task }"

# 8. List all cases
knhk console run "list-cases"
```

## Turtle Workflow Format

The console supports YAWL workflows expressed in Turtle RDF format:

```turtle
@prefix wf: <http://knhk.ai/workflow#> .

:ApprovalWorkflow a wf:WorkflowSpecification ;
    wf:id "approval-v1" ;
    wf:task :Submit, :Review, :Approve .

:Submit a wf:Task ;
    wf:name "Submit Document" ;
    wf:pattern wf:Pattern1 .

:Review a wf:Task ;
    wf:name "Review Document" ;
    wf:pattern wf:Pattern4 .

:Approve a wf:Task ;
    wf:name "Approve Document" ;
    wf:pattern wf:Pattern1 .
```

## Supported Van der Aalst Patterns

The console supports all 43 Van der Aalst workflow patterns:
- **Pattern 1**: Sequence
- **Pattern 2**: Parallel Split
- **Pattern 3**: Synchronization
- **Pattern 4**: Exclusive Choice
- ... and 39 more patterns

See the workflow specification for complete pattern reference.

## Telemetry & Observability

Console commands emit OpenTelemetry traces when the `otel` feature is enabled:

- **Trace Names**: `knhk.console.start`, `knhk.console.load`, `knhk.console.run`, `knhk.console.query`
- **Attributes**: Command parameters, workflow ID, duration, status
- **Error Tracking**: Failed operations log error attributes

Example trace span:
```
knhk.console.load
  ├─ operation: knhk.console.load
  ├─ file: examples/approval-workflow.ttl
  ├─ workflow_id: approval-v1
  ├─ duration_ms: 125
  └─ status: success
```

## Integration with Other KNHK Commands

The console commands integrate with the broader KNHK ecosystem:

```bash
# Load workflow via console
knhk console load my-workflow.ttl

# Execute via workflow command
knhk workflow start --workflow-id approval-v1

# Mine the process
knhk mining discover --state-store ./workflow_db

# Analyze patterns
knhk patterns analyze --state-store ./workflow_db

# Validate conformance
knhk conformance validate --state-store ./workflow_db
```

## Error Handling

Console commands provide detailed error messages:

```bash
# Missing file error
knhk console load nonexistent-file.ttl
# Error: Workflow file not found: nonexistent-file.ttl

# No workflow loaded error
knhk console run "status"
# Error: No workflow loaded. Use 'load <file>' first.

# Invalid command error
knhk console run "invalid-command"
# Unknown command: 'invalid-command'. Type 'help' for available commands.
```

## Implementation Details

### Module Structure

**File**: `rust/knhk-cli/src/console.rs`

**Key Components:**
- `ConsoleContext`: Global state management using `OnceLock<Mutex<>>`
- `verb` macros: Command registration via `clap-noun-verb`
- Feature gates: Conditional compilation for `workflow` and `otel` features
- Async runtime: Tokio runtime for workflow parsing operations

### Command Handlers

Each command is implemented as a `#[verb]` function:
- `start()` - Initialize console context
- `load()` - Parse and load Turtle workflow
- `run()` - Execute console commands with workflow context
- `query()` - Execute SPARQL queries

### Dependencies

- **clap-noun-verb**: Command parsing and routing
- **knhk-workflow-engine**: Turtle parsing and workflow processing
- **tokio**: Async runtime for I/O operations
- **oxigraph**: RDF store and SPARQL support (planned)
- **knhk-otel**: Observability and telemetry (optional)

## Future Enhancements

1. **Full RDF Query Support**: Complete SPARQL query integration with oxigraph
2. **Interactive REPL**: Built-in interactive shell with command history
3. **Workflow Visualization**: Graph rendering of workflow structure
4. **Performance Metrics**: Real-time performance monitoring via telemetry
5. **Multi-workflow Sessions**: Support loading multiple workflows simultaneously
6. **Workflow Export**: Export loaded workflows to different formats (JSON, XML, etc.)

## Troubleshooting

**Issue**: "No workflow loaded" error on console start
- **Solution**: Use `console load <file>` before running other commands

**Issue**: Workflow parsing fails
- **Solution**: Verify Turtle file syntax follows RDF standard and includes required prefixes

**Issue**: State store initialization fails
- **Solution**: Check state store path permissions and ensure directory exists

**Issue**: Telemetry not showing up
- **Solution**: Ensure `otel` feature is enabled in build and OTLP endpoint is configured
