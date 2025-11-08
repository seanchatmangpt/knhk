# KNHK CLI Workflow Commands

## Usage

The workflow commands are integrated into the main `knhk` CLI using the noun-verb pattern:

```bash
knhk workflow <verb> [args]
```

## Available Commands

### Parse Workflow
```bash
knhk workflow parse <file> [--output <output-file>]
```

Parse a workflow from Turtle file and optionally save as JSON.

Example:
```bash
knhk workflow parse examples/simple-sequence.ttl
knhk workflow parse workflow.ttl --output workflow.json
```

### Register Workflow
```bash
knhk workflow register <file> [--state-store <path>]
```

Register a workflow specification in the state store.

Example:
```bash
knhk workflow register examples/simple-sequence.ttl
knhk workflow register workflow.json --state-store /custom/path
```

### Create Case
```bash
knhk workflow create <spec-id> [--data <json>] [--state-store <path>]
```

Create a new workflow case with optional input data.

Example:
```bash
knhk workflow create <spec-id>
knhk workflow create <spec-id> --data '{"customer_id":"12345","amount":1000.0}'
```

### Start Case
```bash
knhk workflow start <case-id> [--state-store <path>]
```

Start execution of a workflow case.

Example:
```bash
knhk workflow start <case-id>
```

### Execute Case
```bash
knhk workflow execute <case-id> [--state-store <path>]
```

Execute a workflow case.

Example:
```bash
knhk workflow execute <case-id>
```

### Get Case Status
```bash
knhk workflow get <case-id> [--state-store <path>]
```

Get the current status of a workflow case.

Example:
```bash
knhk workflow get <case-id>
```

### Cancel Case
```bash
knhk workflow cancel <case-id> [--state-store <path>]
```

Cancel a running workflow case.

Example:
```bash
knhk workflow cancel <case-id>
```

### List Cases
```bash
knhk workflow list [<spec-id>] [--state-store <path>]
```

List all cases, optionally filtered by workflow specification.

Example:
```bash
knhk workflow list
knhk workflow list <spec-id>
```

### List Patterns
```bash
knhk workflow patterns
```

List all 43 registered workflow patterns.

Example:
```bash
knhk workflow patterns
```

### Start REST API Server
```bash
knhk workflow serve [--port <port>] [--host <host>] [--state-store <path>]
```

Start the REST API server for workflow operations.

Example:
```bash
knhk workflow serve
knhk workflow serve --port 8080 --host 0.0.0.0
```

## Complete Example Workflow

```bash
# 1. Parse a workflow
knhk workflow parse examples/simple-sequence.ttl

# 2. Register the workflow
knhk workflow register examples/simple-sequence.ttl
# Output: Workflow registered: Simple Sequence Workflow (<spec-id>)

# 3. Create a case
knhk workflow create <spec-id> --data '{"input":"test"}'
# Output: Case created: <case-id>

# 4. Start the case
knhk workflow start <case-id>

# 5. Execute the case
knhk workflow execute <case-id>

# 6. Check status
knhk workflow get <case-id>

# 7. List all patterns
knhk workflow patterns
```

## State Store

By default, the state store is created at `./workflow_db`. You can specify a different path using `--state-store`:

```bash
knhk workflow register workflow.ttl --state-store /path/to/store
```

## Integration with Other Commands

The workflow engine integrates with other KNHK CLI commands:

- Use `knhk connect` to register connectors for workflow tasks
- Use `knhk pipeline` to execute workflows as part of ETL pipelines
- Use `knhk metrics` to monitor workflow performance

